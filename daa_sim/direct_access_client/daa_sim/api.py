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
"""DAAPI Mock"""
import json
import os
from contextlib import asynccontextmanager
from fastapi import FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from fastapi.exceptions import RequestValidationError
from starlette import status
from pydantic import BaseModel, Field

from direct_access_client.daa_sim.consts import (
    SERVICE_VERSION,
    DEFAULT_MAX_EXECUTION_LANES,
    DEFAULT_ACCESS_TOKEN_TTL,
)
from direct_access_client.daa_sim.daa_service import DAAService
from direct_access_client.daa_sim.v1 import authentication, backend, job
from direct_access_client.daa_sim.errors import (
    BackendNotFoundError,
    JobNotFoundError,
    DuplicateJobError,
    InvalidInputError,
    ServiceNotAvailableError,
    NotAuthorizedError,
    InvalidCredentialsError,
    JobNotCancellableError,
    UnableToDeleteJobInNonTerminalStateError,
    ExecutionLanesLimitReachedError,
)
from direct_access_client.daa_sim.v1.models import Error, ErrorResponse
from direct_access_client.daa_sim.consts import SECRET_KEY_FILE


@asynccontextmanager
async def app_lifespan(fastapi_app: FastAPI):
    """startup and shutdown logic using the lifespan parameter of the FastAPI app"""
    _create_fastapi(fastapi_app)
    yield
    # add shutdown process here
    fastapi_app.daa_service.close()


app = FastAPI(
    title="IBM Quantum Qiskit Runtime Direct Access API",
    version="1.0.0-beta",
    description="These APIs are intended to be used by scheduler applications to schedule jobs on a quantum device via Qiskit Runtime primitives.",
    license_info={"name": "IBM"},
    terms_of_service="https://www.ibm.com/support/customer/csol/terms/?id=i126-9425",
    lifespan=app_lifespan,
)
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=False,
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.exception_handler(BackendNotFoundError)
async def _handle_backend_not_found_error(_: Request, exc: BackendNotFoundError):
    body = ErrorResponse(
        title="validation failed",
        errors=[exc.dict()],
        trace="",
        status_code=status.HTTP_404_NOT_FOUND,
    )
    return JSONResponse(status_code=status.HTTP_404_NOT_FOUND, content=body.dict())


@app.exception_handler(JobNotFoundError)
async def _handle_job_not_found_error(req: Request, exc: JobNotFoundError):
    body = ErrorResponse(
        title="validation failed",
        errors=[exc.dict(req.app.daa_includes_options)],
        trace="",
        status_code=status.HTTP_404_NOT_FOUND,
    )
    return JSONResponse(status_code=status.HTTP_404_NOT_FOUND, content=body.dict())


@app.exception_handler(DuplicateJobError)
async def _handle_duplicated_job_error(req: Request, exc: DuplicateJobError):
    body = ErrorResponse(
        title="validation failed",
        errors=[exc.dict(req.app.daa_includes_options)],
        trace="",
        status_code=status.HTTP_409_CONFLICT,
    )
    return JSONResponse(status_code=status.HTTP_409_CONFLICT, content=body.dict())


@app.exception_handler(JobNotCancellableError)
async def _handle_job_not_cancellable_error(req: Request, exc: JobNotCancellableError):
    body = ErrorResponse(
        title="validation failed",
        errors=[exc.dict(req.app.daa_includes_options)],
        trace="",
        status_code=status.HTTP_409_CONFLICT,
    )
    return JSONResponse(status_code=status.HTTP_409_CONFLICT, content=body.dict())


@app.exception_handler(UnableToDeleteJobInNonTerminalStateError)
async def _handle_unable_to_delete_error(
    req: Request, exc: UnableToDeleteJobInNonTerminalStateError
):
    body = ErrorResponse(
        title="validation failed",
        errors=[exc.dict(req.app.daa_includes_options)],
        trace="",
        status_code=status.HTTP_409_CONFLICT,
    )
    return JSONResponse(status_code=status.HTTP_409_CONFLICT, content=body.dict())


@app.exception_handler(ServiceNotAvailableError)
async def _handle_service_not_available_error(
    req: Request, exc: ServiceNotAvailableError
):
    body = ErrorResponse(
        title="service inactive",
        errors=[exc.dict(req.app.daa_includes_options)],
        trace="",
        status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
    )
    return JSONResponse(
        status_code=status.HTTP_503_SERVICE_UNAVAILABLE, content=body.dict()
    )


@app.exception_handler(InvalidInputError)
async def _handle_invalid_input_error(req: Request, exc: InvalidInputError):
    body = ErrorResponse(
        title="validation failed",
        errors=[exc.dict(req.app.daa_includes_options)],
        trace="",
        status_code=status.HTTP_400_BAD_REQUEST,
    )
    return JSONResponse(status_code=status.HTTP_400_BAD_REQUEST, content=body.dict())


@app.exception_handler(NotAuthorizedError)
async def _handle_unauthorized_error(req: Request, exc: NotAuthorizedError):
    body = ErrorResponse(
        title="authentication failed",
        errors=[exc.dict(req.app.daa_includes_options)],
        trace="",
        status_code=status.HTTP_401_UNAUTHORIZED,
    )
    return JSONResponse(status_code=status.HTTP_401_UNAUTHORIZED, content=body.dict())


@app.exception_handler(InvalidCredentialsError)
async def _handle_invalid_credentials_error(req: Request, exc: InvalidCredentialsError):
    body = ErrorResponse(
        title="Invalid credentials.",
        errors=[exc.dict(req.app.daa_includes_options)],
        trace="",
        status_code=status.HTTP_401_UNAUTHORIZED,
    )
    return JSONResponse(
        status_code=status.HTTP_401_UNAUTHORIZED, content=body.dict(exclude_none=True)
    )


@app.exception_handler(ExecutionLanesLimitReachedError)
async def _handle_execution_lanes_limit_reached_error(
    req: Request, exc: ExecutionLanesLimitReachedError
):
    body = ErrorResponse(
        title=exc.dict()["message"],
        errors=[exc.dict(req.app.daa_includes_options)],
        trace="",
        status_code=status.HTTP_429_TOO_MANY_REQUESTS,
    )
    return JSONResponse(
        status_code=status.HTTP_429_TOO_MANY_REQUESTS,
        content=body.dict(exclude_none=True),
    )


# tentative - valid until pulse defaults is implemented
@app.exception_handler(NotImplementedError)
async def _handle_not_implemented_errors(_: Request, exc: NotImplementedError):
    error = Error(
        code="",
        location="api",
        message=str(exc),
        more_info="https://cloud.ibm.com/apidocs/quantum-computing#error-handling",
        solution="",
        value="",
    )
    body = ErrorResponse(
        title="server error",
        errors=[error],
        trace="",
        status_code=status.HTTP_501_NOT_IMPLEMENTED,
    )
    return JSONResponse(
        status_code=status.HTTP_501_NOT_IMPLEMENTED, content=body.dict()
    )


# FastAPI uses 422 as invalid request. Need to override to fit to DA API spec.
@app.exception_handler(RequestValidationError)
async def _handle_request_validation_error(_: Request, exc: RequestValidationError):
    errors = []
    for error in exc.errors():
        errors.append(
            Error(
                code=error["type"],
                location=str(error["loc"]),
                message=error["msg"],
                more_info="https://cloud.ibm.com/apidocs/quantum-computing#error-handling",
                solution="The input parameters in the request body are either incomplete or in the wrong format. Be sure to include all required parameters in your request.",
                value="",
            )
        )

    body = ErrorResponse(
        title="validation failed",
        errors=errors,
        trace="",
        status_code=status.HTTP_400_BAD_REQUEST,
    )
    return JSONResponse(status_code=status.HTTP_400_BAD_REQUEST, content=body.dict())


class Version(BaseModel):
    """Version response"""

    version: str = Field(
        description="Current version of the service",
    )


app.include_router(authentication.router)
app.include_router(backend.router)
app.include_router(job.router)


@app.get(
    "/version",
    summary="Get the current version of the service",
    description="Get the current version of the service",
    tags=["Version"],
)
def get_version() -> Version:
    """Get the current version of the service"""
    return Version(version=SERVICE_VERSION).dict()


def _create_fastapi(fastapi_app: FastAPI):

    # retrieve configurations via environment variables, set by app.py
    config = json.loads(os.environ.get("DAASIM_CONFIG"))

    fastapi_app.daa_config = config
    auth_config = config.get("auth", {})
    fastapi_app.daa_auth_enabled = auth_config.get("enabled", True)
    fastapi_app.daa_basicauth_credentials = auth_config.get(
        "token_endpoint_credentials", {}
    )
    fastapi_app.daa_iam_apikeys = auth_config.get("iam_apikeys", {})
    fastapi_app.daa_shared_tokens = auth_config.get("shared_tokens", {})
    fastapi_app.daa_access_token_ttl = auth_config.get("access_token", {}).get(
        "ttl", DEFAULT_ACCESS_TOKEN_TTL
    )

    fastapi_app.daa_includes_options = config.get("include_optional_values", False)
    fastapi_app.daa_max_execution_lanes = config.get(
        "max_execution_lanes", DEFAULT_MAX_EXECUTION_LANES
    )
    aer_options = config.get("aer_options", {})
    fastapi_app.daa_service = DAAService(
        include_opt_fields=fastapi_app.daa_includes_options,
        aer_options=aer_options,
        multiprocess=True,
        backends=config.get("backends"),
    )
    with open(SECRET_KEY_FILE, encoding="utf-8") as key_file:
        key_dict = json.load(key_file)
        fastapi_app.daa_secret_key = key_dict["jwt_token_key"]
