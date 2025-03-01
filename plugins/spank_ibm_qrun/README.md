# SPANK Plugin for QRUN

This is [SPANK plugin](https://slurm.schedmd.com/spank.html) to setup [QRUN](../../commands/qrun/README.md) command runtime by parsing --q-backend/--q-primitive options in their Slurm script and set environment variables required to run QRUN as Slurm tasks.
 
## Prerequisites

* CMake
* gcc


## How to build

> [!NOTE]
> This plugin build depends on [daapi_c](../../commands/qrun/daapi_c) build output. Before starting to build this plugin, make sure you have built daapi_c.

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
optional /usr/lib64/slurm/spank_ibm_qrun.so
```

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
[2025-02-14T05:32:39.068] [11.0] debug:  spank: /etc/slurm/plugstack.conf:2: Loaded plugin spank_ibm_qrun.so
[2025-02-14T05:32:39.069] [11.0] debug:  spank_ibm_qrun: -> slurm_spank_init argc=0
[2025-02-14T05:32:39.069] [11.0] debug:  SPANK: appending plugin option "q-backend"
[2025-02-14T05:32:39.069] [11.0] debug:  SPANK: appending plugin option "q-primitive"
[2025-02-14T05:32:39.069] [11.0] debug:  spank_ibm_qrun Is slurm_spank_task_init() supported ? 1
[2025-02-14T05:32:39.069] [11.0] debug:  spank_ibm_qrun Is slurm_spank_task_exit() supported ? 0
[2025-02-14T05:32:39.069] [11.0] debug:  spank_ibm_qrun <- slurm_spank_init rc=0
```

## License

[GPL-3.0](https://github.com/qiskit-community/spank-plugins/blob/main/LICENSE)
