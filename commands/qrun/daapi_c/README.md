# Direct Access API Client for C

This C API client provides all functionalities provided by Direct Access API, and also provide some extensions for ease of use by developers like below.

* Primitive interface which
  * accepts [EstimatorV2 input](https://github.com/Qiskit/ibm-quantum-schemas/blob/main/schemas/estimator_v2_schema.json) and [SamplerV2 input](https://github.com/Qiskit/ibm-quantum-schemas/blob/main/schemas/sampler_v2_schema.json) as input, which will help existsing [IBM Qiskit Runtime REST API](https://docs.quantum.ibm.com/api/runtime) users to easily migrate.
  * wraps S3 bucket operations such as uploading job input and downloading the results
* Auto refresh of access token, allowing long job executions
* Auto job deletions after cancel
* Auto retry/reconnect after network failures

The Rust implementation is the core of the Client, and the C API provides language binding for that Rust implementation using [cbindgen](https://docs.rs/cbindgen/latest/cbindgen/).

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


## Jump To:
* [Getting Started](#getting-started)
* [Programming Guide](#programming-guide)
* [Utilities](#utilities)
* [Contributing](#contributing)


## Getting Started

### Building the SDK

#### Prerequisites

##### Rust 1.81.0 or above

Install the Rust 1.81.0 or above. This is required to build da_rust and da_cxx. This can be installed by the following:

```shell-session
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```

##### Python 3.11 or above

Install Python 3.11 or above. We recommend using Python virtual environments to cleanly separate from other applications and improve your experience.

```shell-session
$ python3.11 -m venv ~/venv
$ source ~/venv/bin/activate
```

##### Other dependencies

* For macOS, install the xcode command line tools. This is required for Apple clang and gcc/g++. If you have Brew's gcc installed on your Mac, we recommend uninstalling it to resolve compiler environment conflicts. You also must install `cmake`.

* For Linux, you must install the packages for C, C++ and Python software development. Refer each document to understand the packages to be installed. 
  * [AlmaLinux 9](https://github.com/Qiskit/direct-access-client/blob/main/docs/howtos/almalinux9.md#2-creating-dockerfile)
  * [Amazon Linux](https://github.com/Qiskit/direct-access-client/blob/main/docs/howtos/amazonlinux.md#2-creating-dockerfile)
  * [RedHat Enterprise Linux 8](https://github.com/Qiskit/direct-access-client/blob/main/docs/howtos/ubi8.md#2-creating-dockerfile)
  * [RedHat Enterprise Linux 9](https://github.com/Qiskit/direct-access-client/blob/main/docs/howtos/ubi9.md#2-creating-dockerfile)
  * [SuSE 15](https://github.com/Qiskit/direct-access-client/blob/main/docs/howtos/suse15.md#2-creating-dockerfile)
  * [Ubuntu 22.04](https://github.com/Qiskit/direct-access-client/blob/main/docs/howtos/ubuntu22.md#2-creating-dockerfile)
  * [Ubuntu 24.04](https://github.com/Qiskit/direct-access-client/blob/main/docs/howtos/ubuntu24.md#2-creating-dockerfile)
  * [Ubuntu 24.04 (Python 3.13)](https://github.com/Qiskit/direct-access-client/blob/main/docs/howtos/ubuntu24+python313.md#2-creating-dockerfile)
  
#### Building from Source
```shell-session
$ . ~/.cargo/env
$ cargo clean
$ cargo build --release
```

The build process generates the following 4 files that are used in the user's code:

| files | descriptions |
| ---- | ---- |
| direct_access_capi.h | Header file for C programs. |
| ../target/release/libdirect_access_capi.a | Static linked library for Direct Access API Client implementation. |


### Running examples

Examples are available in [this directory](./app).


## Programming Guide

### Including header files

Following 2 header files need to be included in user code.

```cpp
#include "direct_access_capi.h"
```

### Initializing API library

User code needs to call `daapi_init()` once before using the API client library to initialize static resources(logger etc.) in underlying layers. If called more than once, the second and subsequent calls are ignored.

```cpp
daapi_init();
```

### Building API client instance

A ClientBuilder can be used to create a Client with custom configuration. Direct Access API service endpoint URL is specified as constructor arguments.

```cpp
struct ClientBuilder *builder = daapi_bldr_new("http://localhost:8290");
if (!builder) {
    printf("Failed to create a builder.\n");
    return -1;
}
```

Once `builder` is created, you can customize the configuration like below.

```cpp
int rc = daapi_bldr_enable_iam_auth(
      builder, "your_api_key", "your_instance", "http://localhost:8290");
if (rc < 0)
    printf("Failed to enable IAM auth. rc=%d\n", rc);

rc = daapi_bldr_set_timeout(builder, 60.0);
if (rc < 0)
    printf("Failed to enable timeout. rc=%d\n", rc);

rc = daapi_bldr_set_exponential_backoff_retry(builder, 5, 2, 1, 10);
if (rc < 0)
    printf("Failed to enable retries. rc=%d\n", rc);

rc = daapi_bldr_set_s3_bucket(builder, "admin_user", "admin_password",
                              http://localhost:9000", "your_s3_bucket", "us-east");
if (rc < 0)
    printf("Failed to set S3 params. rc=%d\n", rc);
```

Once you specify all required configuration parameters, it is ready to build API client instance for your purpose.

```cpp
auto da_client = builder->build();
struct Client *client = daapi_cli_new(builder);
if (!client) {
    daapi_bldr_free(builder);
    return -1;
}
```

### Invoking C API

You are ready to invoke C API by using created Client instance.

```cpp
struct BackendList *backends = daapi_cli_list_backends(client);
if (backends) {
    for (size_t i = 0; i < backends->length; i++) {
        printf("%s %d\n", backends->backends[i].name,
               backends->backends[i].status);
    }
    rc = daapi_free_backend_list(backends);
    if (rc < 0)
        printf("Failed to free BackendList(%p). rc=%d\n", backends, rc);
}
```

### Releasing resources

Memory allocated by the underlying Rust implementation needs to be freed when it is no longer used by the code; the Direct Access API Client fo C provides several functions with names beginning with daapi_free_ to free these memory resources.

For example, 
```cpp
daapi_free_client(client_ptr);
daapi_free_builder(builder_ptr);
```

### Logging

Users can find the detailed runtime logs for C/Rust client by specifying `RUST_LOG` environment variable with log level.

```bash
RUST_LOG=trace ./app/build/daapi_test
```

## Utilities

Several libraries are available as open source to use Amazon S3 compatible storages from C/C++ programs. However, many of those libraries are so large and complex that they are time consuming to learn and can be a pain point for users when using the Direct Access API. Direct Access Client for C contains a S3 Client that provides the minimum functionality necessary for Direct Access API use cases.

Also, an UUIDv4 generation function is included.

Example is available in [s3.c](./app/src/s3.c) and [uuid.c](./app/src/uuid.c).


## Contributing

Regardless if you are part of the core team or an external contributor, welcome and thank you for contributing to Direct Access API Client for C!

### Solving linting/format issues

Contributor must execute the commands below and fix any issues before submitting Pull Request.

#### Rust code
```shell-session
$ cargo fmt --all -- --check
$ cargo clippy --all-targets -- -D warnings
```

#### Python code
```shell-session
$ pylint direct_access_client
$ black --check direct_access_client
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
doxygen Doxyfile
```
API document will be created under `./html` directory.
