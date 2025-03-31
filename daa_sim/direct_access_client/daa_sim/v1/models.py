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

# pylint: disable=line-too-long

"""API responses"""
from datetime import datetime
from typing import List, Any, Literal, Optional, Set, Dict, Union
from pydantic import BaseModel, Field


#
# Authentication API
#
class TokenResponse(BaseModel):
    """Response of POST /v1/token"""

    access_token: str = Field(
        description="access token",
    )

    expires_in: int = Field(
        description="expires in seconds",
        examples=[3600],
    )

    expiration: int = Field(
        default=None,
        description="expiration",
    )

    token_type: str = Field(
        description="token type",
        examples=["Bearer"],
    )

    refresh_token: str = Field(
        default=None,
        description="(IBM Internal Use Only) token used to obtain a new access_token when the current access_token expires",
    )


#
# Backends API
#
class BackendResponse(BaseModel):
    """Response of GET /v1/backends/{backend_name}"""

    message: str = Field(
        default=None,
        description="Additional details related to the backend status.",
    )

    name: str = Field(
        description="Name of the backend.",
        examples=["ibm_bromont"],
    )

    status: Literal["online", "offline", "paused"] = Field(
        description="online (you can send jobs), offline (you cannot send jobs, retired) and paused (you can send jobs, but they won't be executed until unpaused).",
        examples=["online", "paused"],
    )

    version: str = Field(
        default=None,
        description="Version of the backend.",
    )


class BackendsResponse(BaseModel):
    """Response of GET /v1/backends"""

    backends: List[BackendResponse] = Field(
        description="A list of backend names available for direct access."
    )


class GateConfig(BaseModel):
    """Gate config, used by BackendConfigurationResponse"""

    name: str = Field(
        description="The gate name as it will be referred to in QASM",
        examples=["rz"],
    )

    parameters: List[str] = Field(
        min_length=1,
        description="Variable names for the gate parameters (if any)",
    )

    coupling_map: List[List[int]] = Field(
        default=None,
        description="List of qubit groupings which are coupled by this gate",
    )

    qasm_def: str = Field(
        description="Definition of this gate in terms of QASM primitives U and CX",
    )

    conditional: bool = Field(
        default=False,
        description="This specified gate supports conditional operations (true/false). If this is not specified, then the gate inherits the conditional property of the backend.",
    )

    latency_map: List[List[int]] = Field(
        default=None,
        description="List of qubit groupings which are coupled by this gate",
    )

    description: str = Field(
        default=None,
        description="Description of the gate operation",
    )


class ProcessorType(BaseModel):
    """Processor type, used by BackendConfigurationResponse"""

    family: str = Field(
        description="Processor family indicates quantum chip architecture",
    )

    revision: str = Field(
        description='Revision number reflects design variants within a given processor family. Is typically a semantic versioning value without the patch value, eg., "1.0".',
    )

    segment: str = Field(
        default=None,
        description="Segment, if indicated, is used to distinguish different subsets of the qubit fabric/chip",
    )


class TimingConstraints(BaseModel):
    """Timing constraints, used by BackendConfigurationResponse"""

    granularity: int = Field(
        default=None,
        description="Waveform memory data chunk size",
    )

    min_length: int = Field(
        default=None,
        description="Minimum number of samples required to define a pulse",
    )

    pulse_alignment: int = Field(
        default=None,
        description="Instruction triggering time resolution of pulse channel in units of dt",
    )

    acquire_alignment: int = Field(
        default=None,
        description="Instruction triggering time resolution of acquisition channel in units of dt",
    )


class Hamiltonian(BaseModel):
    """Hamiltonian of the backend, used by BackendConfigurationResponse"""

    h_latex: str = Field(
        description="The Hamiltonian in latex form",
    )

    h_str: List[str] = Field(
        default=None,
        description="The Hamiltonian in machine readable form",
    )

    vars: Any = Field(
        default=None,
        description="Variables in the h_str",
    )

    osc: Any = Field(
        default=None,
        description="Number of levels for each oscillator mode",
    )


class Operate(BaseModel):
    """Operate, used by BackendConfigurationResponse"""

    qubits: List[int] = Field(
        default=None,
    )


class ChannelProperties(BaseModel):
    """Channel properties, used by BackendConfigurationResponse"""

    operates: Operate = Field(
        default=None,
    )

    type: str = Field(
        default=None,
    )

    purpose: str = Field(
        default=None,
    )


class BackendConfigurationResponse(BaseModel):
    """Response of GET /v1/backends/{backend_name}/configuration"""

    backend_name: str = Field(
        description="Backend name",
    )

    sample_name: str = Field(
        default=None,
        description="Sample name",
    )

    backend_version: str = Field(
        pattern="[0-9]+.[0-9]+.[0-9]+$",
        description="Backend version in the form X.X.X",
    )

    n_qubits: int = Field(
        default=1,
        ge=1,
        format="int64",
        description="Number of qubits",
    )

    basis_gates: List[str] = Field(
        min_length=1,
        description="List of basis gates names on the backend",
    )

    # TODO: No need Optional - This is workaround of daa_service.py
    coupling_map: Optional[List[List[int]]] = Field(
        default=None,
        min_length=1,
        description="Array grouping qubits that are physically coupled together on the backend",
    )

    gates: List[GateConfig] = Field(
        min_length=0,
        description="List of basis gates on the backend",
    )

    local: bool = Field(
        description="Backend is local or remote (true/false)",
    )

    simulator: bool = Field(
        default=False,
        description="Backend is a simulator (true/false)",
    )

    conditional: bool = Field(
        default=False,
        description="Backend supports conditional operations (true/false)",
    )

    memory: bool = Field(
        default=False,
        description="Backend supports memory (true/false)",
    )

    max_shots: int = Field(
        ge=1,
        format="int64",
        description="Maximum number of shots supported",
    )

    max_experiments: int = Field(
        default=None,
        ge=1,
        format="int64",
        description="Maximum number of experiments supported",
    )

    n_registers: int = Field(
        default=1,
        ge=1,
        description="Number of register slots available for feedback (if conditional is true)",
    )

    register_map: List[List[int]] = Field(
        default=None,
        description="An array of dimension n_qubits X n_registers that specifies whether a qubit can store a measurement in a certain register slot",
    )

    configurable: bool = Field(
        default=False,
        description="Backend is configurable, if the backend is a simulator (true/false)",
    )

    credits_required: bool = Field(
        default=False,
        description="Backend requires credits to run a job (true/false)",
    )

    online_date: datetime = Field(
        default=None,
        description="Date the backend went online",
    )

    display_name: str = Field(
        default=None,
        description="Alternate name field for the backend",
    )

    description: str = Field(
        default=None,
        description="Description of the backend",
    )

    tags: Set[str] = Field(
        default=None,
    )

    rep_delay_range: List[List[float]] = Field(
        default=None,
        description="Range of delay times between programs (microseconds) allowed by backend.",
    )

    default_rep_delay: float = Field(
        default=None,
        description="Default rep delay.",
    )

    dynamic_reprate_enabled: bool = Field(
        default=False,
        description="Whether delay between programs can be set dynamically using 'rep_delay').",
    )

    measure_esp_enabled: bool = Field(
        default=False,
        description="Whether ESP readout is supported by the backend.",
    )

    supported_instructions: List[str] = Field(
        default=None,
        description="Instructions supported by the backend.",
    )

    supported_features: List[str] = Field(
        default=None,
        description="Array of features supported by the backend such as qobj, qasm3, etc.",
    )

    # TODO: Do we still need QV ? It is always set to none.
    quantum_volume: Optional[int] = Field(
        default=None,
        description="Backend quantum volume.",
    )

    processor_type: ProcessorType = Field(
        default=None,
    )

    qubit_lo_range: List[List[float]] = Field(
        min_length=1,
        description="Frequency range for the qubit LO",
    )

    meas_lo_range: List[List[float]] = Field(
        min_length=1,
        description="Frequency range for the measurement LO",
    )

    timing_constraints: TimingConstraints = Field(
        default=None,
    )

    open_pulse: bool = Field(
        description="The backend supports openPulse (true/false)",
    )

    n_uchannels: int = Field(
        ge=0,
        description="Number of additional control channels",
    )

    hamiltonian: Hamiltonian = Field(
        description="Hamiltonian of the backend",
    )

    u_channel_lo: List[List[Any]] = Field(
        description="Relationship of the U Channel LO's in terms of the qubit LO's",
    )

    meas_levels: List[int] = Field(
        description="Available measurement levels on the backend",
    )

    dt: float = Field(
        ge=0,
        description="Time discretization for the drive and U channels",
    )

    dtm: float = Field(
        ge=0,
        description="Time discretization for the measurement channels",
    )

    rep_times: List[float] = Field(
        description="Program execution times (microseconds) supported by backend.",
    )

    meas_map: List[List[int]] = Field(
        default=None,
        description="Grouping of measurement which are multiplexed.",
    )

    channel_bandwidth: List[float] = Field(
        default=None,
        description="Bandwidth of all channels (qubit,measurement and U)",
    )

    meas_kernels: List[str] = Field(
        description="Available measurement kernels",
    )

    discriminators: List[str] = Field(
        description="Available discriminators",
    )

    acquisition_latency: List[int] = Field(
        default=None,
        description="Array of dimension n_qubits x n_registers. Latency (in units of dt) to write a measurement result from qubit n into register slot m.",
    )

    conditional_latency: List[float] = Field(
        default=None,
        description="Array of dimension n_channels [d->u->m] x n_registers. Latency (in units of dt) to do a conditional operation on channel n from register slot m",
    )

    parametric_pulses: List[str] = Field(
        default=None,
        description="A list of available parametric pulse shapes",
    )

    channels: Dict[str, ChannelProperties] = Field(
        default=None,
        description="A dictionary where each entry represents a channel configuration and contains configuration values such as the channel's mapping to qubits.",
    )


class Nduv(BaseModel):
    """Name-date-unit-value"""

    date: Union[datetime, str] = Field(description="date")

    name: str = Field(description="name")

    unit: str = Field(description="unit")

    value: float = Field(description="value")


class GateProperties(BaseModel):
    """Class representing a gate's properties"""

    gate: str = Field(
        description="Gate name",
    )

    parameters: List[Nduv] = Field(
        description="List of Nduv objects for the name-date-unit-value for the gate",
    )

    qubits: List[int] = Field(
        description="A list of integers representing qubits",
    )


class BackendPropertiesResponse(BaseModel):
    """Response of GET /v1/backends/{backend_name}/properties"""

    backend_name: str = Field(
        description="Backend name",
    )

    backend_version: str = Field(
        pattern="[0-9]+.[0-9]+.[0-9]+$",
        description="Backend version in the form X.X.X",
    )

    gates: List[GateProperties] = Field(
        description="System gate parameters",
    )

    general: List[Nduv] = Field(
        description="General system parameters",
    )

    last_update_date: Union[datetime, str] = Field(
        description="Last date/time that a property was updated.",
    )

    qubits: List[List[Nduv]] = Field(
        min_length=1,
        description="System qubit parameters",
    )


#
# Error responses used across various APIs
#
class Error(BaseModel):
    """Direct Access API error"""

    code: str = Field(
        description="Error code which can be used in client code. Solutions for various error codes are available here https://docs.quantum.ibm.com/errors"
    )

    location: Optional[str] = Field(
        None,
        description="Location is a path-like string indicating where the error occurred. It typically begins with 'path', 'query', 'header', or 'body'. Example: 'body.items[3].tags' or 'path.thing-id'.",
    )

    message: str = Field(
        description="Message is a human-readable explanation of the error."
    )

    more_info: str = Field(description="Link to documentation on how to handle errors.")

    value: Optional[Any] = Field(
        None,
        description="Value is the value at the given location, echoed back to the client to help with debugging. This can be useful for e.g. validating that the client didn't send extra whitespace or help when the client did not log an outgoing request.",
    )


class ErrorResponse(BaseModel):
    """Direct Access API error response"""

    status_code: int = Field(description="status code")

    title: str = Field(description="title")

    trace: str = Field(description="trace")

    errors: List[Error] = Field(description="A list of errors")


#
# Jobs API
#
class StorageOption(BaseModel):
    """Storage option"""

    bucket_crn: str = Field(
        default=None,
        description="Fully specified CRN for the target Cloud Object Storage bucket. "
        "bucket_crn is required only when type is ibmcloud_cos.",
    )

    object_name: str = Field(
        default=None,
        description="Name/ID of the object in the IBM Cloud Object Storage bucket. "
        "May not be specified as a top level option directly under storage. "
        "object_name is required only when type is ibmcloud_cos.",
    )

    presigned_url: str = Field(
        default=None,
        description="Presigned GET or PUT URLs to read job params or write results "
        "and logs to. presigned_url is required only when type is s3_compatible.",
    )

    region: str = Field(
        default=None,
        pattern="^[a-zA-Z-]+$",
        description="region is required only when type is ibmcloud_cos.",
    )

    region_type: Literal["regional", "cross-region", "single-site"] = Field(
        default=None,
        description="Region, Cross-Region, or Single Data Center as defined by IBM "
        "Cloud Object Storage (https://cloud.ibm.com/docs/cloud-object-storage?"
        "topic=cloud-object-storage-endpoints). region_type is required "
        "only when type is ibmcloud_cos.",
    )

    type: Literal["ibmcloud_cos", "s3_compatible"] = Field(
        description="IBM Cloud Object Storage (ibmcloud_cos) is available for "
        "IBM Internal Use Only."
    )


class Storage(BaseModel):
    """Storage specifications"""

    input: StorageOption = Field(
        description="Location from where input to job will be read (omitted properties "
        "will be inherited from default storage option)."
    )

    results: StorageOption = Field(
        description="Location for where job results will be stored (omitted properties "
        "will be inherited from default storage option)."
    )

    logs: StorageOption = Field(
        default=None,
        description="Location for where job logs will be stored (omitted properties "
        "will be inherited from default storage option).",
    )


class Job(BaseModel):
    """Job specification"""

    id: str = Field(  # pylint: disable=invalid-name
        description="Job identifier. Recommended to be UUID.",
        min_length=1,
        max_length=256,
        examples=["6e32f594-189e-4bc5-89a2-3c21e1c7e75a"],
    )

    backend: str = Field(
        description="Name of the backend.",
        min_length=1,
        max_length=256,
        examples=["ibm_bromont"],
    )

    program_id: str = Field(
        description="ID of the program.",
        min_length=1,
        max_length=256,
        examples=["sampler", "estimator"],
    )

    log_level: Literal["debug", "info", "warning", "error", "critical"] = Field(
        default="warning",
        description="Logging level of the program.",
        examples=["info"],
    )

    timeout_secs: int = Field(
        ge=1,
        format="int64",
        description="Time (in seconds) after which job should time out and get cancelled. It is based on system execution time (not wall clock time). System execution time is the amount of time that the system is dedicated to processing your job.",
    )

    storage: Storage


class Usage(BaseModel):
    """Usage data included in JobResponse"""

    quantum_nanoseconds: int = Field(
        default=None,
        format="int64",
        description="Execution time on quantum device in nanoseconds.",
    )


class JobResponse(BaseModel):
    """Response of GET /v1/jobs/{job_id}"""

    backend: str = Field(description="Name of the backend", examples=["ibm_rensselaer"])

    created_time: str = Field(
        description="Time when job was created.",
    )

    # this field is included once job is completed.
    end_time: str = Field(
        default=None,
        description="Time when job reached a terminal status.",
    )

    log_level: Literal["debug", "info", "warning", "error", "critical"] = Field(
        default=None,
        description="Logging level of the program.",
    )

    # pylint: disable=invalid-name
    id: str = Field(
        description="Unique ID for the Job (UUID V4)",
    )

    program_id: str = Field(
        description="ID of the program.",
        min_length=1,
        max_length=256,
        examples=["sampler", "estimator"],
    )

    status: Literal["Running", "Completed", "Cancelled", "Failed"] = Field(
        description="Current status of the job.",
    )

    reason_code: int = Field(
        default=None,
        format="int64",
        description="This field will be set to a numeric error code when the job has failed.",
    )

    reason_message: str = Field(
        default=None,
        description="Reason message explaining why the job is in a particular status. Ex: If a job has Failed, this field will have the error message.",
    )

    usage: Usage = Field(
        description="Job usage details.",
    )

    timeout_secs: int = Field(
        ge=1,
        format="int64",
        description="Time (in seconds) after which job should time out and get cancelled.",
    )

    storage: Storage


class JobsResponse(BaseModel):
    """Response of GET /v1/jobs"""

    jobs: List[JobResponse] = Field(description="A list of job details")
