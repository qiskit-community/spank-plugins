/*
 * This code is part of Qiskit.
 *
 * (C) Copyright IBM 2025.
 *
 * This code is licensed under the Apache License, Version 2.0. You may
 * obtain a copy of this license in the LICENSE.txt file in the root directory
 * of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
 *
 * Any modifications or derivative works of this code must retain this
 * copyright notice, and modified files need to carry a notice indicating
 * that they have been altered from the originals.
 */
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <inttypes.h>

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

static struct ClientBuilder* create_builder() {

  int rc = DAAPI_SUCCESS;

  struct ClientBuilder *builder = daapi_bldr_new(DAAPI_ENDPOINT);
  if (!builder) {
    printf("Failed to create a builder.\n");
    return NULL;
  }

  rc = daapi_bldr_enable_iam_auth(builder, IAM_APIKEY, SERVICE_CRN,
                                  IAM_ENDPOINT);
  if (rc < 0) {
    printf("Failed to enable IAM auth. rc=%d\n", rc);
    goto free_builder;
  }

  rc = daapi_bldr_set_timeout(builder, 60.0);
  if (rc < 0) {
    printf("Failed to enable timeout. rc=%d\n", rc);
    goto free_builder;
  }

  rc = daapi_bldr_set_exponential_backoff_retry(builder, 5, 2, 1, 10);
  if (rc < 0) {
    printf("Failed to enable retries. rc=%d\n", rc);
    goto free_builder;
  }

  rc = daapi_bldr_set_s3_bucket(builder, AWS_ACCESS_KEY_ID,
                                AWS_SECRET_ACCESS_KEY, S3_ENDPOINT, S3_BUCKET,
                                S3_REGION);
  if (rc < 0) {
    printf("Failed to set S3 params. rc=%d\n", rc);
    goto free_builder;
  }

  return builder;

free_builder:
  daapi_free_builder(builder);
  return NULL;
}

static int upload_pubs_to_s3(const char* filename, struct S3Client *s3, const char* object_name) {

  int rc = 0;
  FILE *fp = fopen(filename, "r");
  if (!fp) {
    printf("Failed to open a PUBs JSON file (%s)\n", filename);
    return -1;
  }

  fseeko(fp, 0, SEEK_END);
  long size = ftello(fp);
  char *bufp = malloc(size + 1);
  memset(bufp, '\0', size + 1);
  char *curr_ptr = bufp;
  fseeko(fp, 0, SEEK_SET);
  while(size > 0) {
    size_t sz = fread(curr_ptr, 1, size, fp);
    size -= sz;
    curr_ptr += sz;
  }
  fclose(fp);

  rc = daapi_s3cli_put_object_as_string(s3, S3_BUCKET, object_name, bufp);
  if (rc < 0) {
    printf("Failed to upload job input to S3.\n");
    return -1;
  }
  free(bufp);
  return 0;
}

int main(int argc, char *argv[]) {

  int rc = 0;

  if (argc != 4) {
    printf("run_job <backend> <primitive type> <PUBs JSON file>\n");
    return -1;
  }

  daapi_init();

  struct ClientBuilder *builder = create_builder();
  if (!builder) {
    printf("Failed to create a builder.\n");
    return -1;
  }

  /*
   * Create C API client
   */
  struct Client *client = daapi_cli_new(builder);
  if (!client) {
    daapi_free_builder(builder);
    return -1;
  }
  /* builder is no longer needed. */
  rc = daapi_free_builder(builder);

  /*
   * Create S3 Client to upload & download objects
   */
  struct S3Client *s3 = daapi_s3cli_new(S3_ENDPOINT, AWS_ACCESS_KEY_ID,
                                        AWS_SECRET_ACCESS_KEY, S3_REGION);
  if (!s3) {
    printf("Failed to create S3Client. Exiting..\n");
    return -1;
  }
  printf("S3Client = %p\n", s3);

  /* Generate UUIDv4 as job ID */
  const char *job_id = daapi_uuid_v4_new();
  printf("Job ID = %s\n", job_id);

  /*
   * Upload primitive input to S3 storage
   */
  char input_obj_name[256];
  char results_obj_name[256];
  char logs_obj_name[256];
  memset(input_obj_name, '\0', sizeof(input_obj_name));
  memset(results_obj_name, '\0', sizeof(results_obj_name));
  memset(logs_obj_name, '\0', sizeof(logs_obj_name));
  snprintf(input_obj_name, sizeof(input_obj_name), "%s_input.json", job_id);
  snprintf(results_obj_name, sizeof(results_obj_name), "%s_results.json", job_id);
  snprintf(logs_obj_name, sizeof(logs_obj_name), "%s_logs.txt", job_id);

  rc = upload_pubs_to_s3(argv[3], s3, input_obj_name);
  if (rc < 0) {
    printf("Failed to upload job input to S3.\n");
    return -1;
  }
 
  char payload[2048];
  memset(payload, '\0', sizeof(payload));

  /* Generate presigned URL for getting job input from S3 */
  const char* input_url = daapi_s3cli_get_presigned_url_for_get(s3, S3_BUCKET, input_obj_name, 83400);
  if (!input_url) {
    printf("Failed to generate presigned URL for input.\n");
    return -1;
  }

  /* Generate presigned URL for putting job results to S3 */
  const char* results_url = daapi_s3cli_get_presigned_url_for_put(s3, S3_BUCKET, results_obj_name, 83400);
  if (!results_url) {
    printf("Failed to generate presigned URL for results.\n");
    return -1;
  }

  /* Generate presigned URL for putting job logs to S3 */
  const char* logs_url = daapi_s3cli_get_presigned_url_for_put(s3, S3_BUCKET, logs_obj_name, 83400);
  if (!logs_url) {
    printf("Failed to generate presigned URL for logs.\n");
    return -1;
  }

  /*
   * Construct payload to run job
   */
  snprintf(
    payload,
    sizeof(payload),
    "{\
       \"id\": \"%s\",\
       \"backend\": \"%s\",\
       \"program_id\": \"%s\",\
       \"log_level\": \"debug\",\
       \"timeout_secs\": 432000,\
       \"storage\": {\
         \"input\": {\"presigned_url\": \"%s\", \"type\": \"s3_compatible\"},\
         \"results\": {\"presigned_url\": \"%s\", \"type\": \"s3_compatible\"},\
         \"logs\": {\"presigned_url\": \"%s\", \"type\": \"s3_compatible\"}\
       }\
    }",
    job_id, argv[1], argv[2], input_url, results_url, logs_url);

  daapi_free_string((char*)input_url);
  daapi_free_string((char*)results_url);
  daapi_free_string((char*)logs_url);

  /*
   * Submit job and wait for its completion.
   */
  const char* running_job_id = daapi_cli_run_job(client, payload);
  if (!running_job_id) {
    printf("Failed to run a job.\n");
    return -1;
  }
  daapi_free_string((char*)running_job_id);

  JobStatus status = RUNNING;
  while(1) {
    rc = daapi_cli_get_job_status(client, job_id, &status); 
    if (rc == DAAPI_SUCCESS && status != RUNNING) {
      break;
    }
    sleep(1);
  } 
  printf("Job %s was completed. Final state = %d\n", job_id, status);

  if (status == COMPLETED) {
    /*
     * Once the job completes, the results and logs will be uploaded to the S3 compatible object
     * storage using the presigned URLs
     */
    const char* results = daapi_s3cli_get_object_as_string(s3, S3_BUCKET, results_obj_name);
    if (results) {
      printf("results: %s\n", results);
      daapi_free_string((char*)results);
    }
    else {
      printf("Failed to retrieve the results from S3.\n");
    }

    const char* logs = daapi_s3cli_get_object_as_string(s3, S3_BUCKET, logs_obj_name);
    if (logs) {
      printf("logs: %s\n", logs);
      daapi_free_string((char*)logs);
    }
    else {
      printf("Failed to retrieve the logs from S3.\n");
    }

    /* Retrieves usage metrics */
    struct Metrics* metrics = daapi_cli_get_metrics(client, job_id);
    if (metrics) {
      printf("created_time: %s, end_time: %s, quantum_nanoseconds: %" PRId64 "\n",
             metrics->created_time, metrics->end_time,
             metrics->quantum_nanoseconds);
      daapi_free_metrics(metrics);
    }
  }

  /*
   * Clean up
   */
  rc = daapi_cli_delete_job(client, job_id);
  if (rc < 0) {
    printf("Failed to delete Job(%s). rc=%d\n", job_id, rc);
  }

  rc = daapi_free_client(client);
  if (rc < 0) {
    printf("Failed to free Client(%p). rc=%d\n", client, rc);
  }

  return 0;
}
