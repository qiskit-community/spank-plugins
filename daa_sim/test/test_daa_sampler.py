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
import json
import logging
from typing import Any, Dict, List
import numpy as np

from qiskit import QuantumCircuit, qasm3
from qiskit.circuit import ClassicalRegister, QuantumRegister, Parameter
from qiskit.circuit.library import real_amplitudes
from qiskit.primitives import PrimitiveResult, PubResult
from qiskit.primitives.containers import BitArray
from qiskit.primitives.containers.sampler_pub import SamplerPub, SamplerPubLike
from qiskit.primitives.containers.data_bin import DataBin

from qiskit_ibm_runtime.utils import RuntimeEncoder

from direct_access_client.daa_sim.daa_service import DAAService
from direct_access_client.daa_sim.consts import PYTEST_JOBS_DIR
from .test_daa_base import DAASTestBase

logging.getLogger(__name__).setLevel(logging.WARNING)


def generate_sampler_input(
    pub_likes: List[SamplerPubLike],
    shots: int = 1000,
    options: Dict[str, Any] = None,
    use_qasm3: bool = True,
) -> str:
    """generate input json string from SamplerPub"""
    dict_pubs = []

    for pub_like in pub_likes:
        pub = SamplerPub.coerce(pub_like)
        if use_qasm3:
            circ = qasm3.dumps(
                pub.circuit,
                disable_constants=True,
                allow_aliasing=True,
                experimental=qasm3.ExperimentalFeatures.SWITCH_CASE_V1,
            )
        else:
            circ = pub.circuit

        if len(pub.circuit.parameters) == 0:
            if pub.shots:
                dict_pubs.append((circ, None, pub.shots))
            else:
                dict_pubs.append((circ))
        else:
            param_array = pub.parameter_values.as_array(pub.circuit.parameters).tolist()
            if pub.shots:
                dict_pubs.append((circ, param_array, pub.shots))
            else:
                dict_pubs.append((circ, param_array))

    input_json = {
        "pubs": dict_pubs,
        "shots": shots,
        "options": options if options else {},
        "version": 2,
        "support_qiskit": True,
    }
    return json.dumps(input_json, cls=RuntimeEncoder)


class DAASamplerTest(DAASTestBase):
    """Test cases for DAA Sampler Services"""

    @classmethod
    def setUpClass(cls):
        DAASTestBase.clean_job_dir()

    def setUp(self):
        super().setUp()

        self._shots = 10000
        self._seed = 123

        self._cases = []
        hadamard = QuantumCircuit(1, 1, name="Hadamard")
        hadamard.h(0)
        hadamard.measure(0, 0)
        self._cases.append((hadamard, None, {0: 5000, 1: 5000}))  # case 0

        bell = QuantumCircuit(2, name="Bell")
        bell.h(0)
        bell.cx(0, 1)
        bell.measure_all()
        self._cases.append((bell, None, {0: 5000, 3: 5000}))  # case 1

        pqc = real_amplitudes(num_qubits=2, reps=2)
        pqc.measure_all()
        self._cases.append((pqc, [0] * 6, {0: 10000}))  # case 2
        self._cases.append((pqc, [1] * 6, {0: 168, 1: 3389, 2: 470, 3: 5973}))  # case 3
        self._cases.append(
            (pqc, [0, 1, 1, 2, 3, 5], {0: 1339, 1: 3534, 2: 912, 3: 4215})
        )  # case 4
        self._cases.append(
            (pqc, [1, 2, 3, 4, 5, 6], {0: 634, 1: 291, 2: 6039, 3: 3036})
        )  # case 5

        pqc2 = real_amplitudes(num_qubits=2, reps=3)
        pqc2.measure_all()
        self._cases.append(
            (pqc2, [0, 1, 2, 3, 4, 5, 6, 7], {0: 1898, 1: 6864, 2: 928, 3: 311})
        )  # case 6

    def _sampler_run(self, inputs, options=None, shots=10000, use_qasm3=True):
        """run sampler"""
        if not options:
            options = {"default_shots": shots, "seed_simulator": 123}

        _, result = self.run_daa_service(
            self.service.execute_job,
            generate_sampler_input(inputs, shots, options, use_qasm3),
            "sampler",
            self.service.default_backend_name,
        )
        return result

    def test_initialize_and_close(self):
        """test init and close"""
        service = DAAService(jobs_dir=PYTEST_JOBS_DIR)
        service.close()
        try:
            service.close()
            self.fail()
        except:  # pylint: disable=bare-except
            pass

    def test_sampler_run(self):
        """Test run()."""

        with self.subTest("single"):
            bell, _, target = self._cases[1]
            bell = self._pm.run(bell)

            result = self._sampler_run([(bell)])

            self.assertIsInstance(result, PrimitiveResult)
            self.assertIsInstance(result.metadata, dict)
            self.assertEqual(len(result), 1)
            self.assertIsInstance(result[0], PubResult)
            self.assertIsInstance(result[0].data, DataBin)
            self.assertIsInstance(result[0].data.meas, BitArray)
            self._assert_allclose(result[0].data.meas, np.array(target))

        with self.subTest("single with param"):
            pqc, param_vals, target = self._cases[2]
            pqc = self._pm.run(pqc)
            params = (param.name for param in pqc.parameters)

            result = self._sampler_run([(pqc, {params: param_vals})])

            self.assertIsInstance(result, PrimitiveResult)
            self.assertIsInstance(result.metadata, dict)
            self.assertEqual(len(result), 1)
            self.assertIsInstance(result[0], PubResult)
            self.assertIsInstance(result[0].data, DataBin)
            self.assertIsInstance(result[0].data.meas, BitArray)
            self._assert_allclose(result[0].data.meas, np.array(target))

        with self.subTest("multiple"):
            pqc, param_vals, target = self._cases[2]
            pqc = self._pm.run(pqc)
            params = (param.name for param in pqc.parameters)

            result = self._sampler_run(
                [(pqc, {params: [param_vals, param_vals, param_vals]})]
            )

            self.assertIsInstance(result, PrimitiveResult)
            self.assertIsInstance(result.metadata, dict)
            self.assertEqual(len(result), 1)
            self.assertIsInstance(result[0], PubResult)
            self.assertIsInstance(result[0].data, DataBin)
            self.assertIsInstance(result[0].data.meas, BitArray)
            self._assert_allclose(
                result[0].data.meas, np.array([target, target, target])
            )

    def test_sampler_run_multiple_times(self):
        """Test run() returns the same results if the same input is given."""
        bell, _, _ = self._cases[1]

        bell = self._pm.run(bell)
        _ = self._sampler_run([(bell)])

        result1 = self._sampler_run([(bell)])
        meas1 = result1[0].data.meas

        result2 = self._sampler_run([(bell)])
        meas2 = result2[0].data.meas

        self._assert_allclose(meas1, meas2, rtol=0)

    def test_sample_run_multiple_circuits(self):
        """Test run() with multiple circuits."""
        bell, _, target = self._cases[1]

        bell = self._pm.run(bell)
        result = self._sampler_run([(bell), (bell), (bell)])

        self.assertEqual(len(result), 3)
        self._assert_allclose(result[0].data.meas, np.array(target))
        self._assert_allclose(result[1].data.meas, np.array(target))
        self._assert_allclose(result[2].data.meas, np.array(target))

    def test_sampler_run_with_parameterized_circuits(self):
        """Test run() with parameterized circuits."""
        pqc1, param1, target1 = self._cases[4]
        pqc2, param2, target2 = self._cases[5]
        pqc3, param3, target3 = self._cases[6]

        pqc1, pqc2, pqc3 = self._pm.run([pqc1, pqc2, pqc3])
        result = self._sampler_run([(pqc1, param1), (pqc2, param2), (pqc3, param3)])

        self.assertEqual(len(result), 3)
        self._assert_allclose(result[0].data.meas, np.array(target1))
        self._assert_allclose(result[1].data.meas, np.array(target2))
        self._assert_allclose(result[2].data.meas, np.array(target3))

    def test_run_1qubit(self):
        """test for 1-qubit cases"""
        qc = QuantumCircuit(1)
        qc.measure_all()
        qc2 = QuantumCircuit(1)
        qc2.x(0)
        qc2.measure_all()

        result = self._sampler_run([(qc), (qc2)])

        self.assertEqual(len(result), 2)
        for i in range(2):
            self._assert_allclose(result[i].data.meas, np.array({i: self._shots}))

    def test_run_2qubit(self):
        """test for 2-qubit cases"""
        qc0 = QuantumCircuit(2)
        qc0.measure_all()
        qc1 = QuantumCircuit(2)
        qc1.x(0)
        qc1.measure_all()
        qc2 = QuantumCircuit(2)
        qc2.x(1)
        qc2.measure_all()
        qc3 = QuantumCircuit(2)
        qc3.x([0, 1])
        qc3.measure_all()

        result = self._sampler_run([(qc0), (qc1), (qc2), (qc3)])

        self.assertEqual(len(result), 4)
        for i in range(4):
            self._assert_allclose(result[i].data.meas, np.array({i: self._shots}))

    def test_run_single_circuit(self):
        """Test for single circuit case."""

        with self.subTest("No parameter"):
            circuit, _, target = self._cases[1]
            circuit = self._pm.run(circuit)
            param_target = [
                (None, np.array(target)),
                ({}, np.array(target)),
            ]
            for param, target in param_target:
                with self.subTest(f"{circuit.name} w/ {param}"):
                    result = self._sampler_run([(circuit, param)])
                    self.assertEqual(len(result), 1)
                    self._assert_allclose(result[0].data.meas, target)

        with self.subTest("One parameter"):
            circuit = QuantumCircuit(1, 1, name="X gate")
            param = Parameter("_x")
            circuit.ry(param, 0)
            circuit.measure(0, 0)
            circuit = self._pm.run(circuit)
            param_target = [
                ({"_x": np.pi}, np.array({1: self._shots})),
                ({param: np.pi}, np.array({1: self._shots})),
                ({"_x": np.array(np.pi)}, np.array({1: self._shots})),
                ({param: np.array(np.pi)}, np.array({1: self._shots})),
                ({"_x": [np.pi]}, np.array({1: self._shots})),
                ({param: [np.pi]}, np.array({1: self._shots})),
                ({"_x": np.array([np.pi])}, np.array({1: self._shots})),
                ({param: np.array([np.pi])}, np.array({1: self._shots})),
            ]
            for param, target in param_target:
                with self.subTest(f"{circuit.name} w/ {param}"):
                    result = self._sampler_run([(circuit, param)])
                    self.assertEqual(len(result), 1)
                    self._assert_allclose(result[0].data.c, target)

        with self.subTest("More than one parameter"):
            circuit, param, target = self._cases[3]
            circuit = self._pm.run(circuit)
            param_target = [
                (param, np.array(target)),
                (tuple(param), np.array(target)),
                (np.array(param), np.array(target)),
                ((param,), np.array([target])),
                ([param], np.array([target])),
                (np.array([param]), np.array([target])),
            ]
            for param, target in param_target:
                with self.subTest(f"{circuit.name} w/ {param}"):
                    result = self._sampler_run([(circuit, param)])
                    self.assertEqual(len(result), 1)
                    self._assert_allclose(result[0].data.meas, target)

    def test_run_reverse_meas_order(self):
        """test for sampler with reverse measurement order"""
        x = Parameter("x")
        y = Parameter("y")

        qc = QuantumCircuit(3, 3)
        qc.rx(x, 0)
        qc.rx(y, 1)
        qc.x(2)
        qc.measure(0, 2)
        qc.measure(1, 1)
        qc.measure(2, 0)

        qc = self._pm.run(qc)

        result = self._sampler_run([(qc, [0, 0]), (qc, [np.pi / 2, 0])])
        self.assertEqual(len(result), 2)

        # qc({x: 0, y: 0})
        self._assert_allclose(result[0].data.c, np.array({1: self._shots}))

        # qc({x: pi/2, y: 0})
        self._assert_allclose(
            result[1].data.c, np.array({1: self._shots / 2, 5: self._shots / 2})
        )

    def test_run_errors(self):
        """Test for errors with run method"""
        qc1 = QuantumCircuit(1)
        qc1.measure_all()
        qc2 = real_amplitudes(num_qubits=1, reps=1)
        qc2.measure_all()
        qc1, qc2 = self._pm.run([qc1, qc2])

        with self.subTest("set parameter values to a non-parameterized circuit"):
            with self.assertRaises(ValueError):
                _ = self._sampler_run([(qc1, [1e2])])
        with self.subTest("missing all parameter values for a parameterized circuit"):
            with self.assertRaises(ValueError):
                _ = self._sampler_run([qc2])
            with self.assertRaises(ValueError):
                _ = self._sampler_run([(qc2, [])])
            with self.assertRaises(ValueError):
                _ = self._sampler_run([(qc2, None)])
        with self.subTest("missing some parameter values for a parameterized circuit"):
            with self.assertRaises(ValueError):
                _ = self._sampler_run([(qc2, [1e2])])
        with self.subTest("too many parameter values for a parameterized circuit"):
            with self.assertRaises(ValueError):
                _ = self._sampler_run([(qc2, [1e2] * 100)])
        with self.subTest("negative shots, run arg"):
            with self.assertRaises(ValueError):
                _ = self._sampler_run([qc1], shots=-1)
        with self.subTest("negative shots, pub-like"):
            with self.assertRaises(ValueError):
                _ = self._sampler_run([(qc1, None, -1)])
        with self.subTest("negative shots, pub"):
            with self.assertRaises(ValueError):
                _ = self._sampler_run([SamplerPub(qc1, shots=-1)])
        with self.subTest("zero shots, run arg"):
            with self.assertRaises(ValueError):
                _ = self._sampler_run([qc1], shots=0)
        with self.subTest("zero shots, pub-like"):
            with self.assertRaises(ValueError):
                _ = self._sampler_run([(qc1, None, 0)])
        with self.subTest("zero shots, pub"):
            with self.assertRaises(ValueError):
                _ = self._sampler_run([SamplerPub(qc1, shots=0)])
        with self.subTest("missing []"):
            with self.assertRaisesRegex(
                ValueError, "An invalid Sampler pub-like was given"
            ):
                _ = self._sampler_run(qc1)
        with self.subTest("missing [] for pqc"):
            with self.assertRaisesRegex(
                ValueError, "Note that if you want to run a single pub,"
            ):
                _ = self._sampler_run((qc2, [0, 1]))

    def test_run_empty_parameter(self):
        """Test for empty parameter"""
        n = 5
        qc = QuantumCircuit(n, n - 1)
        qc.measure(range(n - 1), range(n - 1))
        qc = self._pm.run(qc)
        with self.subTest("one circuit"):
            result = self._sampler_run([qc], shots=self._shots)
            self.assertEqual(len(result), 1)
            self._assert_allclose(result[0].data.c, np.array({0: self._shots}))

        with self.subTest("two circuits"):
            result = self._sampler_run([qc, qc], shots=self._shots)
            self.assertEqual(len(result), 2)
            for i in range(2):
                self._assert_allclose(result[i].data.c, np.array({0: self._shots}))

    def test_run_with_shots_option(self):
        """test with shots option."""
        bell, _, _ = self._cases[1]
        bell = self._pm.run(bell)
        shots = 100

        with self.subTest("run arg"):
            result = self._sampler_run([bell], shots=shots)
            self.assertEqual(len(result), 1)
            self.assertEqual(result[0].data.meas.num_shots, shots)
            self.assertEqual(sum(result[0].data.meas.get_counts().values()), shots)

    def test_run_shots_result_size(self):
        """test with shots option to validate the result size"""
        n = 7  # should be less than or equal to the number of qubits of backend
        qc = QuantumCircuit(n)
        qc.h(range(n))
        qc.measure_all()
        qc = self._pm.run(qc)
        result = self._sampler_run([qc], shots=self._shots)
        self.assertEqual(len(result), 1)
        self.assertLessEqual(result[0].data.meas.num_shots, self._shots)
        self.assertEqual(sum(result[0].data.meas.get_counts().values()), self._shots)

    def test_circuit_with_multiple_cregs(self):
        """Test for circuit with multiple classical registers."""
        cases = []

        # case 1
        a = ClassicalRegister(1, "a")
        b = ClassicalRegister(2, "b")
        c = ClassicalRegister(3, "c")

        qc = QuantumCircuit(QuantumRegister(3), a, b, c)
        qc.h(range(3))
        qc.measure([0, 1, 2, 2], [0, 2, 4, 5])
        qc = self._pm.run(qc)
        target = {
            "a": {0: 5000, 1: 5000},
            "b": {0: 5000, 2: 5000},
            "c": {0: 5000, 6: 5000},
        }
        cases.append(("use all cregs", qc, target))

        # case 2
        a = ClassicalRegister(1, "a")
        b = ClassicalRegister(5, "b")
        c = ClassicalRegister(3, "c")

        qc = QuantumCircuit(QuantumRegister(3), a, b, c)
        qc.h(range(3))
        qc.measure([0, 1, 2, 2], [0, 2, 4, 5])
        qc = self._pm.run(qc)
        target = {
            "a": {0: 5000, 1: 5000},
            "b": {0: 2500, 2: 2500, 24: 2500, 26: 2500},
            "c": {0: 10000},
        }
        cases.append(("use only a and b", qc, target))

        # case 3
        a = ClassicalRegister(1, "a")
        b = ClassicalRegister(2, "b")
        c = ClassicalRegister(3, "c")

        qc = QuantumCircuit(QuantumRegister(3), a, b, c)
        qc.h(range(3))
        qc.measure(1, 5)
        qc = self._pm.run(qc)
        target = {"a": {0: 10000}, "b": {0: 10000}, "c": {0: 5000, 4: 5000}}
        cases.append(("use only c", qc, target))

        # case 4
        a = ClassicalRegister(1, "a")
        b = ClassicalRegister(2, "b")
        c = ClassicalRegister(3, "c")

        qc = QuantumCircuit(QuantumRegister(3), a, b, c)
        qc.h(range(3))
        qc.measure([0, 1, 2], [5, 5, 5])
        qc = self._pm.run(qc)
        target = {"a": {0: 10000}, "b": {0: 10000}, "c": {0: 5000, 4: 5000}}
        cases.append(("use only c multiple qubits", qc, target))

        # case 5
        a = ClassicalRegister(1, "a")
        b = ClassicalRegister(2, "b")
        c = ClassicalRegister(3, "c")

        qc = QuantumCircuit(QuantumRegister(3), a, b, c)
        qc.h(range(3))
        qc = self._pm.run(qc)
        target = {"a": {0: 10000}, "b": {0: 10000}, "c": {0: 10000}}
        cases.append(("no measure", qc, target))

        for title, qc, target in cases:
            with self.subTest(title):
                result = self._sampler_run([qc], shots=self._shots)
                self.assertEqual(len(result), 1)
                data = result[0].data
                self.assertEqual(len(data), 3)
                for creg in qc.cregs:
                    self.assertTrue(hasattr(data, creg.name))
                    self._assert_allclose(
                        getattr(data, creg.name), np.array(target[creg.name])
                    )

    def test_no_cregs(self):
        """Test that the sampler works when there are no classical register in the circuit."""
        qc = QuantumCircuit(2)
        result = self._sampler_run([qc])

        self.assertEqual(len(result), 1)
        self.assertEqual(len(result[0].data), 0)

    def test_empty_creg(self):
        """Test that the sampler works if provided a classical register with no bits."""
        # Test case for issue #12043
        q = QuantumRegister(1, "q")
        c1 = ClassicalRegister(0, "c1")
        c2 = ClassicalRegister(1, "c2")
        qc = QuantumCircuit(q, c1, c2)
        qc.h(0)
        qc.measure(0, 0)

        qc = self._pm.run(qc)
        result = self._sampler_run([qc], shots=self._shots)
        self.assertEqual(result[0].data.c1.array.shape, (self._shots, 0))

    def test_diff_shots(self):
        """Test of pubs with different shots"""
        bell, _, target = self._cases[1]
        bell = self._pm.run(bell)
        shots2 = self._shots + 2
        target2 = {k: v + 1 for k, v in target.items()}
        result = self._sampler_run([(bell, None, self._shots), (bell, None, shots2)])
        self.assertEqual(len(result), 2)
        self.assertEqual(result[0].data.meas.num_shots, self._shots)
        self._assert_allclose(result[0].data.meas, np.array(target))
        self.assertEqual(result[1].data.meas.num_shots, shots2)
        self._assert_allclose(result[1].data.meas, np.array(target2))

    def test_sampler_without_qasm3(self):
        """Test run()."""

        bell, _, target = self._cases[1]
        bell = self._pm.run(bell)

        result = self._sampler_run([(bell)], use_qasm3=False)

        self.assertIsInstance(result, PrimitiveResult)
        self.assertIsInstance(result.metadata, dict)
        self.assertEqual(len(result), 1)
        self.assertIsInstance(result[0], PubResult)
        self.assertIsInstance(result[0].data, DataBin)
        self.assertIsInstance(result[0].data.meas, BitArray)
        self._assert_allclose(result[0].data.meas, np.array(target))
