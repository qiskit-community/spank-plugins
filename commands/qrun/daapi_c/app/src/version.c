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

extern const char *DAAPI_ENDPOINT;

int main(int argc, char *argv[]) {

  int rc = 0;

  daapi_init();

  struct ClientBuilder *builder = daapi_bldr_new(DAAPI_ENDPOINT);
  if (!builder) {
    printf("Failed to create a builder.\n");
    return -1;
  }

  rc = daapi_bldr_set_exponential_backoff_retry(builder, 5, 2, 1, 10);
  if (rc < 0) {
    printf("Failed to enable retries. rc=%d\n", rc);
  }

  struct Client *client = daapi_cli_new(builder);
  if (!client) {
    daapi_free_builder(builder);
    return -1;
  }
  rc = daapi_free_builder(builder);

  const char *ver = daapi_cli_get_version(client);
  if (ver) {
    printf("%s\n", ver);
    rc = daapi_free_string((char *)ver);
    if (rc < 0) {
      printf("Failed to free a string(%d)\n", __LINE__);
    }
  }

  rc = daapi_free_client(client);
  if (rc < 0) {
    printf("Failed to free Client(%p). rc=%d\n", client, rc);
  }

  return 0;
}
