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

"""generating input files for estimatorV2"""

# pylint: disable=invalid-name
import sys
import json
import requests
from qiskit import qasm3
from qiskit import QuantumCircuit
from qiskit.quantum_info import SparsePauliOp
from qiskit.transpiler.preset_passmanagers import generate_preset_pass_manager
from qiskit.primitives.containers.estimator_pub import EstimatorPub
from qiskit_ibm_runtime.utils.backend_converter import convert_to_target
from qiskit_ibm_runtime.utils import RuntimeEncoder
from qiskit_ibm_runtime.models import BackendProperties, BackendConfiguration

# Direct Access API endpoint
base_url = "http://localhost:8290"

# run with daa_sim(Qiskit Aer) ? if this is True, num_qubits of
# the circuit will be reduced to 7 qubits.
# set False if you run with real device.
use_daa_sim = True

# Use IAM based authentication
IBMCLOUD_IAM_ENDPOINT="https://iam.cloud.ibm.com"
IBMCLOUD_API_KEY="YOUR_API_KEY"
SERVICE_CRN="YOUR_PROVISIONED_INSTANCE - crn:v1:...."
iam_headers = {
    "content-type": "application/x-www-form-urlencoded",
    "accept": "application/json",
}
token_response = requests.post(
    f"{IBMCLOUD_IAM_ENDPOINT}/identity/token",
    data=f"grant_type=urn:ibm:params:oauth:grant-type:apikey&apikey={IBMCLOUD_API_KEY}",
    headers=iam_headers,
    timeout=10,
)
resp_json = token_response.json()
access_token = resp_json["access_token"]
token_type = resp_json["token_type"]
headers = {
    "Authorization": f"{token_type} {access_token}",
    "Service-CRN": SERVICE_CRN,
}
# End - IAM based authentication

# (Deprecated) Use AppId based authentication
# APPID_CLIENT_ID="YOUR_APPID_CLIENT_ID"
# APPID_SECRET="YOUR_APPID_SECRET"
# token_response = requests.post(
#    f"{base_url}/v1/token",
#    data={},
#    auth=(APPID_CLIENT_ID, APPID_SECRET)
#    timeout=10,
# )
# resp_json = token_response.json()
# access_token = resp_json["access_token"]
# token_type = resp_json["token_type"]
# headers = {
#    "Authorization": f"{token_type} {access_token}",
# }
# End - AppId based authentication

print(json.dumps(headers, indent=2))

backends_url = f"{base_url}/v1/backends"
backends_response = requests.get(backends_url, headers=headers, timeout=10)
if backends_response.status_code == 200:
    print(json.dumps(backends_response.json(), indent=4))
else:
    print(backends_response.__dict__)

backend_name = "fake_cairo"

backend_config_url = f"{base_url}/v1/backends/{backend_name}/configuration"
backend_config_resp = requests.get(backend_config_url, headers=headers, timeout=10)
if backend_config_resp.status_code == 200:
    backend_config_json = backend_config_resp.json()
    print(json.dumps(backend_config_json, indent=4))
    backend_config = BackendConfiguration.from_dict(backend_config_json)
    print(backend_config)
else:
    print(backend_config_resp.__dict__)
    sys.exit()

backend_props_url = f"{base_url}/v1/backends/{backend_name}/properties"
backend_props_resp = requests.get(backend_props_url, headers=headers, timeout=10)
if backend_props_resp.status_code == 200:
    backend_props_json = backend_props_resp.json()
    print(json.dumps(backend_props_json, indent=4))
    backend_props = BackendProperties.from_dict(backend_props_json)
    print(backend_props)
else:
    print(backend_props_resp.__dict__)
    sys.exit()

# Create a new circuit with two qubits
qc = QuantumCircuit(2)

# Add a Hadamard gate to qubit 0
qc.h(0)

# Perform a controlled-X gate on qubit 1, controlled by qubit 0
qc.cx(0, 1)

# Return a drawing of the circuit using MatPlotLib ("mpl"). This is the
# last line of the cell, so the drawing appears in the cell output.
# Remove the "mpl" argument to get a text drawing.
qc.draw()

observables_labels = ["IZ", "IX", "ZI", "XI", "ZZ", "XX"]
observables = [SparsePauliOp(label) for label in observables_labels]

# Generate transpiler target from backend configuration & properties
target = convert_to_target(backend_config, backend_props)
pm = generate_preset_pass_manager(
    optimization_level=1,
    target=target,
)

# Convert to an ISA circuit and layout-mapped observables.
isa_circuit = pm.run(qc)

# IBM primitive doesn't make any assumption that circuit layout != observable layout,
# in other words, we need to manually tweak the ISA circuit before dump it into OpenQASM3
# string (e.g. prepending barrier for all qubits) otherwise IBM primitive may reconstruct
# circuit with wrong number of qubits.
isa_circuit.barrier()

mapped_observables = [
    observable.apply_layout(isa_circuit.layout) for observable in observables
]

isa_circuit.draw()

# Generate QASM3 instructions
pub = EstimatorPub.coerce((isa_circuit, mapped_observables))
qasm3_str = qasm3.dumps(
    pub.circuit,
    disable_constants=True,
    allow_aliasing=True,
    experimental=qasm3.ExperimentalFeatures.SWITCH_CASE_V1,
)

observables = pub.observables.tolist()

# Generates JSON representation of estimator job
input_json = {
    "pubs": [(qasm3_str, observables)],
    "version": 2,
    "support_qiskit": False,
    "resilience_level": 1,
    "options": {
        "default_shots": 5000,
    },
}

print(json.dumps(input_json, cls=RuntimeEncoder, indent=2))
with open("estimator_input.json", "w", encoding="utf-8") as primitive_input_file:
    json.dump(input_json, primitive_input_file, cls=RuntimeEncoder, indent=2)

print("done")
