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
import time

from pulser import Pulse, Register, Sequence
from pulser.devices import DigitalAnalogDevice

from qrmi import PasqalCloud, Payload, TaskStatus

# pulser will be probably replaced by eduardo qiskit analog gate


register = Register.square(2, spacing=5, prefix="q")
sequence = Sequence(register, DigitalAnalogDevice)
sequence.declare_channel("rydberg", "rydberg_global")
pulse = Pulse.ConstantPulse(100, 2.0, 2, 0.0)
sequence.add(pulse, "rydberg")
serialized_sequence = sequence.to_abstract_repr()

with open("pulser_seq.json", "w") as f:
    f.write(serialized_sequence)

parser = argparse.ArgumentParser(description="An example of Pasqal Cloud QRMI")
parser.add_argument("backend", help="backend name, FRESNEL") # Can consider making this FRESNEL or EMU
args = parser.parse_args()

# instantiate a QRMI
qrmi = PasqalCloud(args.backend)

# Check if QR it's accessible
is_avail = qrmi.is_accessible()
print('Pasqal Cloud QR is %s accessible' % "not" if not is_avail else "")

# Get target
target = qrmi.target()
print("QR Target %s" % target.value)

# nit:start_task would be nicer probably
task_id = qrmi.task_start(Payload.PasqalCloud(sequence=serialized_sequence, job_runs=1000))
print('Task ID: %s' % task_id)

# Get its status
print('Status after creation %s' % qrmi.task_status(task_id))

# Quickly stop it
qrmi.task_stop(task_id)

# Get status, it should be stopped
print('Status after cancelation %s' % qrmi.task_status(task_id))

# Send send another task
new_task_id = qrmi.task_start(Payload.PasqalCloud(sequence=serialized_sequence, job_runs=100))
print('New Task ID: %s' % new_task_id)

# Wait for completion
while True:
    status = qrmi.task_status(new_task_id)
    if status == TaskStatus.Completed:
        print('Task completed')
        time.sleep(2)
        break
    elif status == TaskStatus.Failed:
        print('Task failed')
        break
    else:
        print('Task status %s, waiting 1s' % status)
        time.sleep(1)

# Get the results
print('Results: %s' % qrmi.task_result(new_task_id).value)