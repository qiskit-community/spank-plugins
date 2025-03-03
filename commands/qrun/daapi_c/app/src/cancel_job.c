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

#include "cjson/cJSON.h"
#include "direct_access_capi.h"

extern const char *IAM_APIKEY;
extern const char *SERVICE_CRN;
extern const char *IAM_ENDPOINT;
extern const char *DAAPI_ENDPOINT;

int main(int argc, char *argv[]) {

  int rc = 0;

  if (argc != 2) {
    printf("Missing argument. cancel_job <job_id>\n");
    return -1;
  }

  struct ClientBuilder *builder = daapi_bldr_new(DAAPI_ENDPOINT);
  if (!builder) {
    printf("Failed to create a builder.\n");
    return -1;
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

  struct Client *client = daapi_cli_new(builder);
  if (!client) {
    printf("Failed to create Client.\n");
    goto free_builder;
  }

  rc = daapi_cli_cancel_job(client, argv[1], false);
  if (rc < 0) {
    printf("Failed to cancel job(%s)\n", argv[1]);
  }

  rc = daapi_free_client(client);
  if (rc < 0) {
    printf("Failed to free Client(%p). rc=%d\n", client, rc);
  }

free_builder:
  daapi_free_builder(builder);

  return 0;
}
