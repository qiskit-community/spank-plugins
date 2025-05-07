#!/usr/bin/env python
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

"""generating input files for samplerV2"""

# pylint: disable=invalid-name, duplicate-code
import argparse
import os
import sys
import json
import requests
import numpy as np
from qiskit import qasm3
from qiskit.circuit.library import EfficientSU2
from qiskit.transpiler.preset_passmanagers import generate_preset_pass_manager
from qiskit.primitives.containers.sampler_pub import SamplerPub
from qiskit_ibm_runtime.utils import RuntimeEncoder
from qiskit_ibm_runtime.utils.backend_converter import convert_to_target
from qiskit_ibm_runtime.models import BackendProperties, BackendConfiguration

parser = argparse.ArgumentParser()
parser.add_argument("output")
parser.add_argument('--shots', type=int, default=10000)
args = parser.parse_args()

# Direct Access API endpoint
base_url = os.environ["IBMQRUN_DAAPI_ENDPOINT"]

# run with daa_sim(Qiskit Aer) ? if this is True, num_qubits of
# the circuit will be reduced to 7 qubits.
# set False if you run with real device.
use_daa_sim = True

# Use IAM based authentication
IBMCLOUD_IAM_ENDPOINT = os.environ["IBMQRUN_IAM_ENDPOINT"]
IBMCLOUD_API_KEY = os.environ["IBMQRUN_IAM_APIKEY"]
SERVICE_CRN = os.environ["IBMQRUN_SERVICE_CRN"]
iam_headers = {
    "content-type": "application/x-www-form-urlencoded",
    "accept": "application/json",
}

request_payload = {
    "grant_type": "urn:ibm:params:oauth:grant-type:apikey",
    "apikey": {IBMCLOUD_API_KEY},
}

token_response = requests.post(
    f"{IBMCLOUD_IAM_ENDPOINT}/identity/token",
    data=request_payload,
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

backend_name = os.environ["IBMQRUN_BACKEND"]

backend_config_url = f"{base_url}/v1/backends/{backend_name}/configuration"
backend_config_resp = requests.get(backend_config_url, headers=headers, timeout=10)
if backend_config_resp.status_code == 200:
    backend_config = BackendConfiguration.from_dict(backend_config_resp.json())
else:
    print(backend_config_resp.__dict__)
    sys.exit()

backend_props_url = f"{base_url}/v1/backends/{backend_name}/properties"
backend_props_resp = requests.get(backend_props_url, headers=headers, timeout=10)
if backend_props_resp.status_code == 200:
    backend_props = BackendProperties.from_dict(backend_props_resp.json())
else:
    print(backend_props_resp.__dict__)
    sys.exit()

target = convert_to_target(backend_config, backend_props)

# Create simple circuit - Use SamplerV2 example which is introduced
circuit = EfficientSU2(7, entanglement="linear", flatten=True)
circuit.measure_all()
# The circuit is parametrized, so we will define the parameter values for execution
param_values = np.random.rand(circuit.num_parameters)

# Generate transpiler target from backend configuration & properties
pm = generate_preset_pass_manager(
    optimization_level=1,
    target=target,
)
isa_circuit = pm.run(circuit)

# Generate QASM3 instructions
pub = SamplerPub.coerce((isa_circuit, param_values))
qasm3_str = qasm3.dumps(
    pub.circuit,
    disable_constants=True,
    allow_aliasing=True,
    experimental=qasm3.ExperimentalFeatures.SWITCH_CASE_V1,
)

# Generates JSON representation of primitive job
param_array = pub.parameter_values.as_array(pub.circuit.parameters).tolist()
input_json = {
    "pubs": [(qasm3_str, param_array)],
    "version": 2,
    "support_qiskit": False,
    "shots": args.shots,
    "options": {},
}

with open(args.output, "w", encoding="utf-8") as primitive_input_file:
    json.dump(input_json, primitive_input_file, cls=RuntimeEncoder, indent=2)
