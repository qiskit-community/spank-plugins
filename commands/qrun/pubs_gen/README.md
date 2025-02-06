# Tools to generate EstimatorV2/SamplerV2 primitive input

## Prerequisites
* Python 3.11 or above


## Install dependencies

```shell-session
pip install -f requirements.txt
```

## Tools

### gen_estimator_input.py

Generates EstimatorV2 input.

Parameters:
```shell-session
base_url = "http://localhost:8290"
iam_apikey = "demoapikey1"
backend_name = "fake_brisbane"
```

Usage:
```bash
python gen_estimator_input.py
```

Output:
`estimator_input.json` will be created.

### gen_sampler_input.py

Generates SamplerV2 input.

Parameters:
```shell-session
base_url = "http://localhost:8290"
iam_apikey = "demoapikey1"
backend_name = "fake_cairo"
```

Usage:
```bash
python gen_sampler_input.py
```

Output:
`sampler_input.json` will be created.
