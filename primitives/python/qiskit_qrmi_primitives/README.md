# Python Implementation of Sampler/Estimator primitives

## Directory structure

| Directory | Description |
| ---- | ---- |
| [qrmi_primitives](./qrmi_primitives) | Sampler and Estimator primitive implementations based on QRMI |
| [qrmi_primitives/pulser/pasqal](./qrmi_primitives/pulser/pasqal/) | Primitives implementation for Pasqal on Pulser |
| [qrmi_primitives/qiskit/ibm](./qrmi_primitives/qiskit/ibm) | Primitives implementation for IBM Quantum services |
| [qrmi_primitives/qiskit/pasqal](./qrmi_primitives/qiskit/pasqal) | Primitives implementation for Pasqal on Qiskit |
| [examples](./examples) | Sample programs to demonstrate Sampler and Estimator primitives. |

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

Examples are available in [./examples](./examples) directory. Refer README in each sub directory.

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
