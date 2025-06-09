# SPANK Plugin for QRMI

This is a [SPANK plugin](https://slurm.schedmd.com/spank.html) that configures access to Quantum Resources from user jobs. It handles the acquisition and release of access to Quantum Resources and sets the necessary environment variables for executing Quantum workloads. The available Quantum Resources are specified in the qrmi_config.json file, which is managed by the administrator.
 
## Prerequisites

* Rust 1.85.1 or above
* Slurm header & library
  * slurm/slurm.h must be available under /usr/include
  * libslurm.so must be available under /usr/lib64 or /usr/lib/x86_64-linux-gnu
* you'll also need OpenSSL (libssl-dev or openssl-devel on most Unix distributions).

## How to build

```shell-session
. ~/.cargo/env
cargo clean
cargo build --release
```

## SBATCH option

This SPANK plugin registers the following option. Slurm user can specify which Quantum Resources are used for the Slurm job script.

```bash
--qpu=names             Comma separated list of QPU resources to use.
```

For example,
```bash
#!/bin/bash

#SBATCH --job-name=sampler_job
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=1
#SBATCH --qpu=test_heron,test_eagle

# Your script goes here
source /shared/pyenv/bin/activate
srun python /shared/job_scripts/sampler.py
```

## Configuring available Quantum Resources

Refer [qrmi_config.json.example](./qrmi_config.json.example) as example.

The `resources` array contains a set of available Quantum Resources which can be used by Slurm users in the jobs. Each Quantum Resource definition contains:

| properties | descriptions |
| ---- | ---- |
| name | Quantum resource name. e.g. Quantum backend name. |
| type | Resource type (`direct-access`, `qiskit-runtime-service` and `pasqal-cloud`) |
| environment | A set of environment variables to work with QRMI. Current implementations assume API endpoint and credentials are specified via environment variable setting. |

If a user specifies a resource with the --qpu option that is not defined in the qrmi_config.json file, the specification will be ignored.

If the user sets the necessary environment variables for job execution themselves, it is not required to specify them in this file. In this case, the environment property will be `{}`.

## Installation

If the above build step is successful, a Linux shared library named `libspank_qrmi.so` will be created under the `target/release/` directory. 

SPANK plugin are loaded in up to five separate contexts during a Slurm job as described in [this page](https://slurm.schedmd.com/spank.html#SECTION_SPANK-PLUGINS). Copy this library to `/usr/lib64/slurm` directory on the nodes load this plugin.

In addition, add the following 1 line to the /etc/slurm/plugstack.conf on the nodes where this plugin is installed.

Note that administrator needs to create qrmi_config.json file and specify the path as plugin argument like below.

```bash
optional /usr/lib64/slurm/libspank_qrmi.so /etc/slurm/qrmi_config.json
```

Once you complete installation, you must find `--qpu=names` option in the sbatch help message.

```bash
Options provided by plugins:
      --qpu=names             Comma separated list of QPU resources to use.

```

## Logging

This plugin uses Slurm logger for logging. Log messages from this plugin can be found in /var/log/slurm/slurmd.log, etc.

```bash
[2025-05-06T02:48:52.038] [81.batch] info: spank{id="spank_qrmi" cb="slurm_spank_init_post_opt" ctx="Remote"}: qpu = test_heron, type = IBMDirectAccess env = {
[2025-05-06T02:48:52.062] [81.batch] info: spank{id="spank_qrmi" cb="slurm_spank_init_post_opt" ctx="Remote"}: acquisition token = f3ff05d6-3cd6-402e-b242-fa8d13b9c46e
[2025-05-06T02:48:52.063] [81.batch] info: spank{id="spank_qrmi" cb="slurm_spank_init_post_opt" ctx="Remote"}: qpu = test_eagle, type = IBMDirectAccess env = {
[2025-05-06T02:48:52.082] [81.batch] info: spank{id="spank_qrmi" cb="slurm_spank_init_post_opt" ctx="Remote"}: acquisition token = 48a70138-f8eb-4bf9-ab90-46caa6584e9b
```

## Multiple QPU considerations

At runtime, each QRMI instance is linked to a single QPU resource. To enable the use of multiple Quantum resources within a single job script, this plugin sets environment variables with the resource name as a prefix. For example, if `--qpu=qpu1,qpu2 is specified, the environment variables will be set as follows:

```bash
qpu1_QRMI_IBM_DA_ENDPOINT=http://test1
qpu2_QRMI_IBM_DA_ENDPOINT=http://test2
```

This ensures that each QRMI instance operates with the configuration parameters set for its respective resource during the execution of the Slurm job.

The above environment variable settings are applied only to jobs where the `--qpu=names` option is specified.

This plugin also set the following 2 environment variables which will be referred by QRMI primitives code.

| environment varilables | descriptions |
| ---- | ---- |
| SLURM_JOB_QPU_RESOURCES | Comma separated list of QPU resources to use at runtime. Undocumented resources will be filtered out. For example, `qpu1,qpu2`. |
| SLURM_JOB_QPU_TYPES | Comma separated list of Resource type (`direct-access`, `qiskit-runtime-service` and `pasqal-cloud`). For example, `direct-access,direct-access` |

## License

[GPL-3.0](https://github.com/qiskit-community/spank-plugins/blob/main/LICENSE)
