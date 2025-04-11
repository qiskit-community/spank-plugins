# Qiskit Runtime Service QRMI - Examples in C

## Prerequisites

* C compiler/linker, cmake and make
* [QRMI Rust library](../../../README.md)

## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below. This example assumes that a `.env` file is available under the current directory.

| Environment variables | Descriptions |
| ---- | ---- |
| QRMI_RESOURCE_ID | Quantum backend name(e.g. `ibm_torino`) |
| QRMI_IBM_QRS_ENDPOINT | Qiskit Runtime Service endpoint URL(e.g. `https://quantum.cloud.ibm.com/api`) |
| QRMI_IBM_QRS_IAM_ENDPOINT | IBM Cloud IAM endpoint URL(e.g. `https://iam.cloud.ibm.com`) |
| QRMI_IBM_QRS_IAM_APIKEY | IBM Cloud IAM API Key |
| QRMI_IBM_QRS_SERVICE_CRN | Cloud Resource Name(CRN) of the provisioned Qiskit Runtime Service instance, starting with `crn:v1:`. |
| QRMI_IBM_QRS_SESSION_MODE | Session mode, default='dedicated', batch or dedicated. |
| QRMI_IBM_QRS_TIMEOUT_SECONDS | Time (in seconds) after which job should time out and get cancelled. It is based on system execution time (not wall clock time). 
| QRMI_IBM_QRS_SESSION_ID | Session ID, set by acquire function. Optional for acquire function, however, required other functions. |

## Create Qiskit Primitive input file as input

Refer [this tool](../../../../commands/qrun/qiskit_pubs_gen) to generate. You can customize quantum circuits by editting the code.

## How to build this example

```shell-session
$ mkdir build
$ cd build
$ cmake ..
$ make
```

## How to run this example
```shell-session
$ ./build/qiskit_runtime_service
qiskit_runtime_service <primitive input file> <program id>
```
For example,
```shell-session
$ ./build/qiskit_runtime_service sampler_input.json sampler
```
