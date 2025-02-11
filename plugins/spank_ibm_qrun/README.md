# SPANK Plugin for QRUN

This is [SPANK plugin](https://slurm.schedmd.com/spank.html) to setup [QRUN](../../commands/qrun/README.md) command runtime by parsing --q-backend/--q-primitive options in their Slurm script and set environment variables required to run QRUN as Slurm tasks.
 
## Prerequisites

* CMake
* gcc


## How to build

```shell-session
mkdir build
cd build
cmake ..
make
```

## Installation

If the above build step is successful, a Linux shared library named `spank_ibm_qrun.so` will be created under the `build/` directory. 

SPANK plugin are loaded in up to five separate contexts during a Slurm job as described in [this page](https://slurm.schedmd.com/spank.html#SECTION_SPANK-PLUGINS). Copy this library to `/usr/lib64/slurm` directory on the nodes load this plugin.

In addition, add the following 1 line to the /etc/slurm/plugstack.conf on the nodes where this plugin is installed.

```bash
optional /usr/lib64/slurm/spank_ibm_qrun.so <APPID_CLIENT_ID> <APPID_SECRET> <DAAPI_ENDPOINT> <AWS_ACCESS_KEY_ID> <AWS_SECRET_ACCESS_KEY> <S3_ENDPOINT> <S3_BUCKET> <S3_REGION>
```

| arguments | descriptions |
| ---- | ---- |
| APPID_CLIENT_ID | IBM Cloud AppId client ID to get access token from Direct Access API (POST /v1/token). |
| APPID_SECRET | IBM Cloud AppId secret to get access token from Direct Access API (POST /v1/token). |
| DAAPI_ENDPOINT | Direct Access API endpoint URL. |
| AWS_ACCESS_KEY_ID | AWS Access Key ID to access S3 bucket used by Direct Access. |
| AWS_SECRET_ACCESS_KEY | AWS Secret Access Key to access S3 bucket used by Direct Access. |
| S3_ENDPOINT | S3 endpoint URL. |
| S3_BUCKET | Name of S3 bucket used by Direct Access. |
| S3_REGION | Name of S3 instance region. |

## Verifications

If you install this plugin correctly, q-backend and q-primitive options are appeared in the help message of `sbatch`.

```shell-session
sbatch --help

Options provided by plugins:
      --q-backend=name        Name of Qiskit backend.
      --q-primitive=type      Qiskit primitive type(sampler or estimator).

```

## Logging

This plugin uses Slurm logger for logging. Log messages from this plugin can be found in /var/log/slurm/slurmd.log, etc.

```bash
[2025-02-11T09:27:29.022] [3.batch] debug:  spank_ibm_qrun: -> slurm_spank_task_init argc=8 remote=1
[2025-02-11T09:27:29.022] [3.batch] debug:  spank_ibm_qrun: argv[0] = [demo]
[2025-02-11T09:27:29.022] [3.batch] debug:  spank_ibm_qrun: argv[1] = [demopass]
[2025-02-11T09:27:29.022] [3.batch] debug:  spank_ibm_qrun: argv[2] = [http://192.168.1.51:8290]
[2025-02-11T09:27:29.022] [3.batch] debug:  spank_ibm_qrun: argv[3] = [minioadmin]
[2025-02-11T09:27:29.022] [3.batch] debug:  spank_ibm_qrun: argv[4] = [minioadmin]
[2025-02-11T09:27:29.022] [3.batch] debug:  spank_ibm_qrun: argv[5] = [http://192.168.1.51:9000]
[2025-02-11T09:27:29.022] [3.batch] debug:  spank_ibm_qrun: argv[6] = [test]
[2025-02-11T09:27:29.022] [3.batch] debug:  spank_ibm_qrun: argv[7] = [us-east-1]
[2025-02-11T09:27:29.022] [3.batch] debug:  spank_ibm_qrun: S_JOB_UID [0]
[2025-02-11T09:27:29.023] [3.batch] debug:  spank_ibm_qrun: S_JOB_ID [3]
[2025-02-11T09:27:29.023] [3.batch] debug:  spank_ibm_qrun: S_JOB_ARGV argc=1
[2025-02-11T09:27:29.023] [3.batch] debug:  spank_ibm_qrun: job_argv[0] = [/var/spool/slurmd/job00003/slurm_script]
[2025-02-11T09:27:29.023] [3.batch] debug:  spank_ibm_qrun: setenv IBMQRUN_APPID_CLIENT_ID=demo
[2025-02-11T09:27:29.023] [3.batch] debug:  spank_ibm_qrun: setenv IBMQRUN_APPID_SECRET=demopass
[2025-02-11T09:27:29.023] [3.batch] debug:  spank_ibm_qrun: setenv IBMQRUN_DAAPI_ENDPOINT=http://192.168.1.51:8290
[2025-02-11T09:27:29.023] [3.batch] debug:  spank_ibm_qrun: setenv IBMQRUN_AWS_ACCESS_KEY_ID=minioadmin
[2025-02-11T09:27:29.023] [3.batch] debug:  spank_ibm_qrun: setenv IBMQRUN_AWS_SECRET_ACCESS_KEY=minioadmin
[2025-02-11T09:27:29.023] [3.batch] debug:  spank_ibm_qrun: setenv IBMQRUN_S3_ENDPOINT=http://192.168.1.51:9000
[2025-02-11T09:27:29.023] [3.batch] debug:  spank_ibm_qrun: setenv IBMQRUN_S3_BUCKET=test
[2025-02-11T09:27:29.023] [3.batch] debug:  spank_ibm_qrun: setenv IBMQRUN_S3_REGION=us-east-1
[2025-02-11T09:27:29.023] [3.batch] debug:  spank_ibm_qrun: <- slurm_spank_task_init rc=0
[2025-02-11T09:27:29.023] [3.batch] debug2: spank: spank_ibm_qrun.so: task_init = 0
```

## License

[GPL-3.0](https://github.com/qiskit-community/spank-plugins/blob/main/LICENSE)
