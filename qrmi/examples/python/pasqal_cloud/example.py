# -*- coding: utf-8 -*-

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

"""An example of Pasqal Cloud QRMI python-bindings"""

import argparse
import json
from qrmi import PasqalCloud

parser = argparse.ArgumentParser(description="An example of Pasqal Cloud QRMI")
args = parser.parse_args()

QR_ID = "FRESNEL"

# instantiate a QRMI
qrmi = PasqalCloud()

# Check if QR it's accessible
is_avail = qrmi.is_accessible(QR_ID)
print('Pascal Cloud QR is %s accessible' % "not" if not is_avail else "")

# Get target
target = qrmi.target(QR_ID)
print("QR Target %s" % target.value)

# Send a task

# Get its status

# If not done by that time cancel it

# Get status

# Send send another task

# Wait for completion

# Get the results