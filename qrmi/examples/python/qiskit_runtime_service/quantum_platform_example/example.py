# -*- coding: utf-8 -*-
#
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

"""An example of IBM Qiskit Runtime Service QRMI python-bindings"""

import os
import time
import json

import requests
import json

from dotenv import load_dotenv
from functools import wraps
from qiskit.circuit import QuantumCircuit, QuantumRegister, ClassicalRegister
from qiskit.transpiler.preset_passmanagers import generate_preset_pass_manager
from qiskit_ibm_runtime import Session, SamplerV2 as Sampler

from qiskit_ibm_runtime.exceptions import IBMRuntimeError
from qiskit_ibm_runtime.utils.backend_converter import convert_to_target

def _active_session(func):  # type: ignore
    """Decorator used to ensure the session is active."""

    @wraps(func)
    def _wrapper(self, *args, **kwargs):  # type: ignore
        if not self._active:
            raise IBMRuntimeError("The session is closed.")
        return func(self, *args, **kwargs)

    return _wrapper

class QRS_Session(Session):
    def __init__(self):

        self._service = None
        self._instance = os.environ["SESSION_INSTANCE"]
        self._session_id = os.environ["SESSION_ID"]
        self._backend_name = os.environ["RESOURCE_ID"]
        self._max_time = os.environ["SESSION_MAX_TTL"]
        self.api_token = os.environ.get("IAM_APIKEY")
        self._backend = None
        print(f"Session acquired with ID: {self.session_id}")

        self._active = True
        
        # Acquire a session using the backend resource ID from environment variables
        #self.session_id = self.qrmi.acquire() 
        
    @_active_session
    def _run(self, program_id: str, inputs: str, **kwargs):
        """
        Run a Qiskit primitive by sending a task via the QRMI service.
        This method creates a payload, starts the task, polls for status,
        prints the result, and stops the task.
        """
        # Define the endpoint and get the API token from environment variables
        reqUrl = "https://api.quantum-computing.ibm.com/runtime/jobs"
        
        headersList = {
            "Accept": "application/json",
            "Authorization": f"Bearer {self.api_token}",
            "Content-Type": "application/json"
        }
        
        # Construct the payload using function arguments and environment variables as fallbacks.
        payload = json.dumps({
            "program_id": program_id,  # Use the provided program_id
            "backend": os.environ.get("RESOURCE_ID", self._backend_name),
            "hub": os.environ.get("SESSION_HUB", "ibm-q"),
            "group": os.environ.get("SESSION_GROUP", "open"),
            "project": os.environ.get("SESSION_PROJECT", "main"),
            "session_id": self._session_id,
            "params": {
                "pubs": [[ inputs ]],  # Use the provided OpenQASM or equivalent input
                "options": {},
                "version": 2
            }
        }, default=str)
        
        # Submit the job via POST request.
        response = requests.request("POST", reqUrl, data=payload, headers=headersList)
        result_json = response.json()
        print(result_json)
        
        # Extract the job ID from the response.
        job_id = result_json.get("id")
        if not job_id:
            raise Exception("Job ID not found in response.")
        print(f"Task started: {job_id}")
        
        # Poll until the task status is no longer "RUNNING" or "QUEUED".
        while True:
            status_response = requests.request("GET", f"{reqUrl}/{job_id}", headers=headersList)
            status_json = status_response.json()
            status = status_json.get("status")
            if status not in ["RUNNING", "QUEUED", "Running", "Queued"]:
                break
            time.sleep(1)
        
        final_status = status_json.get("status")
        print(f"Task ended with status: {final_status}")
        
        # Retrieve the task result.
        result = status_json.get("result")
        print("Task result:")
        print(result)
        
        # (Optional) Stop the task if necessary by sending a DELETE request or similar.
        # requests.request("DELETE", f"{reqUrl}/{job_id}", headers=headersList)
        
        return result

        
    def target(self):
        # Retrieve token and backend name from environment variables if not set
        backend_name = self._backend_name or os.environ.get("RESOURCE_ID")
        base_url = "https://api.quantum-computing.ibm.com/runtime/backends"
        
        headers = {
            "Authorization": f"Bearer {self.api_token}",
            "Accept": "application/json"
        }
        
        # Get the backend configuration
        config_url = f"{base_url}/{backend_name}/configuration"
        config_response = requests.get(config_url, headers=headers)
        config_response.raise_for_status()
        conf = config_response.json()
        
        # Get the backend properties
        props_url = f"{base_url}/{backend_name}/properties"
        props_response = requests.get(props_url, headers=headers)
        props_response.raise_for_status()
        props = props_response.json()
        try:
            return convert_to_target(conf, props)
        except:
            return None
        
if __name__ == "__main__":
    load_dotenv()

    # Build a Bell state circuit.
    qr = QuantumRegister(2, name="qr")
    cr = ClassicalRegister(2, name="cr")
    qc = QuantumCircuit(qr, cr, name="bell")
    qc.h(qr[0])
    qc.cx(qr[0], qr[1])
    qc.measure(qr, cr)

    # Create a session with the acquired QRMI instance.
    session = QRS_Session()

    # Transpile the circuit using a preset pass manager.
    target = session.target()
    pm = generate_preset_pass_manager(
        optimization_level=1,
        target=target,
        )
    isa_circuit = pm.run(qc)

    # Run the circuit using the Sampler.
    # The sampler will invoke session._run under the hood.
    sampler = Sampler(mode=session)
    job = sampler.run([isa_circuit])
    pub_result = job.result()[0]
    print(f"Sampler job ID: {job.job_id()}")
    print(f"Counts: {pub_result.data.cr.get_counts()}")