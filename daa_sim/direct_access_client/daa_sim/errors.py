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

"""Exception classes"""

import json


class DAAServiceError(Exception):
    """An exception raised by DAA service"""

    def __init__(
        self,
        *,
        message: str,
        location: str,
        solution: str,
        value: str,
        code: str,
        more_info: str = "https://cloud.ibm.com/apidocs/quantum-computing#error-handling",
    ) -> None:
        super().__init__(message)
        self._json = {
            "message": message,
            "location": location,
            "solution": solution,
            "value": value,
            "code": code,
            "more_info": more_info,
        }

    def __str__(self):
        return json.dumps(self._json)

    def dict(self, incl_options: bool = True) -> dict:
        """Returns JSON representation of this error

        Args:
            incl_options(bool): False if option fields not included.

        Returns:
            dict: JSON representation of this error
        """
        resp = self._json.copy()
        if incl_options is False:
            del resp["location"]
            del resp["solution"]
            del resp["value"]

        return resp


class BackendNotFoundError(DAAServiceError):
    """An exception raised if the specified backend is not found"""

    def __init__(
        self,
        backend_name: str,
    ) -> None:
        super().__init__(
            message=f"Backend {backend_name} not found.",
            code="1216",
            solution="Use a different backend.",
            location="",
            value=backend_name,
        )


class JobNotFoundError(DAAServiceError):
    """An exception raised if the specified job id is not found"""

    def __init__(
        self,
        job_id: str,
    ) -> None:
        super().__init__(
            message=f"Job not found. Job ID: {job_id}.",
            code="1291",
            solution="Verify the job ID is correct, and that you have the correct access permissions.",
            location="",
            value=job_id,
        )


class DuplicateJobError(DAAServiceError):
    """An exception raised if a duplicate job with same job id
    was submitted.
    """

    def __init__(
        self,
        job_id: str,
    ) -> None:
        super().__init__(
            message=f"Job with duplicate id already exists. Job ID: {job_id}",
            code="1231",
            solution="Check that the provided id is unique or delete the job and try again.",
            location="",
            value=job_id,
        )


class JobNotCancellableError(DAAServiceError):
    """An exception raised if a duplicate job with same job id
    was submitted.
    """

    def __init__(
        self,
        job_id: str,
    ) -> None:
        super().__init__(
            message=f"Job is not cancellable. Job ID: {job_id}.",
            code="1306",
            solution="Try again or contact support.",
            location="",
            value=job_id,
        )


class UnableToDeleteJobInNonTerminalStateError(DAAServiceError):
    """An exception raised if a duplicate job with same job id
    was submitted.
    """

    def __init__(
        self,
        job_id: str,
    ) -> None:
        super().__init__(
            message=f"Deleting a job in a non-terminal state is not possible. Job ID: {job_id}.",
            code="1337",
            solution="Postpone job deletion until job is in a terminal state.",
            location="",
            value=job_id,
        )


class InvalidInputError(DAAServiceError):
    """An exception raised if an input is invalid."""

    def __init__(
        self,
        message: str,
        value: str,
    ) -> None:
        super().__init__(
            message=message,
            code="1337",
            solution="Check the JSON schema.",
            location="",
            value=value,
        )


class ServiceNotAvailableError(DAAServiceError):
    """An exception raised if service is inactive."""

    def __init__(
        self,
    ) -> None:
        super().__init__(
            message="service inactive",
            code="",
            solution="Try again or contact support.",
            location="",
            value="",
        )


class NotAuthorizedError(DAAServiceError):
    """An exception raised if authentication is failed."""

    def __init__(
        self,
    ) -> None:
        super().__init__(
            message="You are not authorized to perform this action.",
            code="1200",
            solution="Try again or contact support.",
            location="authentication",
            value="",
        )


class InvalidCredentialsError(DAAServiceError):
    """An exception raised if the specified credentials are invalid."""

    def __init__(
        self,
    ) -> None:
        super().__init__(
            message="Invalid credentials.",
            code="1201",
            solution="Verify your credentials and try again.",
            location="header",
            value="",
        )


class ExecutionLanesLimitReachedError(DAAServiceError):
    """An exception raised if the maximum number of execution lanes has been reached."""

    def __init__(
        self,
        backend: str,
    ) -> None:
        super().__init__(
            message=f"The maximum number of execution lanes for backend {backend} has been reached.",
            code="1232",
            solution="Delete completed jobs and/or wait for other clients to delete jobs.",
            location="",
            value="",
        )
