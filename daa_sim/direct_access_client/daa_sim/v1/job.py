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

# pylint: disable=line-too-long

"""Job API Mock"""
import json
import logging.config
from typing import Annotated
from fastapi import APIRouter, HTTPException, Body, Response, Request, Depends
from fastapi.security import HTTPAuthorizationCredentials
from starlette import status
from direct_access_client.daa_sim.auth import (
    internal_shared_key_security,
    access_token_security,
    verify_token,
)

from direct_access_client.daa_sim.v1.models import ErrorResponse, Job, JobsResponse
from direct_access_client.daa_sim.errors import (
    InvalidInputError,
    ExecutionLanesLimitReachedError,
)

logger = logging.getLogger("api")

router = APIRouter()


@router.post(
    "/v1/jobs",
    summary="Run a job",
    description="Invoke a Qiskit Runtime primitive.",
    response_model_exclude_none=True,
    responses={
        400: {"model": ErrorResponse},
        401: {"model": ErrorResponse},
        409: {"model": ErrorResponse},
        429: {"model": ErrorResponse},
    },
    status_code=204,
    tags=["Jobs"],
)
def run_job(
    request: Request,
    job: Annotated[
        Job,
        Body(
            openapi_examples={
                "Using IBM Cloud Object Storage": {
                    "value": {
                        "id": "6e32f594-189e-4bc5-89a2-3c21e1c7e75a",
                        "backend": "ibm_bromont",
                        "timeout_secs": 10000,
                        "storage": {
                            "input": {
                                "type": "ibmcloud_cos",
                                "region": "us-east",
                                "region_type": "regional",
                                "bucket_crn": "crn:v1:bluemix:public:cloud-object-storage:global:a/abc123:abc123:bucket:my-bucket",
                                "object_name": "params:6e32f594-189e-4bc5-89a2-3c21e1c7e75a",
                            },
                            "results": {
                                "type": "ibmcloud_cos",
                                "region": "us-east",
                                "region_type": "regional",
                                "bucket_crn": "crn:v1:bluemix:public:cloud-object-storage:global:a/abc123:abc123:bucket:my-bucket",
                                "object_name": "results:6e32f594-189e-4bc5-89a2-3c21e1c7e75a",
                            },
                            "logs": {
                                "type": "ibmcloud_cos",
                                "region": "us-east",
                                "region_type": "regional",
                                "bucket_crn": "crn:v1:bluemix:public:cloud-object-storage:global:a/abc123:abc123:bucket:my-bucket",
                                "object_name": "logs:6e32f594-189e-4bc5-89a2-3c21e1c7e75a",
                            },
                        },
                        "program_id": "sampler",
                    }
                },
                "Using S3 compatible storage": {
                    "value": {
                        "id": "6e32f594-189e-4bc5-89a2-3c21e1c7e75b",
                        "backend": "ibm_bromont",
                        "timeout_secs": 10000,
                        "storage": {
                            "input": {
                                "type": "s3_compatible",
                                "presigned_url": "<presigned_get_url>",
                            },
                            "results": {
                                "type": "s3_compatible",
                                "presigned_url": "<presigned_put_url>",
                            },
                            "logs": {
                                "type": "s3_compatible",
                                "presigned_url": "<presigned_put_url>",
                            },
                        },
                        "program_id": "estimator",
                    }
                },
            }
        ),
    ],
    access_token: Annotated[
        HTTPAuthorizationCredentials, Depends(access_token_security)
    ],
    internal_shared_key: Annotated[
        HTTPAuthorizationCredentials, Depends(internal_shared_key_security)
    ],
) -> Response:
    """Run a job"""
    verify_token(request, access_token, internal_shared_key)
    service = request.app.daa_service
    if job.backend not in service.available_backends():
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=f"backend: {job.backend} is not available.",
        )

    job_json = job.model_dump(exclude_none=True)
    logger.info(json.dumps(job_json, indent=2))

    job_list = service.get_jobs()["jobs"]
    num_lanes = 0
    for each_job in job_list:
        if each_job["backend"] == job_json["backend"]:
            num_lanes += 1

    if num_lanes >= request.app.daa_max_execution_lanes:
        raise ExecutionLanesLimitReachedError(job.backend)

    result = service.execute_job(job_json)
    if result["ok"] is True:
        return Response(status_code=status.HTTP_204_NO_CONTENT)

    raise InvalidInputError("Invalid program_id specified.", job_json["program_id"])


@router.get(
    "/v1/jobs",
    summary="Get jobs",
    description="Returns jobs submitted by current client in ascending order of created time by default.",
    responses={
        200: {"model": JobsResponse},
        401: {"model": ErrorResponse},
    },
    response_model_exclude_none=True,
    tags=["Jobs"],
)
def get_jobs(
    request: Request,
    access_token: Annotated[
        HTTPAuthorizationCredentials, Depends(access_token_security)
    ],
    internal_shared_key: Annotated[
        HTTPAuthorizationCredentials, Depends(internal_shared_key_security)
    ],
) -> JobsResponse:
    """Get jobs

    Args:
        request(Request): incoming HTTP Request
        access_token(HTTPAuthorizationCredentials): access token credentials
        internal_shared_key(HTTPAuthorizationCredentials): internal shared key credentials

    Returns
        JobsResponse: a list of job details
    """
    verify_token(request, access_token, internal_shared_key)
    service = request.app.daa_service
    return service.get_jobs()


@router.delete(
    "/v1/jobs/{job_id}",
    summary="Delete a job",
    description="Deletes a job if it has terminated.",
    response_model_exclude_none=True,
    responses={
        401: {"model": ErrorResponse},
        404: {"model": ErrorResponse},
        409: {"model": ErrorResponse},
    },
    status_code=204,
    tags=["Jobs"],
)
def delete_job(
    request: Request,
    job_id: str,
    access_token: Annotated[
        HTTPAuthorizationCredentials, Depends(access_token_security)
    ],
    internal_shared_key: Annotated[
        HTTPAuthorizationCredentials, Depends(internal_shared_key_security)
    ],
) -> Response:
    """Deletes a job if it has terminated.

    Args:
        request(Request): incoming HTTP request
        job_id(str): job id
        access_token(HTTPAuthorizationCredentials): access token credentials
        internal_shared_key(HTTPAuthorizationCredentials): internal shared key credentials

    """
    verify_token(request, access_token, internal_shared_key)
    service = request.app.daa_service
    service.delete_job(job_id)
    return Response(status_code=status.HTTP_204_NO_CONTENT)


@router.post(
    "/v1/jobs/{job_id}/cancel",
    summary="Cancel a job",
    description="Cancels a job if it has not yet terminated.",
    response_model_exclude_none=True,
    responses={
        401: {"model": ErrorResponse},
        404: {"model": ErrorResponse},
        409: {"model": ErrorResponse},
        500: {"model": ErrorResponse},
    },
    status_code=204,
    tags=["Jobs"],
)
def cancel_job(
    request: Request,
    job_id: str,
    access_token: Annotated[
        HTTPAuthorizationCredentials, Depends(access_token_security)
    ],
    internal_shared_key: Annotated[
        HTTPAuthorizationCredentials, Depends(internal_shared_key_security)
    ],
) -> Response:
    """Cancels a job if it has not yet terminated.

    Args:
        request(Request): incoming HTTP request
        job_id(str): job id
        access_token(HTTPAuthorizationCredentials): access token credentials
        internal_shared_key(HTTPAuthorizationCredentials): internal shared key credentials

    """
    verify_token(request, access_token, internal_shared_key)
    service = request.app.daa_service
    service.cancel_job(job_id)
    return Response(status_code=status.HTTP_204_NO_CONTENT)
