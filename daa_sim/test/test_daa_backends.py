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

"""Tests for DAAService"""
import logging

from qiskit import QuantumCircuit
from qiskit.primitives import PrimitiveResult, PubResult
from qiskit.primitives.containers import BitArray
from qiskit.primitives.containers.data_bin import DataBin
from qiskit.transpiler.preset_passmanagers import generate_preset_pass_manager

from direct_access_client.daa_sim.daa_service import DAAService
from .test_daa_base import DAASTestBase
from .test_daa_sampler import generate_sampler_input

logging.getLogger(__name__).setLevel(logging.WARNING)


class DAABackendsTest(DAASTestBase):
    """Test cases for DAA Backends Services"""

    @classmethod
    def setUpClass(cls):
        DAASTestBase.clean_job_dir()

    def _sampler_run(self, inputs, backend, options=None, shots=10000):
        """run sampler"""
        if not options:
            options = {"default_shots": shots, "seed_simulator": 123}

        return self.run_daa_service(
            self.service.execute_job,
            generate_sampler_input(inputs, shots, options),
            "sampler",
            backend,
        )

    def test_backends(self):
        """Test backends"""
        backend_list = self.service.backends()
        self.assertIsNotNone(backend_list)
        self.assertIn("backends", backend_list)
        self.assertIsInstance(backend_list["backends"], list)
        self.assertTrue(len(backend_list["backends"]) > 0)
        for backend in backend_list["backends"]:
            self.assertIn("message", backend)
            self.assertIn("name", backend)
            self.assertIn("status", backend)
            self.assertIn("version", backend)

    def test_available_backends(self):
        """Test backends"""
        available_backends = self.service.available_backends()
        self.assertIsNotNone(available_backends)
        self.assertTrue(len(available_backends) > 0)
        for backend in available_backends:
            self.assertIsInstance(backend, str)

    def test_backend_details(self):
        """Test backend_details"""
        backend_list = self.service.backends()
        for backend in backend_list["backends"]:
            self.assertIn("name", backend)
            backend_name = backend["name"]
            backend_details = self.service.get_backend_details(backend_name)
            self.assertIn("message", backend_details)
            self.assertIn("name", backend_details)
            self.assertEqual(backend_name, backend_details["name"])
            self.assertIn("status", backend_details)
            self.assertIn("version", backend_details)

    def test_backend_configuration(self):
        """Test backend_configuration()."""
        backend_list = self.service.backends()
        for backend in backend_list["backends"]:
            self.assertIn("name", backend)
            backend_name = backend["name"]
            backend_config = self.service.get_backend_configuration(backend_name)
            self.assertIn("backend_name", backend_config)
            self.assertIn("backend_version", backend_config)
            self.assertIn("n_qubits", backend_config)
            self.assertIn("basis_gates", backend_config)
            self.assertIn("gates", backend_config)
            self.assertIn("local", backend_config)
            self.assertIn("simulator", backend_config)
            self.assertIn("conditional", backend_config)
            self.assertIn("memory", backend_config)
            self.assertIn("max_shots", backend_config)
            self.assertIn("qubit_lo_range", backend_config)
            self.assertIn("meas_lo_range", backend_config)
            self.assertIn("open_pulse", backend_config)
            self.assertIn("n_uchannels", backend_config)
            self.assertIn("hamiltonian", backend_config)
            self.assertIn("u_channel_lo", backend_config)
            self.assertIn("meas_lo_range", backend_config)
            self.assertIn("dt", backend_config)
            self.assertIn("dtm", backend_config)
            self.assertIn("rep_times", backend_config)
            self.assertIn("meas_kernels", backend_config)
            self.assertIn("discriminators", backend_config)

            self.assertIsInstance(backend_config["gates"], list)
            for gate in backend_config["gates"]:
                self.assertIn("name", gate)
                self.assertIsInstance(gate["name"], str)
                self.assertIn("parameters", gate)
                self.assertIsInstance(gate["parameters"], list)
                self.assertIn("qasm_def", gate)
                self.assertIsInstance(gate["qasm_def"], str)

    def test_backend_properties(self):
        """Test backend_properties"""
        backend_list = self.service.backends()
        for backend in backend_list["backends"]:
            self.assertIn("name", backend)
            backend_name = backend["name"]
            backend_props = self.service.get_backend_properties(backend_name)
            self.assertIn("backend_name", backend_props, backend)
            self.assertIn("backend_version", backend_props)
            self.assertIn("gates", backend_props)
            self.assertIn("general", backend_props, backend)
            self.assertIn("last_update_date", backend_props)
            self.assertIn("qubits", backend_props)

    def test_sampler(self):
        """test noisy backend"""
        hadamard = QuantumCircuit(2, 2, name="Bell")
        hadamard.h(0)
        hadamard.cx(0, 1)
        hadamard.measure(0, 0)
        hadamard.measure(1, 1)

        backend_list = self.service.backends()
        for backend in backend_list["backends"]:
            self.assertIn("name", backend)
            backend_name = backend["name"]
            backend_config = self.service.get_backend_configuration(backend_name)

            pm = generate_preset_pass_manager(
                optimization_level=0,
                basis_gates=backend_config["basis_gates"],
                coupling_map=backend_config["coupling_map"],
            )
            circ = pm.run(hadamard)
            _, result = self._sampler_run([(circ)], backend_name)

            self.assertIsInstance(result, PrimitiveResult)
            self.assertIsInstance(result.metadata, dict)
            self.assertEqual(len(result), 1)
            self.assertIsInstance(result[0], PubResult)
            self.assertIsInstance(result[0].data, DataBin)
            self.assertIsInstance(result[0].data.c, BitArray)
            self.assertNotEqual(
                len(result[0].data.c.get_counts()), 2
            )  # must include noise
