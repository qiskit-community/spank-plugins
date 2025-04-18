# Python examples for Sampler/Estimator primitives with IBM Qiskit Runtime Service QRMI

## Prerequisites

* Python 3.11 or 3.12
* [Installation of QRMI primitives python package(`qrmi-primitives`)](../../README.md)

## Install dependencies

Assuming your python virtual environment is located at `~/py311venv_qrmi_primitives/bin/activate`,

```shell-session
$ source ~/py311venv_qrmi_primitives/bin/activate
$ pip install -r requirements.txt
```

## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below. This example assumes that a `.env` file is available under the current directory.

| Environment variables | Descriptions |
| ---- | ---- |
| QRMI_RESOURCE_ID | Quantum backend name(e.g. `ibm_torino`) |
| QRMI_IBM_QRS_ENDPOINT | Qiskit Runtime Service endpoint URL(e.g. `https://quantum.cloud.ibm.com/api`) |
| QRMI_IBM_QRS_IAM_ENDPOINT | IBM Cloud IAM endpoint URL(e.g. `https://iam.cloud.ibm.com`) |
| QRMI_IBM_QRS_IAM_APIKEY | IBM Cloud IAM API Key |
| QRMI_IBM_QRS_SERVICE_CRN | Cloud Resource Name(CRN) of the provisioned Direct Access instance, starting with `crn:v1:`. |
| QRMI_IBM_QRS_TIMEOUT_SECONDS | Time (in seconds) after which job should time out and get cancelled. It is based on system execution time (not wall clock time).
| QRMI_IBM_QRS_SESSION_MODE | Session mode, default='dedicated', batch or dedicated. |
| QRMI_IBM_QRS_SESSION_ID | Session ID, set by acquire function. Optional for acquire function, however, required other functions. |
System execution time is the amount of time that the system is dedicated to processing your job. |

## How to run

### SamplerV2

```shell-session
$ python sampler.py
```

### EstimatorV2

```shell-session
$ python estimator.py
```

### SQD tutorial

[01_chemistry_hamiltonian.ipynb](./01_chemistry_hamiltonian.ipynb) is QRMI primitive port of [Improving energy estimation of a chemistry Hamiltonian with SQD](https://github.com/Qiskit/qiskit-addon-sqd/blob/main/docs/tutorials/01_chemistry_hamiltonian.ipynb). Start jupyter notebook and run all cells from beginning.
