# IBM Qiskit Runtime Service QRMI - Examples in Python

## Prerequisites

* Rust 1.85.1 or above
* Python 3.11 or 3.12
* [QRMI python package installation](../../../README.md)

## Install dependencies

```shell-session
$ source ~/py311_qrmi_venv/bin/activate
$ pip install -r ../requirements.txt
```

## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below. This example assumes that a `.env` file is available under the current directory.

| Environment variables | Descriptions |
| ---- | ---- |
| RESOURCE_ID | Quantum backend name(e.g. `ibm_torino`) |
| SESSION_HUB | Hub name |
| SESSION_GROUP | Group Name |
| SESSION_PROJECT | Project Name |
| IAM_APIKEY | API Key. |
| SESSION_MAX_TTL | Time after which session should time out and get cancelled |
| SESSION_MODE | Session mode name, default is dedicated |

## How to run

```shell-session

$ bash prolog.sh
$ python example.py
$ bash epilog.sh

An example of IBM Qiskit Runtime Service QRMI
```