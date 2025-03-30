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

# pylint: disable=missing-function-docstring, invalid-name, fixme, too-many-locals, line-too-long

"""Direct Access API service"""
import logging
import logging.config
import json
import datetime as dt
from pathlib import Path
from typing import Dict, Any, List
from concurrent.futures import (
    Executor,
    ThreadPoolExecutor,
)
from abc import ABC, abstractmethod
import os
from os.path import isdir, isfile
import multiprocessing as mp
import threading
import importlib
from collections import OrderedDict

import numpy as np
import urllib3
from qiskit import qasm3
from qiskit.circuit import QuantumCircuit, ClassicalRegister
from qiskit.primitives.containers import (
    PrimitiveResult,
    PubResult,
    BitArray,
    DataBin,
)
from qiskit_aer.primitives import SamplerV2, EstimatorV2

from qiskit_ibm_runtime import RuntimeEncoder, RuntimeDecoder
from qiskit_ibm_runtime.fake_provider.backends import (
    FakeBrisbane,
    FakeCairoV2,
    FakeLagosV2,
    FakeTorino,
)

from direct_access_client.daa_sim.errors import (
    BackendNotFoundError,
    JobNotFoundError,
    DuplicateJobError,
    InvalidInputError,
    JobNotCancellableError,
    UnableToDeleteJobInNonTerminalStateError,
    ServiceNotAvailableError,
)

from direct_access_client.daa_sim.consts import DEFAULT_JOBS_DIR
from direct_access_client.daa_sim.logger import UserLogger
from direct_access_client.daa_sim.externals.standard_json_encoder import (
    StandardJSONEncoder,
)

_logger = logging.getLogger(__name__)


class ProgramRuntimeEncoder(RuntimeEncoder):
    """JSON Encoder that uses terra qpy"""

    def default(self, obj: Any) -> Any:  # pylint: disable=arguments-differ
        # pylint: disable=too-many-return-statements
        if isinstance(obj, np.number):
            return obj.item()
        if isinstance(obj, BitArray):
            out_val = {"array": obj.array, "num_bits": obj.num_bits}
            return {"__type__": "BitArray", "__value__": out_val}
        if isinstance(obj, DataBin):
            # pylint: disable=protected-access
            out_val = {
                "field_names": obj._FIELDS,
                "field_types": [str(field_type) for field_type in obj._FIELD_TYPES],
                "shape": obj._SHAPE,
                "fields": {
                    field_name: getattr(obj, field_name) for field_name in obj._FIELDS
                },
            }
            return {"__type__": "DataBin", "__value__": out_val}
        if isinstance(obj, PubResult):
            out_val = {"data": obj.data, "metadata": obj.metadata}
            return {"__type__": "PubResult", "__value__": out_val}
        if isinstance(obj, PrimitiveResult):
            out_val = {"pub_results": list(obj), "metadata": obj.metadata}
            return {"__type__": "PrimitiveResult", "__value__": out_val}
        return super().default(obj)


class SharedStorage(ABC):
    """Abstract class of storage"""

    def __init__(self):
        pass

    @abstractmethod
    def get(self, storage: Dict[str, str]) -> str:
        pass

    @abstractmethod
    def put(self, storage: Dict, data: str) -> None:
        pass


class S3Compatible(SharedStorage):
    """S3 compatible storage to share input and output"""

    def __init__(self):
        pass

    def get(self, storage: Dict[str, str]) -> str:
        with urllib3.PoolManager(cert_reqs="CERT_NONE") as http:
            resp = http.request("GET", storage["presigned_url"])
            if resp.status == 200:
                return resp.data.decode("utf-8")

            raise Exception(  # pylint: disable=broad-exception-raised
                "Failed to deserialize job json"
            )

    def put(self, storage: Dict, data: str) -> None:
        with urllib3.PoolManager(cert_reqs="CERT_NONE") as http:
            resp = http.request("PUT", storage["presigned_url"], body=data)
            if resp.status != 200:
                raise Exception(  # pylint: disable=broad-exception-raised
                    "Failed to upload result"
                )


class FileStorage(SharedStorage):
    """file system to share input and output"""

    def __init__(self):
        pass

    def get(self, storage: Dict[str, str]) -> str:
        file_path = storage["file_path"]
        with open(file_path, encoding="utf-8") as f:
            return f.read()

    def put(self, storage: Dict, data: str) -> None:
        file_path = storage["file_path"]
        with open(file_path, mode="w", encoding="utf-8") as f:
            f.write(data)


class DAAService:
    """Implementation of Direct Access API service"""

    DEFAULT_SAMPLER_SHOTS: int = 1000

    DEFAULT_AVALIABLE_BACKENDS = OrderedDict(
        [
            (FakeBrisbane.backend_name, FakeBrisbane),  # 127Q
            (FakeCairoV2.backend_name, FakeCairoV2),  # 27Q
            (FakeLagosV2.backend_name, FakeLagosV2),  # 7Q
            (FakeTorino.backend_name, FakeTorino),  # 133Q
        ]
    )

    def __init__(
        self,
        *,
        executor: Executor = None,
        include_opt_fields=False,
        aer_options: Dict = None,
        multiprocess: bool = False,
        jobs_dir: str = DEFAULT_JOBS_DIR,
        backends: List[Dict[str, str]] = None,
    ):
        """
        Constracts DAAService instance

        Args:
            executor(Executor): Thread pool executor to run the background tasks.
            include_opt_fields(bool): True if you want to include optional fields in the responses.
            aer_options(Dict): may contain backend_options for AerSimulator ctor with "samplerV2" and "estimatorV2" keys.
            multiprocess(bool): True if job will be executed in the spawned process, otherwise False(Thread).
            jobs_dir(str): Job metadata directory path
            backend(List[Dict[str, str]]): Backend list to override default.
        """
        self._active = False
        self._incl_opt_values = include_opt_fields
        self._executor = executor if executor else ThreadPoolExecutor()
        self._jobs_dir = Path(jobs_dir)
        self._jobs_dir.mkdir(exist_ok=True)

        # Overrides backend list if specified.
        self._backends_spec = backends
        if backends is None:
            self._available_backends = DAAService.DEFAULT_AVALIABLE_BACKENDS
        else:
            self._available_backends = OrderedDict()
            for backend in backends:
                try:
                    backend_name, clazz = self.load_backend(
                        backend["module"], backend["clazz"]
                    )
                    self._available_backends[backend_name] = clazz
                except (ModuleNotFoundError, AttributeError) as err:
                    _logger.warning(
                        "Loading backend failed. Reason: %s. Skipped.",
                        str(err),
                    )

        _logger.info("backends: %s", list(self._available_backends.keys()))

        self._storage_options = {
            "s3_compatible": S3Compatible(),
            "file_system": FileStorage(),
        }
        self._active = True

        self._estimatorV2_options = (
            aer_options.get("estimatorV2", {}) if aer_options is not None else {}
        )
        self._samplerV2_options = (
            aer_options.get("samplerV2", {}) if aer_options is not None else {}
        )

        self._mp_workers = {}
        self._mp_enabled = multiprocess
        # exclusive control is required to R/W job status files because
        # writer and reader are running in different threads or processes.
        # without this lock/unlock, JSONDecodeError: Extra data: line * column * (char *)
        # error will intermittently be occurred json.load().
        if self._mp_enabled is True:
            self._status_lock = mp.RLock()
        else:
            self._status_lock = threading.RLock()

    @property
    def default_backend_name(self):
        """Returns the name of the first backend in self._available_backends as default.
        This is used by pytest testcases only.
        """
        return list(self._available_backends.keys())[0]

    def _assert_if_inactive(self):
        """throw execption if service is closed"""
        if not self._active:
            raise ServiceNotAvailableError()

    def close(self):
        """close service"""
        self._assert_if_inactive()
        self._executor.shutdown()

        # cancel all running jobs.
        for job in self.get_jobs()["jobs"]:
            if job["status"] == "Running":
                self.cancel_job(job["id"])

        self._active = False

        # Next line is required to fix "There appear to
        # be 1 leaked semaphore objects to clean up at shutdown" warning
        # by resource tracker
        self._status_lock = None

    def __enter__(self, *args):
        return self

    def __exit__(self, *args):
        self.close()

    def available_backends(self) -> List[str]:
        return self._available_backends.keys()

    def backends(self) -> Dict:
        return {
            "backends": [
                self.get_backend_details(backend_name)
                for backend_name in self._available_backends
            ]
        }

    def _get_backend(self, backend_name: str):
        if backend_name not in self._available_backends:
            raise BackendNotFoundError(backend_name)

        return self._available_backends[backend_name]()

    def get_backend_details(self, backend_name):
        if backend_name not in self._available_backends:
            raise BackendNotFoundError(backend_name)

        resp = {
            "name": backend_name,
            "status": "online",
        }
        if self._incl_opt_values:
            resp["message"] = f"backend {backend_name}"
            resp["version"] = "1.0.0"

        return resp

    _CONFIG_NAMES = {
        "backend_name",
        "sample_name",
        "backend_version",
        "n_qubits",
        "basis_gates",
        "coupling_map",
        "gates",
        "local",
        "simulator",
        "conditional",
        "memory",
        "max_shots",
        "max_experiments",
        "n_registers",
        "register_map",
        "configurable",
        "credits_required",
        "online_date",
        "display_name",
        "description",
        "tags",
        "default_rep_delay",
        "dynamic_reprate_enabled",
        "measure_esp_enabled",
        "supported_instructions",
        "supported_features",
        "quantum_volume",
        "processor_type",
        "qubit_lo_range",
        "meas_lo_range",
        "timing_constraints",
        "open_pulse",
        "n_uchannels",
        "hamiltonian",
        "u_channel_lo",
        "meas_levels",
        "dt",
        "dtm",
        "rep_times",
        "meas_map",
        "channel_bandwidth",
        "meas_kernels",
        "discriminators",
        "acquisition_latency",
        "conditional_latency",
        "parametric_pulses",
        "channels",
    }

    def get_backend_configuration(self, backend_name: str) -> Dict:
        backend = self._get_backend(backend_name)
        if backend_name == "aer":
            config_dict = backend.configuration().to_dict()
            config_dict["qubit_lo_range"] = [[0, 0]]
            config_dict["meas_lo_range"] = [[0, 0]]
            config_dict["n_uchannels"] = 0
            config_dict["hamiltonian"] = {"h_latex": ""}
            config_dict["u_channel_lo"] = []
            config_dict["meas_levels"] = []
            config_dict["dt"] = 0
            config_dict["dtm"] = 0
            config_dict["rep_times"] = []
            config_dict["meas_kernels"] = []
            config_dict["discriminators"] = []
            config_dict["open_pulse"] = False
        else:
            config_dict = self._get_backend(backend_name)._get_conf_dict_from_json()
            config_dict["u_channel_lo"] = [
                [
                    {
                        "q": subitem["q"],
                        "scale": [subitem["scale"].real, subitem["scale"].imag],
                    }
                    for subitem in item
                ]
                for item in config_dict["u_channel_lo"]
            ]

        ret = {}
        for config_key in config_dict:
            if config_key in DAAService._CONFIG_NAMES:
                ret[config_key] = config_dict[config_key]

        ret["gates"] = [
            gate for gate in ret["gates"] if gate["qasm_def"] and gate["parameters"]
        ]

        return ret

    def get_backend_properties(self, backend_name: str) -> Dict:
        backend = self._get_backend(backend_name)
        if backend_name == "aer":
            config_dict = backend.configuration().to_dict()
            return {
                "backend_name": config_dict["backend_name"],
                "backend_version": config_dict["backend_version"],
                "last_update_date": "2000-01-01 00:00:00Z",
                "qubits": [
                    [
                        {
                            "date": "2000-01-01 00:00:00Z",
                            "name": "T1",
                            "unit": "\u00b5s",
                            "value": 0.0,
                        },
                        {
                            "date": "2000-01-01 00:00:00Z",
                            "name": "T2",
                            "unit": "\u00b5s",
                            "value": 0.0,
                        },
                        {
                            "date": "2000-01-01 00:00:00Z",
                            "name": "frequency",
                            "unit": "GHz",
                            "value": 0.0,
                        },
                        {
                            "date": "2000-01-01 00:00:00Z",
                            "name": "readout_error",
                            "unit": "",
                            "value": 0.0,
                        },
                        {
                            "date": "2000-01-01 00:00:00Z",
                            "name": "operational",
                            "unit": "",
                            "value": 1,
                        },
                    ]
                    for _ in range(config_dict["n_qubits"])
                ],
                "gates": config_dict["gates"],
                "general": [],
            }

        # if backend != aer
        props = self._get_backend(backend_name).properties().to_dict()
        config_dict = self.get_backend_configuration(backend_name)
        for fill_key in ["backend_name", "backend_version"]:
            if props.get(fill_key) in {None, ""}:
                props[fill_key] = config_dict[fill_key]
        # FIX: last_update_date is None even though this is required property
        # Workaround to pickup the date value in the first qubit's Nduv.
        props["last_update_date"] = props["qubits"][0][0]["date"]
        return props

    def _get_storage(self, storage_type: str) -> SharedStorage:
        if storage_type in self._storage_options:
            return self._storage_options[storage_type]

        raise InvalidInputError(
            message=f"Unsupported storage option: type={storage_type}",
            value=storage_type,
        )

    def job_status(
        self,
        job: Dict[str, str],
        status: str = None,
        expected_prev_status: str = None,
        reason_message: str = None,
        reason_code: int = None,
        create: bool = False,
    ) -> Dict[str, str]:
        with self._status_lock:
            return self._job_status(
                job, status, expected_prev_status, reason_message, reason_code, create
            )

    def _job_status(
        self,
        job: Dict[str, str],
        status: str = None,
        expected_prev_status: str = None,
        reason_message: str = None,
        reason_code: int = None,
        create: bool = False,
    ) -> Dict[str, str]:
        self._assert_if_inactive()
        job_status_file = self._jobs_dir / job["id"]

        if not status:
            if not job_status_file.is_file():
                return {"id": job["id"], "status": "NA"}
            with open(job_status_file, encoding="utf-8") as f:
                return json.load(f)
        else:
            if create and job_status_file.is_file():
                raise DuplicateJobError(job["id"])
            job["status"] = status
            if create and status == "Running":
                job["created_time"] = (
                    dt.datetime.now(dt.timezone.utc).isoformat().replace("+00:00", "Z")
                )
            if status in ["Completed", "Failed", "Cancelled"]:
                job["end_time"] = (
                    dt.datetime.now(dt.timezone.utc).isoformat().replace("+00:00", "Z")
                )
            if reason_message:
                job["reason_message"] = reason_message
            if reason_code:
                job["reason_code"] = reason_code
            if create or not job_status_file.exists():
                with open(job_status_file, mode="w", encoding="utf-8") as f:
                    json.dump(job, f)
            else:
                with open(job_status_file, mode="r+", encoding="utf-8") as f:
                    if expected_prev_status:
                        prev = json.load(f)
                        f.seek(0)
                        if prev["status"] != expected_prev_status:
                            raise ValueError(
                                f"status is not expected: id={job['id']}, status={prev['status']}, expected={expected_prev_status}"
                            )
                    json.dump(job, f)
                    if expected_prev_status:
                        f.truncate()
            return job

    def get_job_status(self, job_id: str) -> Dict:
        return self.job_status({"id": job_id})

    def get_job_detail(self, job_id: str) -> Dict:
        return self.get_job_status(job_id)

    def get_jobs(self) -> List:
        self._assert_if_inactive()

        job_status_dir = self._jobs_dir
        if not isdir(job_status_dir):
            return {"jobs": []}

        with self._status_lock:
            jobs = []
            for job_file in os.listdir(job_status_dir):
                if isfile(job_status_dir / job_file):
                    with open(job_status_dir / job_file, encoding="utf-8") as f:
                        jobs.append(json.load(f))
            jobs.sort(key=lambda job: job["created_time"])
            return {"jobs": jobs}

    def delete_job(self, job_id: str) -> None:
        self._assert_if_inactive()

        job_status_dir = self._jobs_dir
        if not isdir(job_status_dir):
            raise JobNotFoundError(job_id)

        job_status = self.get_job_status(job_id)

        if job_status["status"] == "NA":
            raise JobNotFoundError(job_id)

        if job_status["status"] not in {"Completed", "Failed", "Cancelled"}:
            raise UnableToDeleteJobInNonTerminalStateError(job_id)

        mp_worker = self._mp_workers.get(job_id)
        if mp_worker is not None:
            try:
                mp_worker.close()
            except ValueError:
                pass
            self._mp_workers.pop(job_id, None)

        job_status_file = self._jobs_dir / job_id
        try:
            os.remove(job_status_file)
        except FileNotFoundError as e:
            raise JobNotFoundError(job_id) from e

    def cancel_job(self, job_id: str) -> None:
        self._assert_if_inactive()

        job_status_dir = self._jobs_dir
        if not isdir(job_status_dir):
            raise JobNotFoundError(job_id)

        job_status = self.get_job_status(job_id)

        if job_status["status"] == "NA":
            raise JobNotFoundError(job_id)

        if job_status["status"] in {"Completed", "Failed", "Cancelled"}:
            raise JobNotCancellableError(job_id)

        process = self._mp_workers.get(job_id)
        if process is not None and process.is_alive():
            # kill worker process
            process.terminate()
            _logger.info("Worker process %d was terminated.", process.pid)

        self.job_status(
            job_status,
            "Cancelled",
            expected_prev_status="Running",
        )

    def execute_job(self, job: Dict[str, str]) -> Dict[str, str]:
        self._assert_if_inactive()
        program_id = job["program_id"]
        if len(program_id) == 0:
            # not reached in daa_sim case since parameter values will be validated
            # by pydantic.
            self.job_status(
                job, "Failed", reason_message="No Program", reason_code=1300
            )
            return {"ok": False}

        job["usage"] = {}

        # status=Running
        self.job_status(job, "Running", create=True)

        if program_id == "sampler":
            if self._mp_enabled is True:
                ctx = mp.get_context("spawn")
                process = ctx.Process(
                    target=DAAService._run_samplerV2_job_func,
                    args=(
                        job,
                        self._samplerV2_options,
                        str(self._jobs_dir),
                        self._backends_spec,
                    ),
                )
                process.start()
                self._mp_workers[job["id"]] = process
            else:
                self._executor.submit(self.execute_samplerV2, job)
            return {"ok": True}

        if program_id == "estimator":
            if self._mp_enabled is True:
                ctx = mp.get_context("spawn")
                process = ctx.Process(
                    target=DAAService._run_estimatorV2_job_func,
                    args=(
                        job,
                        self._estimatorV2_options,
                        str(self._jobs_dir),
                        self._backends_spec,
                    ),
                )
                process.start()
                self._mp_workers[job["id"]] = process
            else:
                self._executor.submit(self.execute_estimatorV2, job)
            return {"ok": True}

        # status=Failed
        self.job_status(job, "Failed", reason_message=f"Unknown Program: {program_id}")
        return {"ok": False}

    @staticmethod
    def deserialize_input(json_str: str) -> Dict:
        return json.loads(json_str, cls=RuntimeDecoder)

    @staticmethod
    def serialize_output(obj: Any, support_qiskit: bool = True) -> str:
        if support_qiskit is True:
            return json.dumps(obj, cls=ProgramRuntimeEncoder)
        return json.dumps(obj, cls=StandardJSONEncoder)

    @staticmethod
    def qasm3_loads_hack(input_publike: str):
        circ = qasm3.loads(input_publike)
        if len(circ.cregs) == 0 and len(circ.clbits) != 0:
            cr = ClassicalRegister(name="c", bits=circ.clbits)
            circ.add_register(cr)
        return circ

    def execute_samplerV2(self, job: Dict[str, str]) -> None:
        user_logger = UserLogger(job["id"], job.get("log_level"))
        try:
            self._assert_if_inactive()

            user_logger.info("Executing %s", job["id"])
            _logger.info("Executing %s", job["id"])

            storage = job["storage"]
            backend_name = (
                job["backend"] if "backend" in job else self.default_backend_name
            )
            backend = self._get_backend(backend_name)

            job_params = storage["input"]
            input_storage_type = job_params["type"]

            input_storage = self._get_storage(input_storage_type)
            sampler_input = input_storage.get(job_params)

            sampler_input = DAAService.deserialize_input(sampler_input)

            if "pubs" not in sampler_input:
                raise ValueError(f"no pubs in this input: {sampler_input}")

            if "version" not in sampler_input:
                raise ValueError(f"no version in this input: {sampler_input}")

            if sampler_input["version"] != 2:
                raise ValueError(
                    f"For SamplerV2, version should always be 2: {sampler_input}"
                )

            options_input = (
                sampler_input["options"] if "options" in sampler_input else {}
            )
            options = {}
            for option_key, option_value in options_input.items():
                if option_key in [
                    "backend_options",
                    "run_options",
                ]:
                    # Maintains backend_options and run_options, allowing daa_sim users to configure
                    # AerSimulator with DA API job parameters.
                    options[option_key] = option_value
                else:
                    # Simply ignore other options with warning message
                    _logger.warning(
                        "Unsupported option %s=%s specified. Ignored.",
                        option_key,
                        option_value,
                    )

            options = self._samplerV2_options | options

            input_publikes = sampler_input["pubs"]
            shots = int(sampler_input.get("shots", DAAService.DEFAULT_SAMPLER_SHOTS))

            pubs = []
            for input_publike in input_publikes:
                if isinstance(input_publike, QuantumCircuit):
                    pubs.append(input_publike)
                elif isinstance(input_publike, str):
                    pubs.append(DAAService.qasm3_loads_hack(input_publike))
                elif len(input_publike) == 1:
                    if isinstance(input_publike[0], QuantumCircuit):
                        pubs.append(input_publike)
                    else:
                        pubs.append(DAAService.qasm3_loads_hack(input_publike[0]))
                elif len(input_publike) > 1:
                    pubs.append(
                        (
                            (
                                input_publike[0]
                                if isinstance(input_publike[0], QuantumCircuit)
                                else DAAService.qasm3_loads_hack(input_publike[0])
                            ),
                            input_publike[1],
                            input_publike[2] if len(input_publike) > 2 else None,
                        )
                    )

            sampler = SamplerV2.from_backend(backend, options=options)

            result = sampler.run(pubs, shots=shots).result()
            usage_nanoseconds = 0
            for pub_result in result:
                usage_nanoseconds += int(
                    pub_result.metadata["simulator_metadata"]["time_taken_execute"]
                    * 1_000_000_000
                )
            job["usage"] = {"quantum_nanoseconds": usage_nanoseconds}

            support_qiskit = sampler_input.get("support_qiskit", True)
            result_json = DAAService.serialize_output(result, support_qiskit)
            _logger.info(json.dumps(result_json, indent=2))

            results = storage["results"]
            results_storage_type = results["type"]
            results_storage = self._get_storage(results_storage_type)
            results_storage.put(results, result_json)

            # status=Completed
            self.job_status(job, "Completed", expected_prev_status="Running")

            user_logger.info("Finished %s", job["id"])
            _logger.info("Finished %s", job["id"])

        except Exception as ex:  # pylint: disable=broad-exception-caught
            _logger.exception("SamplerV2 error. job_id=%s", job["id"], exc_info=ex)
            user_logger.exception(ex)
            # status=Failed
            self.job_status(job, "Failed", reason_message=str(ex), reason_code=5203)

        finally:
            logs = storage.get("logs")
            if logs is not None:
                # if user specifies log storage location, upload the logs.
                logs_storage_type = logs["type"]
                logs_storage = self._get_storage(logs_storage_type)
                logs_storage.put(logs, user_logger.getvalue())

    def execute_estimatorV2(self, job: Dict[str, str]) -> None:
        user_logger = UserLogger(job["id"], job.get("log_level"))

        try:
            self._assert_if_inactive()

            user_logger.info("Executing %s", job["id"])
            _logger.info("Executing %s", job["id"])

            storage = job["storage"]
            backend_name = (
                job["backend"] if "backend" in job else self.default_backend_name
            )
            backend = self._get_backend(backend_name)

            job_params = storage["input"]
            input_storage_type = job_params["type"]

            input_storage = self._get_storage(input_storage_type)
            estimator_input = input_storage.get(job_params)

            estimator_input = DAAService.deserialize_input(estimator_input)

            if "pubs" not in estimator_input:
                raise ValueError(f"no pubs in this input: {estimator_input}")

            if "version" not in estimator_input:
                raise ValueError(f"no version in this input: {estimator_input}")

            if estimator_input["version"] != 2:
                raise ValueError(
                    f"For EstimatorV2, version should always be 2: {estimator_input}"
                )

            options_input = (
                estimator_input["options"] if "options" in estimator_input else {}
            )
            options = {}
            for option_key, option_value in options_input.items():
                if option_key in [
                    "default_precision",
                    "backend_options",
                    "run_options",
                ]:
                    # Of the options defined in the EstimatorV2 schema, only the "default_precision"
                    # option is supported by AerSimulator.
                    # Maintains backend_options and run_options, allowing daa_sim users to configure
                    # AerSimulator with DA API job parameters.
                    options[option_key] = option_value
                else:
                    # Simply ignore other options with warning message
                    _logger.warning(
                        "Unsupported option %s=%s specified. Ignored.",
                        option_key,
                        option_value,
                    )

            options = self._estimatorV2_options | options

            # print(json.dumps(estimator_input, indent=2, cls=RuntimeEncoder))
            input_publikes = estimator_input["pubs"]
            for input_publike in input_publikes:
                if isinstance(input_publike[0], str):
                    input_publike[0] = qasm3.loads(input_publike[0])

            estimator = EstimatorV2.from_backend(backend, options=options)

            if "precision" in estimator_input:
                result = estimator.run(
                    input_publikes, precision=estimator_input.get("precision")
                ).result()
            else:
                result = estimator.run(input_publikes).result()

            usage_nanoseconds = 0
            for pub_result in result:
                usage_nanoseconds += int(
                    pub_result.metadata["simulator_metadata"]["time_taken_execute"]
                    * 1_000_000_000
                )
            job["usage"] = {"quantum_nanoseconds": usage_nanoseconds}

            support_qiskit = estimator_input.get("support_qiskit", True)
            result_json = DAAService.serialize_output(result, support_qiskit)
            _logger.info(json.dumps(result_json, indent=2))

            results = storage["results"]
            results_storage_type = results["type"]
            results_storage = self._get_storage(results_storage_type)
            results_storage.put(results, result_json)

            # status=Completed
            self.job_status(job, "Completed", expected_prev_status="Running")

            user_logger.info("Finished %s", job["id"])
            _logger.info("Finished %s", job["id"])

        except Exception as ex:  # pylint: disable=broad-exception-caught
            _logger.exception("EstimatorV2 error. job_id=%s", job["id"], exc_info=ex)
            user_logger.exception(ex)
            # status=Failed
            self.job_status(job, "Failed", reason_message=str(ex), reason_code=5203)

        finally:
            logs = storage.get("logs")
            if logs is not None:
                # if user specifies log storage location, upload the logs.
                logs_storage_type = logs["type"]
                logs_storage = self._get_storage(logs_storage_type)
                logs_storage.put(logs, user_logger.getvalue())

    @staticmethod
    def _run_estimatorV2_job_func(
        job: Dict[str, str],
        options: Dict,
        jobs_dir: str,
        backends: list[dict[str, str]],
    ) -> None:
        """A function to run EstimatorV2 job, invoked in the spawned process.

        Note that class instance method cannot be specified to
        ProcessPoolExecutor.submit().
        (ThreadPoolExecutor.submit() can accept it though)

        Args:
            job(Dict[str, str]): Job input
        """
        pid = os.getpid()
        _logger.info("Worker pid=%d started.", pid)

        service = DAAService(
            aer_options={"estimatorV2": options}, jobs_dir=jobs_dir, backends=backends
        )

        # run estimator job
        service.execute_estimatorV2(job)

    @staticmethod
    def _run_samplerV2_job_func(
        job: Dict[str, str],
        options: Dict,
        jobs_dir: str,
        backends: list[dict[str, str]],
    ) -> None:
        """A function to run SamplerV2 job, invoked in the spawned process.

        Note that class instance method cannot be specified to
        ProcessPoolExecutor.submit().
        (ThreadPoolExecutor.submit() can accept it though)

        Args:
            job(Dict[str, str]): Job input
        """
        pid = os.getpid()
        _logger.info("Worker pid=%d started.", pid)

        service = DAAService(
            aer_options={"samplerV2": options}, jobs_dir=jobs_dir, backends=backends
        )

        # run sampler job
        service.execute_samplerV2(job)

    @staticmethod
    def load_backend(module_name: str, class_name: str):
        module = importlib.import_module(module_name)
        clazz = getattr(module, class_name)
        backend_name = clazz.backend_name

        return backend_name, clazz
