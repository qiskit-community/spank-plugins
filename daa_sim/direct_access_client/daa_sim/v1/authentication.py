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

"""Authentication API Mock"""
import datetime as dt
import logging.config
from typing import Annotated
import jwt
from fastapi import APIRouter, Request, Depends
from fastapi.security.http import HTTPBase
from fastapi.security import (
    HTTPBasic,
    HTTPBasicCredentials,
    HTTPAuthorizationCredentials,
)
from direct_access_client.daa_sim.v1.models import TokenResponse, ErrorResponse
from direct_access_client.daa_sim.errors import (
    InvalidCredentialsError,
    NotAuthorizedError,
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
apikey_security = HTTPBase(
    scheme="apikey",
    scheme_name="IAMApiKeyAuth",
    description="Use the IAM service API key to generate access_token. Include it in the Authorization header as 'apikey {your IAM api key}'.",
    auto_error=False,
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
    apikey: Annotated[HTTPAuthorizationCredentials, Depends(apikey_security)],
) -> TokenResponse:
    """Get access token

    Args:
        request(Request): Incoming HTTP request
        credentials(HTTPBasicCredentials): Basic authentication credential
        apikey(HTTPAuthorizationCredentials): IAM API key

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
    elif apikey is not None:
        apikeys = request.app.daa_iam_apikeys
        if apikey.credentials not in apikeys:
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
        expiration=int(payload["exp"].timestamp()) if apikey is not None else None,
    )
