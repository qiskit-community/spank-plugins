# Tools to generate EstimatorV2/SamplerV2 primitive input

## Prerequisites
* Python 3.11 or above


## Install dependencies

```shell-session
pip install -f requirements.txt
```

## Tools

### gen_estimator_input.py

Generates EstimatorV2 input for the circuit introduced in [Getting started doc](https://docs.quantum.ibm.com/guides/get-started-with-primitives#get-started-with-estimator).

Usage:
```shell-session
usage: gen_estimator_inputs.py [-h] [--iam_url IAM_URL] backend base_url apikey crn

A tool to generate SamplerV2 input for testing

positional arguments:
  backend     Backend name
  base_url    API endpoint
  apikey      IAM API key
  crn         'Service CRN of your instance

options:
  -h, --help  show this help message and exit
  --iam_url IAM_URL  IAM endpoint
```

Example:
```bash
python gen_estimator_input.py ibm_marrakesh https://quantum.cloud.ibm.com/api <your apikey> <your instance>
```

Output:
`estimator_input_{backend name}.json` will be created.

### gen_sampler_input.py

Generates SamplerV2 input for the circuit introduced in [Getting started doc](https://docs.quantum.ibm.com/guides/get-started-with-primitives#get-started-with-sampler).

Usage:
```shell-session
usage: gen_sampler_inputs.py [-h] [--iam_url IAM_URL] backend base_url apikey crn

A tool to generate SamplerV2 input for testing

positional arguments:
  backend     Backend name
  base_url    API endpoint
  apikey      IAM API key
  crn         'Service CRN of your instance

options:
  -h, --help  show this help message and exit
  --iam_url IAM_URL  IAM endpoint
```

Example:
```bash
python gen_sampler_input.py ibm_marrakesh https://quantum.cloud.ibm.com/api <your apikey> <your instance>
```

Output:
`sampler_input_{backend name}.json` will be created.
