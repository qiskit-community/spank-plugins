# This code is part of Qiskit.
#
# (C) Copyright 2025 IBM. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""QRMI Service"""

import os
from logging import getLogger
from typing import List, Union

from qrmi import IBMDirectAccess, IBMQiskitRuntimeService, PasqalCloud

logger = getLogger("qrmi")


class QRMIService:
    """Class for interacting with the QRMI resources"""

    def __init__(self):
        qpus = os.environ["SLURM_JOB_QPU_RESOURCES"]
        logger.debug("qpus: %s", qpus)
        qpus = qpus.split(",")
        qpu_types = os.environ["SLURM_JOB_QPU_TYPES"]
        logger.debug("qpu types: %s", qpu_types)
        qpu_types = qpu_types.split(",")

        if len(qpus) != len(qpu_types):
            raise ValueError("Inconsistent specifications of QPU resources and types")

        self._qrmi_resources = {}
        for i, qpu in enumerate(qpus):
            qpu = qpu.strip()
            qrmi = None
            if qpu_types[i] == "direct-access":
                qrmi = IBMDirectAccess(qpu)
            elif qpu_types[i] == "qiskit-runtime-service":
                qrmi = IBMQiskitRuntimeService(qpu)
            elif qpu_types[i] == "pasqal-cloud":
              qrmi = PasqalCloud(qpu)
            else:
                logger.warning(
                    "Unsupported resource type: %s specified for %s", qpu_types[i], qpu
                )

            if qrmi.is_accessible() is True:
                self._qrmi_resources[qpu] = qrmi
            else:
                logger.debug("%s is not accessible now. ignored.", qpu)

    def resources(self) -> List[Union[IBMDirectAccess, IBMQiskitRuntimeService, PasqalCloud]]:
        """Return all accessible QRMI resources.

        Returns:
            List[Union[IBMDirectAccess, IBMQiskitRuntimeService, PasqalCloud]]: QRMI resources
        """
        return list(self._qrmi_resources.values())

    def resource(
        self, resource_id: str
    ) -> Union[IBMDirectAccess, IBMQiskitRuntimeService, PasqalCloud]:
        """Return a single backend matching the specified resource identifier.

        Args:
            resource_id: A resource identifier, i.e. backend name for IBM Quantum.

        Returns:
            Union[IBMDirectAccess, IBMQiskitRuntimeService, PasqalCloud]: QRMI resource if found, otherwise None.
        """
        return self._qrmi_resources.get(resource_id)
