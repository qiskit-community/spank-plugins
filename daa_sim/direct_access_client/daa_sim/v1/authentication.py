# -*- coding: utf-8 -*-

# (C) Copyright 2024, 2025 IBM. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

# pylint: disable=line-too-long

"""Authentication API Mock"""
import datetime as dt
import logging.config
from typing import Annotated
import urllib
import jwt
from fastapi import APIRouter, Request, Depends
from fastapi.security import (
    HTTPBasic,
    HTTPBasicCredentials,
)
from direct_access_client.daa_sim.v1.models import TokenResponse, ErrorResponse
from direct_access_client.daa_sim.errors import (
    InvalidCredentialsError,
    NotAuthorizedError,
    IAMAPIKeyNotFoundError,
    IAMPropertyMissingOrEmptyError,
)
from direct_access_client.daa_sim.consts import (
    ACCESS_TOKEN_TYPE,
    JWT_SIGNING_ALGO,
    JWT_SUBJECT,
)

logger = logging.getLogger("api")


basicauth_security = HTTPBasic(
    auto_error=False,
    scheme_name="AppIDBearerAuth",
    description="(Deprecated) Use IBM Cloud App ID to generate access_token. Format: bearer {access_token}",
)

router = APIRouter()


@router.post(
    "/v1/token",
    summary="Get access token",
    description="Endpoint to request a short lived access token by passing provided client ID and secret in order to access other endpoints.",
    response_model_exclude_none=True,
    responses={
        200: {"model": TokenResponse},
        401: {"model": ErrorResponse},
    },
    tags=["Authentication"],
)
def get_access_token(
    request: Request,
    credentials: Annotated[HTTPBasicCredentials, Depends(basicauth_security)],
) -> TokenResponse:
    """Get access token

    Args:
        request(Request): Incoming HTTP request
        credentials(HTTPBasicCredentials): Basic authentication credential

    Returns:
        TokenResponse: Access token
    """
    if credentials is not None:
        logger.warning(
            "IBM App ID authentication is deprecated. Use IAM API key authentication."
        )
        basicauth_credentials = request.app.daa_basicauth_credentials
        if basicauth_credentials.get(credentials.username) != credentials.password:
            raise InvalidCredentialsError()
    else:
        raise NotAuthorizedError()

    ttl = request.app.daa_access_token_ttl
    now = dt.datetime.utcnow()
    payload = {
        "sub": JWT_SUBJECT,
        "iat": now,
        "exp": now + dt.timedelta(seconds=ttl),
    }

    return TokenResponse(
        access_token=jwt.encode(
            payload, request.app.daa_secret_key, algorithm=JWT_SIGNING_ALGO
        ),
        expires_in=ttl,
        token_type=ACCESS_TOKEN_TYPE,
    )


@router.post(
    "/identity/token",
    summary="Get IAM access token for simulation",
    description="Endpoint to request a short lived access token by passing provided client ID and secret in order to access other endpoints. This can be used if IAM is not available.",
    response_model_exclude_none=True,
    responses={
        200: {"model": TokenResponse},
        401: {"model": ErrorResponse},
    },
    tags=["Authentication"],
)
async def get_iam_access_token(
    request: Request,
) -> TokenResponse:
    """Get access token

    Args:
        request(Request): Incoming HTTP request

    Returns:
        TokenResponse: Access token
    """
    body = await request.body()
    body = urllib.parse.unquote(body.decode("utf-8"))

    apikey = None
    grant_type = None
    params = body.split("&")
    for param in params:
        if param.startswith("apikey="):
            apikey = param[len("apikey=") :]
        elif param.startswith("grant_type="):
            grant_type = param[len("grant_type=") :]

    if grant_type is None or grant_type != "urn:ibm:params:oauth:grant-type:apikey":
        raise IAMPropertyMissingOrEmptyError()

    if apikey is not None:
        apikeys = request.app.daa_iam_apikeys
        if apikey not in apikeys:
            raise IAMAPIKeyNotFoundError()
    else:
        raise IAMPropertyMissingOrEmptyError()

    ttl = request.app.daa_access_token_ttl
    now = dt.datetime.utcnow()
    payload = {
        "sub": JWT_SUBJECT,
        "iat": now,
        "exp": now + dt.timedelta(seconds=ttl),
    }

    return TokenResponse(
        access_token=jwt.encode(
            payload, request.app.daa_secret_key, algorithm=JWT_SIGNING_ALGO
        ),
        expires_in=ttl,
        token_type=ACCESS_TOKEN_TYPE,
        expiration=int(payload["exp"].timestamp()),
    )
