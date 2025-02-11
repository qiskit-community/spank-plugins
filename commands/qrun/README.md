# QRUN - Slurm Command to run quantum circuits on IBM Quantum backends

QRUN is a Slurm command to run quantum circuits on IBM Quantum backends. This command accepts Qiskit [Primitive Unified Bloc(PUB)s](https://docs.quantum.ibm.com/guides/primitive-input-output#overview-of-pubs) as program argument, and run it on the specified IBM Quantum backends.

Since this command is designed to be used in a Slurm job, configuration parameters such as endpoint URL, backend name, and primitive type are taken from the process environment variables.

The program will not terminate until the quantum circuit has completed or failed to execute.

## Prerequisites

* Rust 1.84.0 or above. Refer [this page](https://www.rust-lang.org/tools/install) to install.


## Structure

| directories | descriptions |
| ---- | ---- |
| daapi | Rust implementation of Direct Access API client. This is portable & reusable library, and currently used by `qrun` command in this project. Note that same code has been used by other users and projects, so do not put this Slurm project specific implementation into this library. |
| qrun | Rust implementation of QRUN command. This command accepts Qiskit Primitive Unified Bloc(PUBs) as program argument, and run on the specified IBM Quantum backends. |
| qiskit_pubs_gen | Python implementation of the tools to generate Qiskit Primitive Unified Bloc(PUBs) as example. You can customize the quantum circuits and generate PUBs JSON file for QRUN execution. |


## How to build

```shell-session
. ~/.cargo/env
cargo build --release
```

## Installation

If the above build step is successful, an executable file named `qrun` will be created under the `./target/release/` directory. Copy this command to any directory(e.g. `/usr/local/bin`) in the PATH of the Slurm node where slurmd is running.

## How to run

```shell-session
./target/release/qrun <PUBs JSON file>
```

Example:
```shell-session
./target/release/qrun ../../demo/qrun/pubs/sampler_input.json
```

### Environment variables

| environment variable | descriptions |
| ---- | ---- |
| IBMQRUN_BACKEND | Name of Quantum backend where the specified quantum circuit runs on. |
| IBMQRUN_PRIMITIVE | Qiskit primitive type. `sampler` or `estimator`. |
| IBMQRUN_DAAPI_ENDPOINT | Direct Access API endpoint URL. |
| IBMQRUN_AWS_ACCESS_KEY_ID | AWS Access Key ID to access S3 bucket used by Direct Access. |
| IBMQRUN_AWS_SECRET_ACCESS_KEY | AWS Secret Access Key to access S3 bucket used by Direct Access. |
| IBMQRUN_S3_ENDPOINT | S3 endpoint URL. |
| IBMQRUN_S3_BUCKET | Name of S3 bucket used by Direct Access. |
| IBMQRUN_S3_REGION | Name of S3 instance region. |
| IBMQRUN_APPID_CLIENT_ID | IBM Cloud AppId client ID to get access token from Direct Access API (POST /v1/token). |
| IBMQRUN_APPID_SECRET | IBM Cloud AppId secret to get access token from Direct Access API (POST /v1/token). |


## Feature flags

The qrun & direct_access_api crate defines some [Cargo features](https://doc.rust-lang.org/cargo/reference/manifest.html#the-features-section) to enable using Direct Access Client for Rust in a variety of freestanding environments.

### features = ["ibmcloud_appid_auth"]

Provides the support of IBM Cloud App ID authentication. This is deprecated authentication method. This is just backward compatibility.

## Contributing

Regardless if you are part of the core team or an external contributor, welcome and thank you for contributing to Direct Access API Client for Rust!

### Solving linting/format issues

Contributor must execute the commands below and fix any issues before submitting Pull Request.

#### Rust code
```shell-session
$ cd crates
$ cargo fmt --all -- --check
$ cargo clippy --all-targets -- -D warnings
```

#### Python code
```shell-session
$ pylint direct_access_client
$ black --check direct_access_client
```

## License

[GPL-3.0](https://github.com/qiskit-community/spank-plugins/blob/main/LICENSE)
