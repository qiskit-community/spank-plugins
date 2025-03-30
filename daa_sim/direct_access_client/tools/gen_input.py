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

# pylint: disable=invalid-name

"""A tool to generate input file for the specified primitive type"""
import argparse
import json
from typing import Any, Dict, List
import warnings
import boto3
from qiskit import QuantumCircuit, qasm3, transpile
from qiskit.circuit.library import QuantumVolume, real_amplitudes
from qiskit.primitives.containers.estimator_pub import EstimatorPub
from qiskit.quantum_info import SparsePauliOp
from qiskit.transpiler.preset_passmanagers import generate_preset_pass_manager
from qiskit_ibm_runtime.utils import RuntimeEncoder
from qiskit_ibm_runtime import QiskitRuntimeService, IBMBackend

warnings.filterwarnings("ignore")


def get_s3_client(
    endpoint_url: str,
    aws_access_key_id: str,
    aws_secret_access_key: str,
) -> Any:
    """Returns S3 client

    Args:
        endpoint_url(str): S3 API endpoint
        aws_access_key_id(str): S3 access key
        aws_secret_access_key(str): S3 secret access key

    Returns:
        Any: S3 Client
    """
    return boto3.client(
        "s3",
        endpoint_url=endpoint_url,
        aws_access_key_id=aws_access_key_id,
        aws_secret_access_key=aws_secret_access_key,
    )


# pylint: disable=dangerous-default-value
def generate_sampler_input(
    circs: List[QuantumCircuit],
    basis_gates: List[str] = ["rz", "sx", "ecr"],
    shots: int = 1000,
    options: Dict[str, Any] = {},  # W0102: Dangerous default value {} as argument
    backend: IBMBackend = None,
) -> str:
    """Generates JSON representation of sampler job"""
    pubs = []
    for circ in circs:
        pub = []
        if backend is not None:
            transpiled = transpile(circ, backend=backend)
        else:
            transpiled = transpile(circ, basis_gates=basis_gates)
        pub.append(
            qasm3.dumps(
                transpiled,
                disable_constants=True,
                experimental=qasm3.ExperimentalFeatures.SWITCH_CASE_V1,
            )
        )
        pubs.append(pub)

    input_json = {
        "pubs": pubs,
        "version": 2,
        "support_qiskit": False,
        "shots": shots,
        "options": options,
    }
    return json.dumps(input_json, cls=RuntimeEncoder)


def generate_estimator_input(
    circs: List[QuantumCircuit],
    hamiltonians: List[SparsePauliOp],
    parameter_lists: List[Any],
    precision: float,
) -> str:
    """Generates JSON representation of estimator job"""
    input_json = {
        "version": 2,
        "support_qiskit": False,
        "precision": precision,
        "options": {},
    }

    dict_pubs = []
    for index, circ in enumerate(circs):
        pub = EstimatorPub.coerce((circ, hamiltonians[index], parameter_lists[index]))

        qasm3_str = qasm3.dumps(
            pub.circuit,
            disable_constants=True,
            allow_aliasing=True,
            experimental=qasm3.ExperimentalFeatures.SWITCH_CASE_V1,
        )

        observables = pub.observables.tolist()
        param_array = pub.parameter_values.as_array(pub.circuit.parameters).tolist()

        if len(pub.circuit.parameters) == 0:
            if pub.precision is None:
                dict_pubs.append((qasm3_str, observables))
            else:
                dict_pubs.append((qasm3_str, observables, param_array, pub.precision))
        else:
            if pub.precision is None:
                dict_pubs.append((qasm3_str, observables, param_array))
            else:
                dict_pubs.append((qasm3_str, observables, param_array, pub.precision))

    input_json["pubs"] = dict_pubs

    return json.dumps(input_json, cls=RuntimeEncoder)


def main():
    """main"""
    parser = argparse.ArgumentParser(
        prog="gen_input.py", description="A tool to generate input job params files."
    )
    parser.add_argument(
        "s3_bucket_name",
        help="S3 bucket name",
    )
    parser.add_argument(
        "s3_endpoint_url",
        help="S3 endpoint URL",
    )
    parser.add_argument(
        "aws_access_key_id",
        help="AWS S3 access key id",
    )
    parser.add_argument(
        "aws_secret_access_key",
        help="AWS S3 secret access key",
    )
    parser.add_argument(
        "--type",
        default="sampler",
        help="Primitive type('sampler' or 'estimator'). Default: 'sampler'",
    )
    parser.add_argument(
        "--gates",
        nargs="+",
        default=["rz", "sx", "ecr"],
        help="Basis gates. Default: rz, sx, ecr",
    )
    parser.add_argument(
        "-e",
        "--expiration",
        default=3600,
        type=int,
        help="Time in seconds for the presigned URL to remain valid.",
    )
    parser.add_argument(
        "--backend",
        help="An optional backend object which can be used as the source of the default values for the basis_gates etc",
    )
    parser.add_argument(
        "--s3_object",
        help="An optional S3 object name to store an input. Default name is 'input.{type}.json'.",
    )
    args = parser.parse_args()

    if args.backend is not None:
        service = QiskitRuntimeService()
        backend = service.backend(args.backend)
    else:
        backend = None

    if args.type == "sampler":
        circ0 = QuantumVolume(10)
        circ0.measure_all()
        circ1 = QuantumVolume(10)
        circ1.measure_all()
        circ2 = QuantumVolume(10)
        circ2.measure_all()

        json_str = generate_sampler_input(
            [circ0, circ1, circ2], shots=10000, basis_gates=args.gates, backend=backend
        )

    elif args.type == "estimator":
        if backend is not None:
            pm = generate_preset_pass_manager(
                optimization_level=0,
                backend=backend,
            )
        else:
            pm = generate_preset_pass_manager(
                optimization_level=0,
                basis_gates=args.gates,
            )
        psi1 = pm.run(real_amplitudes(num_qubits=2, reps=2))
        H1 = SparsePauliOp.from_list([("II", 1), ("IZ", 2), ("XI", 3)])
        theta1 = [0, 1, 1, 2, 3, 5]

        json_str = generate_estimator_input([psi1], [H1], [theta1], 0.01)

    else:
        raise ValueError(f"Unsupported pritimitive type: {args.type}")

    s3_obj_name = (
        args.s3_object if args.s3_object is not None else f"input.{args.type}.json"
    )

    print(f"Generating {args.type} input file to {args.s3_bucket_name}/{s3_obj_name}")

    s3 = get_s3_client(
        endpoint_url=args.s3_endpoint_url,
        aws_access_key_id=args.aws_access_key_id,
        aws_secret_access_key=args.aws_secret_access_key,
    )
    s3.put_object(Bucket=args.s3_bucket_name, Key=s3_obj_name, Body=json_str)

    presigned_url = s3.generate_presigned_url(
        ClientMethod="get_object",
        Params={"Bucket": args.s3_bucket_name, "Key": s3_obj_name},
        ExpiresIn=args.expiration,
        HttpMethod="GET",
    )

    print(f"presigned url = {presigned_url}")


if __name__ == "__main__":
    main()
