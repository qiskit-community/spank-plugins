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

"""Logger to generate the logs for users"""
import io
import logging
from direct_access_client.daa_sim.consts import DEFAULT_LOG_LEVEL


class UserLogger(logging.Logger):
    """Logger for users"""

    # log format
    FORMAT = "%(asctime)s %(levelname)-9s %(message)s"

    def __init__(self, name: str, level: str):
        """Construct UserLogger

        Args:
            name(str): logger name
            level(str): loglevel string such as "critical", "error", "warning" etc.

        Returns:
            UserLogger: instance
        """
        if level is None:
            level = DEFAULT_LOG_LEVEL
        super().__init__(name, getattr(logging, level.upper(), logging.WARNING))
        self._str_io = io.StringIO()
        handler = logging.StreamHandler(self._str_io)
        handler.setFormatter(logging.Formatter(self.FORMAT))
        self.addHandler(handler)

    def getvalue(self) -> str:
        """Return a str containing the entire contents of the buffer.
        Newlines are decoded as if by read(), although the stream position is not changed.

        Returns:
            str: a str containing the entire contents of the buffer.
        """
        return self._str_io.getvalue()
