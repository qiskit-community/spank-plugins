# -*- coding: utf-8 -*-

# (C) Copyright 2024 IBM. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""A tool to delete S3 objects in the specified bucket"""

import json
import argparse
import warnings
import boto3

warnings.filterwarnings("ignore")


def _yes_or_no(message: str) -> bool:
    while True:
        choice = input(message).lower()
        if choice in ["y", "yes"]:
            return True
        elif choice in ["n", "no"]:
            return False


def main():
    """main()"""
    parser = argparse.ArgumentParser(
        prog="gen_presigned_url.py",
        description="A tool to delete all S3 objects in the specified S3 bucket.",
    )
    parser.add_argument("s3_bucket_name", help="Bucket name.")
    parser.add_argument("s3_endpoint_url", help="S3 endpoint URL.")
    parser.add_argument("aws_access_key_id", help="AWS S3 access key id.")
    parser.add_argument("aws_secret_access_key", help="AWS S3 secret access key.")
    args = parser.parse_args()

    cos = boto3.client(
        "s3",
        aws_access_key_id=args.aws_access_key_id,
        aws_secret_access_key=args.aws_secret_access_key,
        endpoint_url=args.s3_endpoint_url,
    )
    objects = cos.list_objects_v2(Bucket=args.s3_bucket_name)

    keys = []
    if objects["KeyCount"] == 0:
        print(f"There are no objects in the bucket: {args.s3_bucket_name}. Exiting.")
        return

    for k in objects["Contents"]:
        keys.append({"Key": k["Key"]})

    print(f"Bucket: {args.s3_bucket_name} contains..")
    for k in keys:
        print(k["Key"])

    is_yes = _yes_or_no("Are you sure to delete all objects? [y/N]:")
    if is_yes is False:
        return

    def _split_list(arr):
        for idx in range(0, len(arr), 1000):
            yield arr[idx : idx + 1000]

    devide_list = list(_split_list(keys))

    for key_list in devide_list:
        resp = cos.delete_objects(
            Bucket=args.s3_bucket_name,
            Delete={
                "Objects": key_list,
            },
        )
        for deleted in resp["Deleted"]:
            print(json.dumps(deleted))

    print(len(keys), "objects deleted.")


if __name__ == "__main__":
    main()
