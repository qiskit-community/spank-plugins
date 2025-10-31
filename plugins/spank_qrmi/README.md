# SPANK Plugin for QRMI

This is a [SPANK plugin](https://slurm.schedmd.com/spank.html) that configures access to Quantum Resources from user jobs. It handles the acquisition and release of access to Quantum Resources and sets the necessary environment variables for executing Quantum workloads. The available Quantum Resources are specified in the qrmi_config.json file, which is managed by the administrator.

## Prerequisites

* Compilation requires the following tools:
  * Rust compiler 1.86 or above [Link](https://www.rust-lang.org/tools/install)
  * A C compiler: for example, GCC(gcc) on Linux and Clang(clang-tools-extra) for Rust unknown targets/cross compilations. QRMI and its Spank plugin are compatible with a compiler conforming to the C11 standard.
  * make/cmake (make/cmake RPM for RHEL compatible OS
  * openssl (openssl-devel RPM for RHEL compatible OS)
  * zlib (zlib-devel RPM for RHEL compatible OS)
  * Slurm header files(slurm/slurm.h etc.) must be available on your host

* Runtime requires the following tools:
  * gcc (libgcc RPM for RHEL compatible OS)
  * openssl (openssl-libs RPM for RHEL compatible OS)
  * zlib (zlib RPM for RHEL compatible OS)


## How to build

```shell-session
. ~/.cargo/env
mkdir build
cd build
cmake ..
make
```

By default, the [CMakeLists.txt](./CMakeLists.txt) file expects the Slurm header file (`slurm.h`) to be located in `/usr/include/slurm`, but this can be customized as shown below.

```shell-session
SLURM_INCLUDE_DIRS=<directory containing slurm/slurm.h> cmake ..
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
#SBATCH --qpu=ibm_quebec,ibm_sherbrooke

# Your script goes here
source /shared/pyenv/bin/activate
srun python /shared/qrmi/examples/qiskit_primitives/ibm/sampler.py
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

> [!NOTE]
> If you are using a QPU resource with the resource type `qiskit-runtime-service`, use an account that supports [opening a session](https://quantum.cloud.ibm.com/docs/en/guides/run-jobs-session#open-a-session), such as a Premium plan.
> If you are using an account that does not support opening a session, such as an Open plan account, add `QRMI_IBM_QRS_SESSION_MODE="batch"` to the environment variable list in qrmi_config.json as workaround:

## Installation

If the above build step is successful, a Linux shared library named `spank_qrmi.so` will be created under the `build/` directory. 

In addition, add the following 1 line to the `/etc/slurm/plugstack.conf` on the nodes where this plugin is installed.

Note that administrator needs to create `qrmi_config.json` file and specify the path as plugin argument like below.

```bash
optional /usr/lib64/slurm/spank_qrmi.so /etc/slurm/qrmi_config.json
```

> [!NOTE]
> There are optional argumentis available. It allows you to add environment variables to the Slurm process where the SPANK plugin is loaded.
> ```bash
> --env:{environment variable name}={environment variable value}
> ```
>
> For example, when interacting with Quantum resources via an HTTP proxy, the environment variables `http_proxy`, `https_proxy`, and `no_proxy` are required. These can be added as shown below.
> ```bash
> optional /usr/lib64/slurm/spank_qrmi.so /etc/slurm/qrmi_config.json --env:http_proxy=http://192.168.1.128:3128 --env:https_proxy=http://192.168.1.128:3128
> ```

For allocator node, your don't need to specify the path to qrmi_config.json like below.

```bash
optional /usr/lib64/slurm/spank_qrmi.so
```


The following table shows which Slurm context nodes these files should be copied to.
| Files | Slurm contexts |
| ---- | ---- |
| `plugstack.conf` | local, remote, allocator, slurmd and job_script. For more details of each context, refer [SPANK Plugin documentation](https://slurm.schedmd.com/spank.html#SECTION_SPANK-PLUGINS) |
| `qrmi_config.json` | [remote](https://slurm.schedmd.com/spank.html#OPT_remote) (Compute nodes) |
| `spank_qrmi.so` | [allocator](https://slurm.schedmd.com/spank.html#OPT_allocator) and [remote](https://slurm.schedmd.com/spank.html#OPT_remote) (Login nodes and Compute nodes) |


> [!NOTE]
> Once plugstack.conf is updated, spank plugins will be loaded at runtime during the next job launch, which means administrators do not need to restart Slurm cluster.


Once you complete installation, you must find `--qpu=names` option in the sbatch help message.

```bash
Options provided by plugins:
      --qpu=names             Comma separated list of QPU resources to use.

```

## Logging

This plugin uses Slurm logger for logging. Log messages from this plugin can be found in /var/log/slurm/slurmd.log, etc.

```bash
[2025-07-31T09:43:34.019] [21.batch] debug:  spank: /etc/slurm/plugstack.conf:1: Loaded plugin spank_qrmi.so
[2025-07-31T09:43:34.019] [21.batch] debug:  spank_qrmi_c(6582, 0): -> slurm_spank_init argc=1 remote=1
[2025-07-31T09:43:34.019] [21.batch] debug:  SPANK: appending plugin option "qpu"
[2025-07-31T09:43:34.019] [21.batch] debug:  spank_qrmi_c(6582,0): <- slurm_spank_init rc=0
[2025-07-31T09:43:34.019] [21.batch] debug2: spank: spank_qrmi.so: init = 0
[2025-07-31T09:43:34.019] [21.batch] debug:  spank_qrmi_c: --qpu=[ibm_sherbrooke,ibm_torino]
[2025-07-31T09:43:34.019] [21.batch] debug:  spank_qrmi_c(6582, 0): -> slurm_spank_init_post_opt argc=1 remote=1
[2025-07-31T09:43:34.019] [21.batch] debug:  spank_qrmi_c, fffffffb
[2025-07-31T09:43:34.019] [21.batch] debug:  spank_qrmi_c: argv[0] = [/etc/slurm/qrmi_config.json]
[2025-07-31T09:43:34.020] [21.batch] debug:  spank_qrmi_c: name(ibm_sherbrooke), type(1) found in qrmi_config
```

You can enable QRMI runtime log by specifying the following `srun` arguments.

|  sbatch/srun option | Slurm log level (SRUN_DEBUG) | QRMI log level (RUST_LOG) |
| ---- | ---- | ---- |
| (default) | 3 | info |
| --quiet | 2 | error |
| --verbose | 4 | debug |
| -vv or more | 5 | trace |


Example:

```bash
srun -vv python /shared/qrmi/examples/qiskit_primitives/ibm/sampler.py
```


## Multiple QPU considerations

At runtime, each QRMI instance is linked to a single QPU resource. To enable the use of multiple Quantum resources within a single job script, this plugin sets environment variables with the resource name as a prefix. For example, if `--qpu=qpu1,qpu2` is specified, the environment variables will be set as follows:

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
