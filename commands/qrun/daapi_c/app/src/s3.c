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

extern const char *IAM_APIKEY;
extern const char *SERVICE_CRN;
extern const char *IAM_ENDPOINT;
extern const char *S3_ENDPOINT;
extern const char *AWS_ACCESS_KEY_ID;
extern const char *AWS_SECRET_ACCESS_KEY;
extern const char *S3_REGION;
extern const char *S3_BUCKET;
extern const char *DAAPI_ENDPOINT;
extern const char *OBJECT_FOR_GET;
extern const char *OBJECT_FOR_PUT;
extern const char *OBJECT_AS_STR;
extern const char *OBJECT_AS_BYTES;

static void hexdump(const void *data, size_t size) {
  char ascii[17];
  size_t i;
  size_t j;
  ascii[16] = '\0';
  for (i = 0; i < size; ++i) {
    printf("%02X ", ((unsigned char *)data)[i]);
    if (((unsigned char *)data)[i] >= ' ' &&
        ((unsigned char *)data)[i] <= '~') {
      ascii[i % 16] = ((unsigned char *)data)[i];
    }
    else {
      ascii[i % 16] = '.';
    }
    if ((i + 1) % 8 == 0 || i + 1 == size) {
      printf(" ");
      if ((i + 1) % 16 == 0) {
        printf("|  %s \n", ascii);
      } 
      else if (i + 1 == size) {
        ascii[(i + 1) % 16] = '\0';
        if ((i + 1) % 16 <= 8) {
          printf(" ");
        }
        for (j = (i + 1) % 16; j < 16; ++j) {
          printf("   ");
        }
        printf("|  %s \n", ascii);
      }
    }
  }
}

static void get_presigned_urls(struct S3Client *s3) {
  int rc = 0;
  const char *url = daapi_s3cli_get_presigned_url_for_get(s3, S3_BUCKET,
                                                          OBJECT_FOR_GET, 3600);
  if (url) {
    printf("Presigned URL for GetObject = %s\n", url);
    rc = daapi_free_string((char *)url);
    if (rc < 0) {
      printf("Failed to free a string (%d)", __LINE__);
    }
  }

  url = daapi_s3cli_get_presigned_url_for_put(s3, S3_BUCKET, OBJECT_FOR_PUT,
                                              3600);
  if (url) {
    printf("Presigned URL for PutObject = %s\n", url);
    rc = daapi_free_string((char *)url);
    if (rc < 0) {
      printf("Failed to free a string (%d)", __LINE__);
    }
  }
}

static void put_objects(struct S3Client *s3) {
  int rc = daapi_s3cli_put_object_as_string(s3, S3_BUCKET, OBJECT_AS_STR,
                                        "Hello, World.");
  if (rc == 0) {
    printf("PutObject(%s) was succeeded.\n", OBJECT_AS_STR);
  }
  else {
    printf("PutObject(%s) was failed.\n", OBJECT_AS_STR);
  }

  uint8_t buffer[] = {'D', 'E', 'A', 'D', 'B', 'E', 'E', 'F', '\0'};
  rc = daapi_s3cli_put_object_as_bytes(s3, S3_BUCKET, OBJECT_AS_BYTES, buffer,
                                       sizeof(buffer));
  if (rc == 0) {
    printf("PutObject(%s) was succeeded.\n", OBJECT_AS_BYTES);
  }
  else {
    printf("PutObject(%s) was failed.\n", OBJECT_AS_BYTES);
  }
}

static void get_objects(struct S3Client *s3) {
  int rc = 0;
  const char *content =
      daapi_s3cli_get_object_as_string(s3, S3_BUCKET, OBJECT_AS_STR);
  if (content) {
    printf("GetObject(%s) %p, %s\n", OBJECT_AS_STR, content, content);
    rc = daapi_free_string((char *)content);
    if (rc < 0) {
      printf("Failed to free a string (%d)", __LINE__);
    }
  }
  else {
    printf("GetObject(%s) was failed.\n", OBJECT_AS_STR);
  }

  struct Buffer *buf =
      daapi_s3cli_get_object_as_bytes(s3, S3_BUCKET, OBJECT_AS_BYTES);
  if (buf) {
    hexdump(buf->data, buf->size);
    daapi_free_buffer(buf);
  }
  else {
    printf("GetObject was failed.\n");
  }
}

static void delete_objects(struct S3Client *s3) {
  int rc = 0;
  struct S3ObjectList *objects = daapi_s3cli_list_objects(s3, S3_BUCKET);
  if (objects) {
    for (int i = 0; i < objects->length; i++) {
      char *key = objects->objects[i].key;
      rc = daapi_s3cli_delete_object(s3, S3_BUCKET, key);
      if (rc == 0) {
        printf("%s was deleted.\n", key);
      }
      else {
        printf("Failed to delete %s.\n", key);
      }
    }
    rc = daapi_free_s3_object_list(objects);
    if (rc == 0) {
      printf("S3ObjectList(%p) was deleted.\n", objects);
    }
    else {
      printf("Failed to delete S3ObjectList(%p).\n", objects);
    }
  }
}

int main(int argc, char *argv[]) {

  int rc = 0;
  struct S3Client *s3 = daapi_s3cli_new(S3_ENDPOINT, AWS_ACCESS_KEY_ID,
                                        AWS_SECRET_ACCESS_KEY, S3_REGION);
  if (!s3) {
    printf("Failed to create S3Client. Exiting..\n");
    return -1;
  }

  get_presigned_urls(s3);

  put_objects(s3);

  get_objects(s3);

  delete_objects(s3);

  rc = daapi_free_s3client(s3);
  if (rc == 0) {
    printf("S3Client(%p) was deleted.\n", s3);
  }
  else {
    printf("Failed to delete S3Client(%p).\n", s3);
  }

  return 0;
}
