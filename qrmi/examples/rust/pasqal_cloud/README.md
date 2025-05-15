# Pasqal CLoud QRMI - Examples in Rust

## Prerequisites

* Python 3.11 or 3.12
* [QRMI Rust library](../../../README.md)

## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below. This example assumes that a `.env` file is available under the current directory.

| Environment variables | Descriptions |
| ---- | ---- |
| QRMI_IBM_QRS_ENDPOINT | Qiskit Runtime Service endpoint URL(e.g. `https://quantum.cloud.ibm.com/api`) |
| QRMI_IBM_QRS_IAM_ENDPOINT | IBM Cloud IAM endpoint URL(e.g. `https://iam.cloud.ibm.com`) |

## Create Pulser Sequence input file as input



## How to build this example

```shell-session
$ cargo clean
$ cargo build --release
```

## How to run this example
```shell-session
$ ../target/release/pasqal_cloud --help
QRMI for Pasqal Cloud - Example

Usage: qrmi-example-pasqal-cloud --backend <BACKEND> --input <INPUT>

Options:
  -b, --backend <BACKEND>        backend name
  -i, --input <INPUT>            primitive input file
  -h, --help                     Print help
  -V, --version                  Print version
```
For example,
```shell-session
$ ../target/release/qrmi-example-pasqal-cloud -b FRESNEL -i input.json
```
