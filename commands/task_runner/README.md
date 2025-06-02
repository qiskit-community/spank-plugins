# QRMI Task Runner

QRMI Task Runner is an executable to run a quantum workload on the specified QPU resource. This is designed to be used in a Slurm job, configuration parameters such as endpoint URL and access credentials are taken from the process environment variables. Users can run a quantum workload by specifying Qiskit Primitive input for IBM Direct Access or Qiskit Runtime Service, or Pulser Sequence for Pascal Cloud. 

This executable receives SIGCONT/SIGTERM signals sent by Slurm infrastructure and cancels the running quantum workload.

## Supported OS

* Linux
  * AlmaLinux 9
  * Amazon Linux 2023
  * CentOS Stream 9
  * CentOS Stream 10
  * RedHat Enterprise Linux 8
  * RedHat Enterprise Linux 9
  * RedHat Enterprise Linux 10
  * Rocky Linux 8
  * Rocky Linux 9
  * SuSE 15
  * Ubuntu 22.04
  * Ubuntu 24.04

* macOS
  * Sequoia 15.1 or above

## Prerequisites

* Rust 1.85.1 or above


## How to build
```shell-session
. ~/.cargo/env
cargo clean
cargo build --release
```

## How to run

```shell-session
$ ./target/release/qrmi_task_runner --help
qrmi_task_runner - Command to run a QRMI task

Usage: qrmi_task_runner [OPTIONS] --qpu-name <name> --input <file>

Options:
  -q, --qpu-name <name>
          QPU resource name

  -i, --input <file>
          Input to QPU resource. Parameters to inject into the primitive for direct-access or qiskit-runtime-service QPU resource. Pulser sequence for pasqal-cloud QPU resource

      --program-id <type>
          ID of the primitive to be executed. Required for direct-access or qiskit-runtime-service QPU resource

          Possible values:
          - estimator: Estimator
          - sampler:   Sampler

      --job-runs <counts>
          Number of times the pulser sequence is repeated. Required for pasqal-cloud QPU resource

  -o, --output <file>
          Write output to <file> instead of stdout

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

### Example - IBM Direct Access or Qiskit Runtime Service

Run estimator primitive on ibm_marrakesh. Arguments `--input` and `--program-id` are required for those resource types.

```shell-session
./target/release/qrmi_task_runner --qpu-name ibm_marrakesh --input estimator_input.json --program-id estimator
```

### Example - Pasqal Cloud

Run Pulser sequence on FRESNEL. Arguments `--input` and `--job-runs` are required for this resource type.

```shell-session
./target/release/qrmi_task_runner --qpu-name FRESNEL --input sequence_input.json --job-runs 1000 FRESNEL
```

### Example - Slurm job script

* The QPU resource name specified for qrmi_task_runner must be one of those specified with the `--qpu` option. In the following example, 2 QPU resources are defined(`ibm_torino` and `ibm_marrakesh`) in the --qpu option, and one of them(`ibm_marrakesh`) is specified for qrmi_task_runner.
* The argument of qrmi_task_runner must be specified according to the resource type corresponding to that QPU resource. If `ibm_marrakesh` was defined as `qiskit-runtime-service` in `qrmi_config.json`, must specify `--input` and `--program-id`.
* The environment variables required for execution are set by the [spank_qrmi](../../plugins/spank_qrmi) and [spank_qrmi_supp](../../plugins/spank_qrmi_supp) plug-ins.

```shell-session
#!/bin/bash

#SBATCH --job-name=qrmi_job
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=1
#SBATCH --qpu=ibm_torino,ibm_marrakesh

/shared/spank-plugins/commands/task_runner/target/release/qrmi_task_runner --qpu-name ibm_marrakesh --input /shared/input/estimator_input.json --program-id estimator
```

By default, task results are output to stdout and written to the `slurm-N.out` file; if `--output <file>` is specified as qrmi_task_runner arguments, results are written to that file.


## Contributing

Regardless if you are part of the core team or an external contributor, welcome and thank you for contributing to QRMI implementations!

### Solving linting/format issues

Contributor must execute the commands below and fix any issues before submitting Pull Request.

#### Rust code
```shell-session
$ . ~/.cargo/env
$ cargo fmt --all -- --check
$ cargo clippy --all-targets -- -D warnings
```
