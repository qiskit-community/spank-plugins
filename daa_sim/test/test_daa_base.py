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

"""Tests for DAAService"""
import os
import time
from abc import ABC
import shutil
import unittest
import logging
import tempfile
import uuid
from typing import Dict
from pathlib import Path
import numpy as np
from numpy.typing import NDArray

from qiskit.primitives.containers import BitArray

from qiskit_ibm_runtime.utils.result_decoder import ResultDecoder

from direct_access_client.daa_sim.daa_service import DAAService
from direct_access_client.daa_sim.consts import PYTEST_JOBS_DIR

logging.getLogger(__name__).setLevel(logging.WARNING)


class DAASTestBase(unittest.TestCase, ABC):
    """Test cases for DAA Sampler Services"""

    @classmethod
    def setUpClass(cls):
        DAASTestBase.clean_job_dir()

    @classmethod
    def clean_job_dir(cls):
        """clean job directory"""
        try:
            shutil.rmtree(PYTEST_JOBS_DIR)
        except Exception:  # pylint: disable=broad-exception-caught
            pass
        os.mkdir(PYTEST_JOBS_DIR)

    def setUp(self):
        super().setUp()
        self.service = DAAService(
            include_opt_fields=True, multiprocess=False, jobs_dir=PYTEST_JOBS_DIR
        )
        self._storage_dir = (
            tempfile.TemporaryDirectory()  # pylint: disable=consider-using-with
        )
        self._storage_dir_name = self._storage_dir.name
        self._request_count = 0

    def tearDown(self):
        if self.service:
            self.service.close()
        self._storage_dir.cleanup()

    def _generate_job(
        self,
        input_file_path: Path,
        results_file_path: Path,
        program_id: str,
        backend: str,
    ) -> Dict:
        """generate a DAA job"""
        return {
            "id": str(uuid.uuid4()),
            "backend": backend,
            "program_id": program_id,
            "storage": {
                "input": {
                    "type": "file_system",
                    "file_path": str(input_file_path),
                },
                "results": {
                    "type": "file_system",
                    "file_path": str(results_file_path),
                },
            },
        }

    def run_daa_service(
        self,
        service_method,
        input_str: str,
        program_id: str,
        backend: str = DAAService.DEFAULT_BACKEND,
        wait_completion: bool = True,
    ):
        """run daa sampler with a list of SamplerPub"""

        dir_path = Path(self._storage_dir_name)
        self._request_count += 1
        input_file_path = dir_path / f"input-{self._request_count}.json"
        results_file_path = dir_path / f"results-{self._request_count}.json"

        with open(input_file_path, mode="w", encoding="utf-8") as input_file:
            input_file.write(input_str)

        job = self._generate_job(
            input_file_path, results_file_path, program_id, backend
        )

        # with DAAService() as service:
        service_method(job)
        if wait_completion:
            while (job_status := self.service.job_status(job))["status"] == "Running":
                time.sleep(0.5)
            if job_status["status"] != "Completed":
                details = (
                    job_status["details"]
                    if "details" in job_status
                    else f"Unknown Error: {job_status}"
                )
                raise ValueError(details)

        else:
            job_status = self.service.job_status(job)
            if job_status["status"] == "Running":
                return job, None
            elif job_status["status"] != "Completed":
                details = (
                    job_status["details"]
                    if "details" in job_status
                    else f"Unknown Error: {job_status}"
                )
                raise ValueError(details)

        with open(results_file_path, encoding="utf-8") as result_file:
            result_str = result_file.read()
            result = ResultDecoder.decode(result_str)
            return job, result

    def _assert_allclose(
        self, bitarray: BitArray, target: NDArray | BitArray, rtol=1e-1, atol=5e2
    ):
        self.assertEqual(bitarray.shape, target.shape)
        for idx in np.ndindex(bitarray.shape):
            int_counts = bitarray.get_int_counts(idx)
            target_counts = (
                target.get_int_counts(idx)
                if isinstance(target, BitArray)
                else target[idx]
            )
            max_key = max(  # pylint: disable=nested-min-max
                max(int_counts.keys()), max(target_counts.keys())
            )
            ary = np.array([int_counts.get(i, 0) for i in range(max_key + 1)])
            tgt = np.array([target_counts.get(i, 0) for i in range(max_key + 1)])
            np.testing.assert_allclose(
                ary, tgt, rtol=rtol, atol=atol, err_msg=f"index: {idx}"
            )
