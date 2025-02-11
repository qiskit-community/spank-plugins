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

"""Backend API Mock"""
import logging.config
from typing import Union, Annotated
from fastapi import APIRouter, Request, Depends
from fastapi.security import HTTPAuthorizationCredentials
from direct_access_client.daa_sim.auth import (
    internal_shared_key_security,
    access_token_security,
    verify_token,
)

from direct_access_client.daa_sim.v1.models import (
    BackendsResponse,
    BackendResponse,
    ErrorResponse,
    BackendConfigurationResponse,
    PulseDefaultsResponse,
    EmptyPulseDefaultsResponse,
    BackendPropertiesResponse,
)

logger = logging.getLogger("api")

router = APIRouter()


@router.get(
    "/v1/backends",
    summary="Get list of backends",
    description="Returns a list of backends enabled for direct access.",
    responses={
        200: {"model": BackendsResponse},
        401: {"model": ErrorResponse},
    },
    response_model_exclude_none=True,
    tags=["Backends"],
)
def list_backends(
    request: Request,
    access_token: Annotated[
        HTTPAuthorizationCredentials, Depends(access_token_security)
    ],
    internal_shared_key: Annotated[
        HTTPAuthorizationCredentials, Depends(internal_shared_key_security)
    ],
) -> BackendsResponse:
    """Returns a list of backend device names.

    Args:
        request(Request): Incoming HTTP request
        access_token(HTTPAuthorizationCredentials): access token credentials
        internal_shared_key(HTTPAuthorizationCredentials): internal shared key credentials

    Returns:
        BackendsResponse: a list of backend device names.
    """
    verify_token(request, access_token, internal_shared_key)
    return request.app.daa_service.backends()


@router.get(
    "/v1/backends/{backend_name}",
    summary="Get backend details",
    description="Returns details of a backend.",
    responses={
        200: {"model": BackendResponse},
        401: {"model": ErrorResponse},
        404: {"model": ErrorResponse},
    },
    response_model_exclude_none=True,
    tags=["Backends"],
)
def get_backend_details(
    request: Request,
    backend_name: str,
    access_token: Annotated[
        HTTPAuthorizationCredentials, Depends(access_token_security)
    ],
    internal_shared_key: Annotated[
        HTTPAuthorizationCredentials, Depends(internal_shared_key_security)
    ],
) -> BackendResponse:
    """Returns details of a backend.

    Args:
        request(Request): Incoming HTTP request
        backend_name(str): backend name
        access_token(HTTPAuthorizationCredentials): access token credentials
        internal_shared_key(HTTPAuthorizationCredentials): internal shared key credentials

    Returns:
        BackendResponse: backend details
    """
    verify_token(request, access_token, internal_shared_key)
    service = request.app.daa_service
    return service.get_backend_details(backend_name)


@router.get(
    "/v1/backends/{backend_name}/configuration",
    summary="Get backend configuration",
    description="Returns configuration of a backend.",
    responses={
        200: {"model": BackendConfigurationResponse},
        401: {"model": ErrorResponse},
        404: {"model": ErrorResponse},
    },
    response_model_exclude_none=True,
    tags=["Backends"],
)
def get_backend_configuration(
    request: Request,
    backend_name: str,
    access_token: Annotated[
        HTTPAuthorizationCredentials, Depends(access_token_security)
    ],
    internal_shared_key: Annotated[
        HTTPAuthorizationCredentials, Depends(internal_shared_key_security)
    ],
) -> BackendConfigurationResponse:
    """Returns configuration of a backend.

    Args:
        request(Request): Incoming HTTP request
        backend_name(str): backend name
        access_token(HTTPAuthorizationCredentials): access token credentials
        internal_shared_key(HTTPAuthorizationCredentials): internal shared key credentials

    Returns:
        BackendConfigurationResponse: API response
    """
    verify_token(request, access_token, internal_shared_key)
    service = request.app.daa_service
    resp = service.get_backend_configuration(backend_name)
    # Workaround for fake_brisbane. Its revision value is integer, not string.
    if (revision := resp.get("processor_type", {}).get("revision")) is not None:
        resp["processor_type"]["revision"] = str(revision)
    return resp


@router.get(
    "/v1/backends/{backend_name}/properties",
    summary="Get backend properties",
    description="Returns properties of a backend.",
    responses={
        200: {"model": BackendPropertiesResponse},
        401: {"model": ErrorResponse},
        404: {"model": ErrorResponse},
    },
    response_model_exclude_none=True,
    tags=["Backends"],
)
def get_backend_properties(
    request: Request,
    backend_name: str,
    access_token: Annotated[
        HTTPAuthorizationCredentials, Depends(access_token_security)
    ],
    internal_shared_key: Annotated[
        HTTPAuthorizationCredentials, Depends(internal_shared_key_security)
    ],
) -> BackendPropertiesResponse:
    """Returns properties of a backend.

    Args:
        request(Request): Incoming HTTP request
        backend_name(str): backend name
        access_token(HTTPAuthorizationCredentials): access token credentials
        internal_shared_key(HTTPAuthorizationCredentials): internal shared key credentials

    Returns:
        BackendPropertiesResponse: API response
    """
    verify_token(request, access_token, internal_shared_key)
    service = request.app.daa_service
    return service.get_backend_properties(backend_name)


@router.get(
    "/v1/backends/{backend_name}/defaults",
    summary="Get backend pulse defaults",
    description="Returns pulse defaults of a backend.",
    responses={
        200: {"model": PulseDefaultsResponse},
        401: {"model": ErrorResponse},
        404: {"model": ErrorResponse},
    },
    response_model_exclude_none=True,
    tags=["Backends"],
)
def get_backend_pulse_defaults(
    request: Request,
    backend_name: str,
    access_token: Annotated[
        HTTPAuthorizationCredentials, Depends(access_token_security)
    ],
    internal_shared_key: Annotated[
        HTTPAuthorizationCredentials, Depends(internal_shared_key_security)
    ],
) -> Union[PulseDefaultsResponse, EmptyPulseDefaultsResponse]:
    """Returns pulse defaults of a backend.

    Args:
        request(Request): Incoming HTTP request
        backend_name(str): backend name
        access_token(HTTPAuthorizationCredentials): access token credentials
        internal_shared_key(HTTPAuthorizationCredentials): internal shared key credentials

    Returns:
        PulseDefaultsResponse: API response
    """
    verify_token(request, access_token, internal_shared_key)
    service = request.app.daa_service
    return service.get_backend_pulse_defaults(backend_name)
