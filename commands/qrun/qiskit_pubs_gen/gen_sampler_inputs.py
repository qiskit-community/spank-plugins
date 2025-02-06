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

"""generating input files for samplerV2"""

# pylint: disable=invalid-name
import sys
import datetime as dt
import json
import requests
import numpy as np
from qiskit import qasm3
from qiskit.circuit.library import IQP
from qiskit.quantum_info import random_hermitian
from qiskit.transpiler.preset_passmanagers import generate_preset_pass_manager
from qiskit_ibm_runtime.utils import RuntimeEncoder
from qiskit_ibm_runtime.utils.backend_converter import convert_to_target
from qiskit_ibm_runtime.models import BackendProperties, BackendConfiguration

# Direct Access API endpoint
base_url = "http://localhost:8290"

# IBM Cloud IAM API Key for access token generation"
iam_apikey = "demoapikey1"

# run with daa_sim(Qiskit Aer) ? if this is True, num_qubits of
# the circuit will be reduced to 7 qubits.
# set False if you run with real device.
use_daa_sim = True

headers = {
    "Authorization": f"apikey {iam_apikey}",
}
get_token_url = f"{base_url}/v1/token"
token_response = requests.post(
    get_token_url, data={}, headers=headers, timeout=10
)
resp_json = token_response.json()

# create HTTP header for subsequent API calls
access_token = resp_json["access_token"]
token_type = resp_json["token_type"]
now = dt.datetime.now(dt.timezone.utc)
headers = {
    "Authorization": f"{token_type} {access_token}",
    "IBM-API-Version": now.strftime("%Y-%m-%d"),
}
print(json.dumps(headers, indent=2))

backends_url = f"{base_url}/v1/backends"
backends_response = requests.get(backends_url, headers=headers, timeout=10)
if backends_response.status_code == 200:
    print(json.dumps(backends_response.json(), indent=4))
else:
    print(backends_response.__dict__)

backend_name = "fake_brisbane"

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

# Create simple circuit - Use SamplerV2 example which is introduced
# in "Getting started with Primitive" page.
# https://docs.quantum.ibm.com/guides/get-started-with-primitives#get-started-with-sampler
num_qubits = backend_config.num_qubits if not use_daa_sim else 7
mat = np.real(random_hermitian(num_qubits, seed=1234))
circuit = IQP(mat)
circuit.measure_all()

# Generate transpiler target from backend configuration & properties
target = convert_to_target(backend_config, backend_props)
pm = generate_preset_pass_manager(
    optimization_level=1,
    target=target,
)
isa_circuit = pm.run(circuit)
isa_circuit.draw()

# Generate QASM3 instructions
qasm3_str = qasm3.dumps(
    isa_circuit,
    disable_constants=True,
    allow_aliasing=True,
    experimental=qasm3.ExperimentalFeatures.SWITCH_CASE_V1,
)

# Generates JSON representation of primitive job
input_json = {
    "pubs": [[qasm3_str]],
    "version": 2,
    "support_qiskit": False,
    "shots": 10000,
    "options": {},
}

print(json.dumps(input_json, cls=RuntimeEncoder, indent=2))
with open("sampler_input.json", "w", encoding="utf-8") as primitive_input_file:
    json.dump(input_json, primitive_input_file, cls=RuntimeEncoder, indent=2)
primitive_input = json.dumps(input_json, cls=RuntimeEncoder)

print("done")
