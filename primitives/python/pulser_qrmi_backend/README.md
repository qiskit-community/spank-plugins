# Python Implementation of Pulser RemoteBackend

## Directory structure

| Directory | Description |
| ---- | ---- |
| [pulser_qrmi_primitives](./pulser_qrmi_primitives) | Pulser Backend implementations based on QRMI for PasqalCloud |
| [./examples/pasqal](./examples/pasqal) | Sample programs to demonstrate Pulser Backend. |


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

### Install QRMI pulser Primitive package
```shell-session
$ pushd primitives/python
$ pip install .
$ popd
```

The `pulser-qrmi-backend` package is installed in the virtual environment.

```shell-session
# pip show pulser-qrmi-backend
Name: pulser-qrmi-backend
Version: 0.1.0
Summary: QRMI based implementations of pulser backend
Home-page: https://pasqal.com
Author: PASQAL SAS
Author-email: contact@pasqal.com
License: Apache 2.0
Location: /root/venv/lib64/python3.11/site-packages
Requires: pulser
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
$ pylint .
$ black --check .
``` 
