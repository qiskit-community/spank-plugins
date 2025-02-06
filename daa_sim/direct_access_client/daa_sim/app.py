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

"""daa_sim main()"""
import argparse
import os
import json
import secrets
import uvicorn
import yaml
from direct_access_client.daa_sim.consts import (
    SECRET_KEY_FILE,
    DEFAULT_BIND_ADDR,
    DEFAULT_PORT,
    DEFAULT_FASTAPI_WORKERS,
)


def main():
    """Application entrypoint"""

    parser = argparse.ArgumentParser(
        prog="daa_sim",
        description="Direct Access API Simulator",
    )
    parser.add_argument("config", help="Configuration file(.yaml)")
    parser.add_argument(
        "--log-config",
        help="Configuration file(.yaml) for python standard logging mechanism.",
        default=os.path.join(os.path.dirname(__file__), "logging.yaml"),
    )
    args, _ = parser.parse_known_args()

    with open(args.config, encoding="utf-8") as config_file:
        config = yaml.safe_load(config_file)

    with open(args.log_config, encoding="utf-8") as logging_yaml:
        log_config = yaml.safe_load(logging_yaml)

    # Store configurations as environment variables to shared with spawned processes.
    os.environ["DAASIM_CONFIG"] = json.dumps(config)
    os.environ["DAASIM_LOG_CONFIG"] = json.dumps(log_config)

    # Generates the secret key used to encode and decode the JWT token
    # for authentication and saves it to a file.
    # This file is referenced by each FastAPI worker process and used to
    # ensure that all workers use the same key.
    if os.path.exists(SECRET_KEY_FILE) is False:
        with open(SECRET_KEY_FILE, "w", encoding="utf-8") as key_file:
            key_dict = {"jwt_token_key": secrets.token_hex(32)}
            json.dump(key_dict, key_file)

    # Start FastAPI application
    try:
        uvicorn.run(
            "direct_access_client.daa_sim.api:app",
            host=config.get("host", DEFAULT_BIND_ADDR),
            port=config.get("port", DEFAULT_PORT),
            log_config=args.log_config,
            workers=config.get("api_workers", DEFAULT_FASTAPI_WORKERS),
        )
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    main()
