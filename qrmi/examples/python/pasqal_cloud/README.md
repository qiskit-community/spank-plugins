# Pasqal Cloud QRMI - Examples in Python

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
| QRMI_RESOURCE_ID | Quantum backend name(e.g. `ibm_torino`) |
| QRMI_IBM_DA_ENDPOINT | Direct Access endpoint URL |


## How to run

```shell-session
$ python example.py -h
usage: example.py [-h] input backend

An example of Pasqal Cloud Python QRMI

positional arguments:
  input       primitive input file
  backend  'FRESNEL'

options:
  -h, --help  show this help message and exit
```
For example,
```shell-session
$ python example.py sampler_input.json FRESNEL
```


export AUTH_URL=https://authenticate.pasqal.cloud/authorize

curl --request POST --url $AUTH_URL --header 'content-type: application/x-www-form-urlencoded' --data grant_type=http://auth0.com/oauth/grant-type/password-realm --data realm=pcs-users --data client_id=$PASQAL_CLOUD_PROJECT_ID --data audience=https://apis.pasqal.cloud/account/api/v1 --data username=$PASQAL_CLOUD_USERNAME --data password=$PASQAL_CLOUD_PASSWORD