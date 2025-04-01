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

"""Constant definitions

This file defines constants used by the python modules in daa_sim.
"""

# Direct Access API service version, consistent with the latest deployment
SERVICE_VERSION = "0.7.0"

# Default max execution lanes
# This default value can be overridden by $.max_execution_lanes in the config.yaml
DEFAULT_MAX_EXECUTION_LANES = 5

# Access token type, used by Direct Access API
ACCESS_TOKEN_TYPE = "Bearer"

# Access token lifetime. By default, access tokens are good for 1 hour.
# This default value can be overwritten by $.auth.access_token.ttl in the config.yaml
DEFAULT_ACCESS_TOKEN_TTL = 3600

# JWT Message authentication code algorithm
JWT_SIGNING_ALGO = "HS256"

# JWT subject
JWT_SUBJECT = "direct-access-api-access-token"

# Secret key filename
SECRET_KEY_FILE = ".secret_keys"

# API endpoint - Default bind address
DEFAULT_BIND_ADDR = "0.0.0.0"

# API endpoint - Default port
DEFAULT_PORT = 8290

# FastAPI workers
DEFAULT_FASTAPI_WORKERS = 1

# Default job metadata directory
DEFAULT_JOBS_DIR = ".jobs"

# Default log level
DEFAULT_LOG_LEVEL = "warning"

# job metadata directory for pytest
PYTEST_JOBS_DIR = ".pytest_jobs"
