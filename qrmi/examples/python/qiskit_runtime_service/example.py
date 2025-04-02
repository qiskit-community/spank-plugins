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

from dotenv import load_dotenv
from qrmi import IBMQiskitRuntimeService, Payload, TaskStatus
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
    def __init__(self, qrmi_instance: IBMQiskitRuntimeService):
        self.qrmi = qrmi_instance
        self._service = None
        self._instance = None
        self._backend = None

        self._active = True
        self._max_time = os.environ["SESSION_MAX_TTL"]
        self.backend_name = os.environ["QRMI_RESOURCE_ID"]
        # Acquire a session using the backend resource ID from environment variables
        self._session_id = self.qrmi.acquire(self.backend_name)
        print(f"Session acquired with ID: {self._session_id}")
    @_active_session
    def _run(self, program_id: str, inputs: str, **kwargs):
        """
        Run a Qiskit primitive by sending a task via the QRMI service.
        This method creates a payload, starts the task, polls for status,
        prints the result, and stops the task.
        """
        json_inputs = json.dumps(inputs, default=str)
        payload = Payload.QiskitPrimitive(input=json_inputs, program_id=program_id)
        job_id = self.qrmi.task_start(payload)
        print(f"Task started: {job_id}")

        # Poll until the task is no longer Running or Queued.
        while True:
            status = self.qrmi.task_status(job_id)
            if status not in [TaskStatus.Running, TaskStatus.Queued]:
                break
            time.sleep(1)

        final_status = self.qrmi.task_status(job_id)
        print(f"Task ended with status: {final_status}")

        # Retrieve and print the task result.
        result = self.qrmi.task_result(job_id)
        print("Task result:")
        print(result.value)

        # Stop the task.
        self.qrmi.task_stop(job_id)
        return result

    def close(self):
        """Release the acquired session."""
        self.qrmi.release(self._session_id)
        print(f"Session {self._session_id} released.")

    def cancel(self, job_id: str):
        """Cancel a running task."""
        self.qrmi.task_stop(job_id)
        print(f"Task {job_id} cancelled.")
    
    def target(self):
        # Call the target function with the resource_id and parse the returned JSON string.
        target_obj = self.qrmi.target(self.backend_name)
        target_json = json.loads(target_obj.value)
        
        print("Target configuration:")
        print(json.dumps(target_json, indent=2))
        
        # Extract configuration and properties from the parsed dictionary.
        self.backend_config = target_json["configuration"]
        self.backend_props = target_json["properties"]
        try:
            return convert_to_target(self.backend_config, self.backend_props)
        except:
            return None
        
if __name__ == "__main__":

    load_dotenv()

    # Instantiate the Qiskit Runtime Service.
    qrmi = IBMQiskitRuntimeService()
    print("Qiskit Runtime Service instantiated:")

    # Create a session with the acquired QRMI instance.
    session = QRS_Session(qrmi)
    print(f'session_id from qrmi = {session._session_id}')
    print(f'session_id from envinronment = {os.environ["QRMI_IBM_QRS_SESSION_ID"]}')

    # Check if the backend is accessible.
    resource_id = os.environ["QRMI_RESOURCE_ID"]
    accessible = qrmi.is_accessible(resource_id)
    print(f"Backend {resource_id} accessible: {accessible}")

    # Build a Bell state circuit.
    qr = QuantumRegister(2, name="qr")
    cr = ClassicalRegister(2, name="cr")
    qc = QuantumCircuit(qr, cr, name="bell")
    qc.h(qr[0])
    qc.cx(qr[0], qr[1])
    qc.measure(qr, cr)

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
    job = sampler.run([qc])
    pub_result = job.result()[0]
    print(f"Sampler job ID: {job.job_id()}")
    print(f"Counts: {pub_result.data.cr.get_counts()}")

    metadata = qrmi.metadata()
    print("Metadata:")
    print(json.dumps(metadata, indent=2))

    # Release the lock and close the session.
    session.close()
