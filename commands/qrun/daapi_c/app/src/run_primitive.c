/*
# This code is part of Qiskit.
#
# (C) Copyright IBM 2025.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.
*/
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

#include "cjson/cJSON.h"
#include "direct_access_capi.h"

extern const char *IAM_APIKEY;
extern const char *SERVICE_CRN;
extern const char *IAM_ENDPOINT;
extern const char *S3_ENDPOINT;
extern const char *AWS_ACCESS_KEY_ID;
extern const char *AWS_SECRET_ACCESS_KEY;
extern const char *S3_REGION;
extern const char *S3_BUCKET;
extern const char *DAAPI_ENDPOINT;

int main(int argc, char *argv[]) {

  int rc = 0;

  if (argc != 4) {
    printf("run_primitive <backend> <primitive type> <PUBs JSON file>\n");
    return -1;
  }

  ProgramId type;
  if (strcmp(argv[2], "sampler") == 0)
    type = SAMPLER;
  else if (strcmp(argv[2], "estimator") == 0)
    type = ESTIMATOR;
  else {
    printf("Unknown primitive type: %s\n", argv[2]);
    return -1;
  }

  struct ClientBuilder *builder = daapi_bldr_new(DAAPI_ENDPOINT);
  if (!builder) {
    printf("Failed to create a builder.\n");
    return -1;
  }

  rc = daapi_bldr_enable_iam_auth(builder, IAM_APIKEY, SERVICE_CRN,
                                  IAM_ENDPOINT);
  if (rc < 0)
    printf("Failed to enable IAM auth. rc=%d\n", rc);

  rc = daapi_bldr_set_timeout(builder, 60.0);
  if (rc < 0)
    printf("Failed to enable timeout. rc=%d\n", rc);

  rc = daapi_bldr_set_exponential_backoff_retry(builder, 5, 2, 1, 10);
  if (rc < 0)
    printf("Failed to enable retries. rc=%d\n", rc);

  rc = daapi_bldr_set_s3_bucket(builder, AWS_ACCESS_KEY_ID,
                                AWS_SECRET_ACCESS_KEY, S3_ENDPOINT, S3_BUCKET,
                                S3_REGION);
  if (rc < 0)
    printf("Failed to set S3 params. rc=%d\n", rc);

  struct Client *client = daapi_cli_new(builder);
  if (!client) {
    daapi_free_builder(builder);
    return -1;
  }
  rc = daapi_free_builder(builder);

  FILE *fp = fopen(argv[3], "r");
  if (!fp) {
    printf("Failed to open PUBs file (%s)\n", argv[1]);
    return -1;
  }

  fseeko(fp, 0, SEEK_END);
  long size = ftello(fp);
  char *bufp = malloc(size);
  char *curr_ptr = bufp;
  fseeko(fp, 0, SEEK_SET);
  while(size > 0) {
    size_t sz = fread(curr_ptr, 1, size, fp);
    size -= sz;
    curr_ptr += sz;
  }
  fclose(fp);

  struct PrimitiveJob *job = daapi_cli_run_primitive(
      client, argv[1], type, 300, DEBUG, bufp, NULL);
  free(bufp);
  if (job) {
    JobStatus final_state;
    bool is_running = false;
    bool is_in_final_state = false;

    rc = daapi_prim_is_running(job, &is_running);
    printf("is_running %d, %d\n", rc, is_running);
    rc = daapi_prim_is_in_final_state(job, &is_in_final_state);
    printf("is_in_final_state %d, %d\n", rc, is_in_final_state);

    const char *job_id = daapi_prim_get_job_id(job);
    printf("JOB ID = %s\n", job_id);

    rc = daapi_prim_wait_for_final_state(job, &final_state);
    printf("wait_for rc=%d, final_state=%d\n", rc, final_state);

    const char *result = daapi_prim_get_result_as_string(job);
    if (result) {
      cJSON *json = cJSON_Parse(result);
      printf("%s\n", cJSON_PrintUnformatted(json));
      cJSON_free(json);
      daapi_free_string((char *)result);
    }

    const char *logs = daapi_prim_get_logs(job);
    if (logs) {
      printf("%s\n", logs);
      daapi_free_string((char *)logs);
    }

    struct Metrics* metrics = daapi_cli_get_metrics(client, job_id);
    if (metrics) {
      printf("created_time: %s, end_time: %s, quantum_nanoseconds: %ld\n",
             metrics->created_time, metrics->end_time,
             metrics->quantum_nanoseconds);
      daapi_free_metrics(metrics);
    }

    rc = daapi_free_primitive(job);
    if (rc < 0) {
      printf("Failed to free Job(%p). rc=%d\n", job, rc);
    }

    rc = daapi_cli_delete_job(client, job_id);
    if (rc < 0) {
      printf("Failed to delete Job(%s). rc=%d\n", job_id, rc);
    }

    rc = daapi_free_string((char *)job_id);
    if (rc < 0) {
      printf("Failed to free job ID. rc=%d\n", rc);
    }
  }

  rc = daapi_free_client(client);
  if (rc < 0)
    printf("Failed to free Client(%p). rc=%d\n", client, rc);

  return 0;
}
