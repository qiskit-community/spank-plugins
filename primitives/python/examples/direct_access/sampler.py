# This code is part of Qiskit.
#
# (C) Copyright 2024, 2025 IBM. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""SamplerV2 example with IBM Direct Access QRMI
Code is based on "Get started with Sampler" tutorial (https://docs.quantum.ibm.com/guides/get-started-with-primitives#get-started-with-sampler).
"""

# pylint: disable=invalid-name
import numpy as np
from dotenv import load_dotenv
from qiskit.circuit.library import EfficientSU2
from qiskit.transpiler.preset_passmanagers import generate_preset_pass_manager
from qrmi_primitives.ibm import IBMDirectAccessSamplerV2
from qrmi import IBMDirectAccess

from target import get_target

# Create QRMI
load_dotenv()
qrmi = IBMDirectAccess()

# Generate transpiler target from backend configuration & properties
target = get_target(qrmi)

# Create a circuit - You need at least one circuit as the input to the Sampler primitive.
circuit = EfficientSU2(127, entanglement="linear", flatten=True)
circuit.measure_all()
# The circuit is parametrized, so we will define the parameter values for execution
param_values = np.random.rand(circuit.num_parameters)

# The circuit and observable need to be transformed to only use instructions
# supported by the QPU (referred to as instruction set architecture (ISA) circuits).
# We'll use the transpiler to do this.
pm = generate_preset_pass_manager(
    optimization_level=1,
    target=target,
)
isa_circuit = pm.run(circuit)
print(f">>> Circuit ops (ISA): {isa_circuit.count_ops()}")

# Initialize QRMI Sampler
options = {
    "default_shots": 10000,
    "run_options": {
        "experimental": {
            "execution_path": "gen3-turbo",
        }
    }
}
sampler = IBMDirectAccessSamplerV2(options=options)

# Next, invoke the run() method to generate the output. The circuit and optional
# parameter value sets are input as primitive unified bloc (PUB) tuples.
job = sampler.run([(isa_circuit, param_values)])
print(f">>> Job ID: {job.job_id()}")
print(f">>> Job Status: {job.status()}")
result = job.result()

# Get results for the first (and only) PUB
pub_result = result[0]
print(f"Counts for the 'meas' output register: {pub_result.data.meas.get_counts()}")
