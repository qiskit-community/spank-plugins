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

import logging

from qiskit import QuantumCircuit
from qiskit.transpiler.preset_passmanagers import generate_preset_pass_manager

from direct_access_client.daa_sim.daa_service import DAAService
from direct_access_client.daa_sim.errors import (
    JobNotFoundError,
    JobNotCancellableError,
)
from .test_daa_base import DAASTestBase
from .test_daa_sampler import generate_sampler_input

logging.getLogger(__name__).setLevel(logging.WARNING)


class DAABackendsTest(DAASTestBase):
    """Test cases for DAA Backends Services"""

    def _sampler_run(
        self, inputs, backend, options=None, shots=10000, wait_completion=True
    ):
        """run sampler"""
        if not options:
            options = {"default_shots": shots, "seed_simulator": 123}

        return self.run_daa_service(
            self.service.execute_job,
            generate_sampler_input(inputs, shots, options),
            "sampler",
            backend,
            wait_completion,
        )

    def run_sampler(self, sync=True):
        """test noisy backend"""
        bell = QuantumCircuit(2, 2, name="Bell")
        bell.h(0)
        bell.cx(0, 1)
        bell.measure(0, 0)
        bell.measure(1, 1)

        backend_name = DAAService.DEFAULT_BACKEND
        backend_config = self.service.get_backend_configuration(backend_name)

        pm = generate_preset_pass_manager(
            optimization_level=0,
            basis_gates=backend_config["basis_gates"],
            coupling_map=backend_config["coupling_map"],
        )

        circ = pm.run(bell)
        job, _ = self._sampler_run([(circ)], backend_name, wait_completion=sync)
        return job, None

    def test_get_jobs(self):
        """check jobs"""
        DAASTestBase.clean_job_dir()

        jobs_ret = self.service.get_jobs()
        self.assertTrue("jobs" in jobs_ret)
        self.assertEqual(len(jobs_ret["jobs"]), 0)

        job_1, _ = self.run_sampler()

        jobs_ret = self.service.get_jobs()
        self.assertEqual(len(jobs_ret["jobs"]), 1)
        job_ret_1 = jobs_ret["jobs"][0]
        self.assertTrue("id" in job_ret_1)
        self.assertEqual(job_1["id"], job_ret_1["id"])

        job_2, _ = self.run_sampler()

        jobs_ret = self.service.get_jobs()
        self.assertEqual(len(jobs_ret["jobs"]), 2)
        job_ret_1 = jobs_ret["jobs"][0]
        job_ret_2 = jobs_ret["jobs"][1]
        self.assertTrue("id" in job_ret_1)
        self.assertTrue("id" in job_ret_2)
        self.assertEqual(job_1["id"], job_ret_1["id"])
        self.assertEqual(job_2["id"], job_ret_2["id"])

    def test_get_job_details(self):
        """check jobs/{job-id}"""
        DAASTestBase.clean_job_dir()

        job_1, _ = self.run_sampler()

        job_detail_1 = self.service.get_job_detail(job_1["id"])
        self.assertTrue("id" in job_detail_1)
        self.assertEqual(job_1["id"], job_detail_1["id"])

    def test_delete_job(self):
        """check delete jobs/{job-id}"""
        DAASTestBase.clean_job_dir()

        try:
            self.service.delete_job("no-such-job")
            self.fail("do not reach here")
        except JobNotFoundError:
            pass

        job_1, _ = self.run_sampler()

        self.service.delete_job(job_1["id"])

        job_status_1 = self.service.get_job_detail(job_1["id"])
        self.assertEqual(job_status_1["status"], "NA")
        try:
            self.service.delete_job(job_1["id"])
            self.fail("do not reach here")
        except JobNotFoundError:
            pass

        jobs_ret = self.service.get_jobs()
        self.assertEqual(len(jobs_ret["jobs"]), 0)

        self.run_sampler()  # job_2
        job_3, _ = self.run_sampler()
        self.run_sampler()  # job_4
        job_5, _ = self.run_sampler()

        jobs_ret = self.service.get_jobs()
        self.assertEqual(len(jobs_ret["jobs"]), 4)

        self.service.delete_job(job_3["id"])
        self.service.delete_job(job_5["id"])

        jobs_ret = self.service.get_jobs()
        self.assertEqual(len(jobs_ret["jobs"]), 2)

    def test_cancel_job(self):
        """check cancel jobs/{job-id}"""
        DAASTestBase.clean_job_dir()

        try:
            self.service.cancel_job("no-such-job")
            self.fail("do not reach here")
        except JobNotFoundError:
            pass

        job_1, _ = self.run_sampler(sync=False)

        try:
            self.service.cancel_job(job_1["id"])
        except JobNotCancellableError:
            self.fail("do not reach here")

        canceled = False
        for _ in range(100):
            try:
                job, _ = self.run_sampler(sync=False)
                self.service.cancel_job(job["id"])
                canceled = True
                break
            except ValueError:
                pass
        self.assertTrue(canceled)
