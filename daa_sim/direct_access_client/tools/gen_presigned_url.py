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

"""A tool to generate S3 presigned url for the specified operation"""

import argparse
from typing import Any
import warnings
import boto3

warnings.filterwarnings("ignore")


def get_s3_client(
    endpoint_url: str,
    aws_access_key_id: str,
    aws_secret_access_key: str,
) -> Any:
    """Returns S3 client

    Args:
        endpoint_url(str): S3 API endpoint
        aws_access_key_id(str): S3 access key
        aws_secret_access_key(str): S3 secret access key

    Returns:
        Any: S3 Client
    """
    return boto3.client(
        "s3",
        endpoint_url=endpoint_url,
        aws_access_key_id=aws_access_key_id,
        aws_secret_access_key=aws_secret_access_key,
    )


def main():
    """main()"""
    parser = argparse.ArgumentParser(
        prog="gen_presigned_url.py", description="A tool to generate presigned URL."
    )
    parser.add_argument("bucket", help="Bucket name.")
    parser.add_argument("object", help="Object name.")
    parser.add_argument("s3_endpoint_url", help="S3 endpoint URL.")
    parser.add_argument("aws_access_key_id", help="AWS S3 access key id.")
    parser.add_argument("aws_secret_access_key", help="AWS S3 secret access key.")

    parser.add_argument(
        "-m", "--method", default="GET", help="HTTP method to use (GET, PUT etc.)"
    )
    parser.add_argument(
        "-e",
        "--expiration",
        default=3600,
        type=int,
        help="Time in seconds for the presigned URL to remain valid.",
    )
    args = parser.parse_args()

    if args.method == "GET":
        # method for downloading a file
        client_method = "get_object"
    elif args.method == "PUT":
        # method for uploading a file
        client_method = "put_object"
    else:
        raise ValueError(f"Unsupported method: {args.method}.")

    s3 = get_s3_client(
        args.s3_endpoint_url, args.aws_access_key_id, args.aws_secret_access_key
    )
    presigned_url = s3.generate_presigned_url(
        ClientMethod=client_method,
        Params={"Bucket": args.bucket, "Key": args.object},
        ExpiresIn=args.expiration,
        HttpMethod=args.method,
    )

    print(
        f"presigned url = {presigned_url} for {args.method}, expiration = {args.expiration}"
    )


if __name__ == "__main__":
    main()
