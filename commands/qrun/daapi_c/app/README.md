# Direct Access API for C Examples

## Prerequisites
* Python 3.11 or above

## Setup

> [!NOTE]
> The Direct Access API for C Example relies on the output of [daapi_c](../../daapi_c)]. If you have not yet built these components, please build them first as follows.
>
> ```shell-session
> pushd ../../daapi_c
> cargo build --release
> popd 
> ```

```shell-session
python3.11 -m venv ~/daapi_c_examples
source ~/daapi_c_examples/bin/activate
pip install --upgrade pip
pip install conan
conan profile detect
conan install conanfile.txt --build=missing
pushd build
cJSON_DIR=./Release/generators cmake .. -DCMAKE_BUILD_TYPE=Release
popd
```

## Build examples

Before building examples, review [config.c](./src/config.c) and update if necessary.

```shell-session
cd build
make clean
make
```

## Examples

### List backends

Source: [list_backends.c](./src/list_backends.c)

Usage:
```bash
./build/list_backends
```
Example:
```bash
./build/list_backends
```

### Cancel a job.

Source: [cancel_job.c](./src/cancel_job.c)

Usage:
```bash
./build/cancel-job <job_id>
```
Example:
```bash
./build/cancel-job 608ee5da-7a12-4cdd-a43f-b56c5bc56176
```

### Delete a job.

Source: [delete_job.c](./src/delete_job.c)

Usage:
```bash
./build/delete-job <job_id>
```
Example:
```bash
./build/delete-job 608ee5da-7a12-4cdd-a43f-b56c5bc56176
```

### List jobs.

Source: [list_jobs.c](./src/list_jobs.c)

Usage:
```bash
./build/list_jobs
```

### Run a job. 

Source: [run_job.c](./src/run_job.c)

Usage:
```bash
./build/run_job <backend_name> <program_id> <params file>
```
Example: 
```bash
./build/run_job fake_brisbane sampler ../../qiskit_pubs_gen/sampler_input.json
```

### Invoke a Qiskit Runtime primitive.

Source: [run_primitive.c](./src/run_primitive.c)

Usage:
```bash 
./build/run_primitive <backend_name> <program_id> <params file>
```
Example:
```bash
./build/run-primitive fake_cairo estimator ../../qiskit_pubs_gen/estimator_input.json
```

### S3 operations

Source: [s3.c](./src/s3.c)

Usage:
```bash 
./build/s3
```

### UUIDv4 generation

Source: [uuid.c](./src/uuid.c)

Usage:
```bash 
./build/uuid
```
