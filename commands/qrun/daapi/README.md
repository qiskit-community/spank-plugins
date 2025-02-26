# Direct Access API Client for Rust

This Rust API client provides all functionalities provided by Direct Access API, and also provide some extensions for ease of use by developers like below.

* Primitive interface which
  * accepts [EstimatorV2 input](https://github.com/Qiskit/ibm-quantum-schemas/blob/main/schemas/estimator_v2_schema.json) and [SamplerV2 input](https://github.com/Qiskit/ibm-quantum-schemas/blob/main/schemas/sampler_v2_schema.json) as input, which will help existsing [IBM Qiskit Runtime REST API](https://docs.quantum.ibm.com/api/runtime) users to easily migrate.
  * wraps S3 bucket operations such as uploading job input and downloading the results
* Auto refresh of access token, allowing long job executions
* Auto job deletions after cancel
* Auto retry/reconnect after network failures


## Supported OS

* Linux
  * AlmaLinux 9
  * Amazon Linux 2023
  * RedHat Enterprise Linux 8
  * RedHat Enterprise Linux 9
  * SuSE 15
  * Ubuntu 22.04
  * Ubuntu 24.04

* macOS
  * Sequoia 15.1 or above

## Prerequisites

* Rust 1.81.0 or above

## How to build
```shell-session
. ~/.cargo/env
cargo clean
cargo build --release
```

## Examples

Examples are available in [this directory](./app).

## API documents

Users can generate API document from source.
```shell-session
. ~/.cargo/env
cargo doc --no-deps --open
```
API document will be created under `../target/doc/direct_access_api` directory. 


## Logging

You can find the detailed runtime logs for Rust client by specifying `RUST_LOG` environment variable with log level.


```bash
RUST_LOG=trace ./bin/list_jobs
```

## Feature flags

The direct_access_api crate defines some [Cargo features](https://doc.rust-lang.org/cargo/reference/manifest.html#the-features-section) to enable using Direct Access Client for Rust in a variety of freestanding environments.

### features = ["ibmcloud_appid_auth"]

Provides the support of IBM Cloud App ID authentication. This is deprecated authentication method. This is just backward compatibility.

### features = ["internal_shared_key_auth"]

Provides the support of internal shared key authentication, which was removed from Direct Access API Beta specification. This is just backward compatibility.

### features = ["api_version"]

Allow clients to specify IBM-API-Version HTTP header value while calling Direct Access API. This was removed from Direct Access API Beta specification. This is just backward compatibility. 


## Programming Guide

### Building API client instance

A ClientBuilder can be used to create a Client with custom configuration. Direct Access API service endpoint URL is specified as constructor arguments. You can customize the configuration by calling `with_` functions.

Once you specify all required configuration parameters, you can create a Client instance by invoking `build()` function for your purpose.

```cpp
let client = ClientBuilder::new("http://localhost:8290")
    .with_auth(AuthMethod::IbmCloudIam {
        apikey: "your_iam_apikey".to_string(),
    })
    .with_timeout(Duration::from_secs(60))
    .with_retry_policy(retry_policy)
    .build()
    .unwrap();
```


### Invoking C++ API

You are ready to invoke Rust API by using created Client instance.

```cpp
  let job_id = client.run_job(&job).await?;
```

All API client related errors are delivered as Error in Result struct like other Rust functions.

## Contributing

Regardless if you are part of the core team or an external contributor, welcome and thank you for contributing to Direct Access API Client for Rust!

### Solving linting/format issues

Contributor must execute the commands below and fix any issues before submitting Pull Request.

#### Rust code
```shell-session
$ . ~/.cargo/env
$ cargo fmt --all -- --check
$ cargo clippy --all-targets -- -D warnings
```

### Running unit test

Contributor must execute the command below and fix any issues before submitting Pull Request.
```shell-session
$ . ~/.cargo/env
$ cargo test
```

### Checking API document

Contributor can generate API document from source.
```shell-session
$ . ~/.cargo/env
$ cargo doc --no-deps
```
API document will be created under `../target/doc/direct_access_api` directory.
