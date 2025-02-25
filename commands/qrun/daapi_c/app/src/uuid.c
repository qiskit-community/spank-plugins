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

#include "direct_access_capi.h"

int main(int argc, char *argv[]) {
  for (int i = 0; i < 10; i++) {
    const char *job_id = daapi_uuid_v4_new();
    printf("UUIDv4: %s\n", job_id);
    daapi_free_string((char *)job_id);
  }
  return 0;
}
