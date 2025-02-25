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
#include <unistd.h>

#include "direct_access_capi.h"

extern const char *IAM_APIKEY;
extern const char *SERVICE_CRN;
extern const char *IAM_ENDPOINT;
extern const char *DAAPI_ENDPOINT;

int main(int argc, char *argv[]) {

  int rc = 0;

  daapi_init();

  struct ClientBuilder *builder = daapi_bldr_new(DAAPI_ENDPOINT);
  if (!builder) {
    printf("Failed to create a builder.\n");
    return -1;
  }

  printf("builder = %p\n", builder);
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

  struct Client *client = daapi_cli_new(builder);
  if (!client) {
    daapi_free_builder(builder);
    return -1;
  }
  rc = daapi_free_builder(builder);
  printf("client = %p\n", client);
  const char *ver = daapi_cli_get_version(client);
  if (ver) {
    printf("VER: %s\n", ver);
    rc = daapi_free_string((char *)ver);
    if (rc < 0)
      printf("Failed to free a string(%d)\n", __LINE__);
  }

  struct BackendList *backends = daapi_cli_list_backends(client);
  if (backends) {
    for (size_t i = 0; i < backends->length; i++) {
      printf("%s %d\n",
             backends->backends[i].name,
             backends->backends[i].status);
    }
    rc = daapi_free_backend_list(backends);
    if (rc < 0)
      printf("Failed to free BackendList(%p). rc=%d\n", backends, rc);
  }

  const char *props = daapi_cli_get_backend_properties(client, "fake_brisbane");
  if (props) {
    printf("%s\n", props);
    daapi_free_string((char *)props);
  }

  const char *config =
      daapi_cli_get_backend_configuration(client, "fake_brisbane");
  if (config) {
    printf("%s\n", config);
    daapi_free_string((char *)config);
  }

  struct JobList *jobs = daapi_cli_list_jobs(client);
  if (jobs) {
    for (size_t i = 0; i < jobs->length; i++) {
      printf("id(%s), status(%d), program_id(%d) quantum_ns(%lld) created_time(%s) end_time(%s)\n",
             jobs->jobs[i].id,
             jobs->jobs[i].status,
             jobs->jobs[i].program_id,
             jobs->jobs[i].metrics.quantum_nanoseconds,
             jobs->jobs[i].metrics.created_time,
             jobs->jobs[i].metrics.end_time);
    }
    rc = daapi_free_job_list(jobs);
    if (rc < 0)
      printf("Failed to free JobList(%p). rc=%d\n", jobs, rc);
  }
  rc = daapi_free_client(client);
  if (rc < 0)
    printf("Failed to free Client(%p). rc=%d\n", client, rc);

  return 0;
}
