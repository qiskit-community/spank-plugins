# Python Implementation of Sampler/Estimator primitives

## Directory structure

| Directory | Description |
| ---- | ---- |
| [qiskit_qrmi_primitives](./qiskit_qrmi_primitives) | Qiskit Sampler and Estimator primitive implementations based on QRMI |
| [qiskit_qrmi_primitives/qiskit_qrmi_primitives/ibm](./qiskit_qrmi_primitives/qiskit_qrmi_primitives/ibm) | Qiskit Primitives implementation for IBM Quantum services |
| [qiskit_qrmi_primitives/qiskit_qrmi_primitives/pasqal](./qiskit_qrmi_primitives/qiskit_qrmi_primitives/pasqal) | Qiskit Primitives implementation for Pasqal |
| [qiskit_qrmi_primitives/examples](./qiskit_qrmi_primitives/examples) | Sample programs to demonstrate Sampler and Estimator primitives. |
| [pulser_qrmi_backend/pulser_qrmi_backend](./pulser_qrmi_backend/pulser_qrmi_backend) | `Backend` implementation for Pasqal on Pulser |
| [pulser_qrmi_backend/examples](./pulser_qrmi_backend/examples) | `Backend` example for Pasqal with Pulser |


## How to install

### Setup Python virtual environment(virtualenv)

```shell-session
python3.11 -m venv ~/py311venv_qrmi_primitives
source ~/py311venv_qrmi_primitives/bin/activate
pip install --upgrade pip
```

### Install QRMI package

```shell-session
$ pushd qrmi
$ pip install -r requirements-dev.txt
$ maturin develop --release
$ popd
```
The `qrmi` package is installed in the virtual environment. Refer the [README](../../qrmi/README.md) for more details.

```shell-session
$ pip show qrmi
Name: qrmi
Version: 0.1.0
Summary: 
Home-page: 
Author: 
Author-email: 
License: 
Location: /root/venv/lib64/python3.11/site-packages
Editable project location: /shared/spank-plugins/qrmi
Requires: 
Required-by: 
```

### Install QRMI Qiskit Primitive package
```shell-session
$ pushd primitives/python
$ pip install .
$ popd
```

The `qrmi-qiskit-primitives` package is installed in the virtual environment.

```shell-session
# pip show qrmi-primitives
Name: qrmi-qiskit-primitives
Version: 0.1.0
Summary: QRMI based implementations of Qiskit PrimitiveV2
Home-page: https://quantum.ibm.com/
Author: IBM Quantum
Author-email: qiskit@us.ibm.com
License: Apache 2.0
Location: /root/venv/lib64/python3.11/site-packages
Requires: qiskit, qiskit_ibm_runtime, qiskit_qasm3_import
Required-by: 
```

## Examples

Examples are available in [./pulser_qrmi_backend/examples](./pulser_qrmi_backend/examples) and [./qiskit_qrmi_primitives/examples](./qiskit_qrmi_primitives/examples)directory. Refer README in each sub directory.

## Contributing

Regardless if you are part of the core team or an external contributor, welcome and thank you for contributing to QRMI primitive implementations!

### Solving linting/format issues

Contributor must execute the commands below and fix any issues before submitting Pull Request.

#### Python code
```shell-session 
$ pushd primitives/python
$ pylint ./qrmi_primitives
$ black --check ./qrmi_primitives
``` 
