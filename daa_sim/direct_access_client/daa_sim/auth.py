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

"""Authentication related functions"""

# pylint: disable=line-too-long

import logging.config
import jwt
from jwt.exceptions import InvalidTokenError
from fastapi import Request
from fastapi.security.http import HTTPBase
from fastapi.security import (
    HTTPBearer,
    HTTPAuthorizationCredentials,
)

from direct_access_client.daa_sim.consts import (
    JWT_SIGNING_ALGO,
    JWT_SUBJECT,
)
from direct_access_client.daa_sim.errors import (
    NotAuthorizedError,
    InvalidCredentialsError,
)

_logger = logging.getLogger("api")


internal_shared_key_security = HTTPBase(
    scheme="apikey",
    scheme_name="InternalSharedKey",
    description="Format: apikey {client_id}:{shared_token}",
    auto_error=False,
)
access_token_security = HTTPBearer(
    scheme_name="BearerAuth",
    description="Use the access token obtained from the /v1/token endpoint to authenticate requests. Include it in the Authorization header as 'Bearer {access_token}'.",
    auto_error=False,
)


def verify_token(
    req: Request,
    access_token: HTTPAuthorizationCredentials,
    internal_shared_key: HTTPAuthorizationCredentials,
) -> bool:
    """Validate the given HTTP authorization credentials.

    Args:
        req(Request): incoming HTTP request
        access_token(HTTPAuthorizationCredentials): access token
        internal_shared_key(HTTPAuthorizationCredentials): internal shared key

    Returns:
        bool: True if OK, otherwise HTTPException will be raised.
    """
    if req.app.daa_auth_enabled is False:
        # skip token verification if disabled
        return True

    service_crn = req.headers.get("Service-CRN")
    if service_crn is None or len(service_crn) == 0:
        _logger.warning("Service-CRN header either missing or empty.")
    elif req.app.service_crn is not None and service_crn != req.app.service_crn:
        _logger.warning(
            "Service-CRN is not matched. expected(%s), actual(%s)",
            req.app.service_crn,
            service_crn,
        )

    token = req.headers.get("Authorization")
    if token is None:
        raise InvalidCredentialsError()

    if access_token is not None:
        try:
            payload = jwt.decode(
                access_token.credentials,
                req.app.daa_secret_key,
                algorithms=[JWT_SIGNING_ALGO],
            )
            if payload.get("sub") == JWT_SUBJECT:
                return True
        except InvalidTokenError as err:
            raise InvalidCredentialsError() from err

    elif internal_shared_key is not None:
        _logger.warning(
            "Shared token-based authentication is deprecated. Use IAM API Key authentication."
        )
        credential = internal_shared_key.credentials.split(":")
        if len(credential) != 2:
            raise InvalidCredentialsError()

        if req.app.daa_shared_tokens.get(credential[0]) == credential[1]:
            return True

    raise NotAuthorizedError()
