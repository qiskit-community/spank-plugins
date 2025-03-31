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

"""EstimatorV2 Primitive implementation with IBM Direct Access QRMI"""
from qrmi import IBMDirectAccess
from .base_estimator import QRMIBaseEstimatorV2


class IBMDirectAccessEstimatorV2(QRMIBaseEstimatorV2):
    """EstimatorV2 for IBMDirectAccess QRMI"""

    def __init__(
        self,
        *,
        options: dict | None = None,
    ) -> None:
        super().__init__(IBMDirectAccess(), options=options)
