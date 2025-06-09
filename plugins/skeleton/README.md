# Skeleton of SPANK Plugin for Study

This is [SPANK plugin](https://slurm.schedmd.com/spank.html) to implement all of SPANK interfaces and allow developers to review function call trace/arguments in Slurm log files.
 
## Prerequisites

* CMake
* gcc
* Slurm header & library
  * slurm/slurm.h must be available under /usr/include
  * libslurm.so must be available under /usr/lib64 or /usr/lib/x86_64-linux-gnu


## How to build

```shell-session
mkdir build
cd build
cmake ..
make
```

## Installation

If the above build step is successful, a Linux shared library named `spank_skeleton.so` will be created under the `build/` directory. 

SPANK plugin are loaded in up to five separate contexts during a Slurm job as described in [this page](https://slurm.schedmd.com/spank.html#SECTION_SPANK-PLUGINS). Copy this library to `/usr/lib64/slurm` directory on the nodes load this plugin.

In addition, add the following 1 line to the /etc/slurm/plugstack.conf on the nodes where this plugin is installed.

```bash
optional /usr/lib64/slurm/spank_skeleton.so <arg1> <arg2>
```

## Verifications

If you install this plugin correctly, `skeleton-option` option are appeared in the help message of `sbatch`.

```shell-session
sbatch --help

Options provided by plugins:
      --skeleton-option=value Option for spank-skeleton.

```

## Logging

This plugin uses Slurm logger for logging. Log messages from this plugin can be found in /var/log/slurm/slurmd.log, etc.

```bash
[2025-02-11T09:27:29.022] [3.batch] debug:  spank_skeleton: -> slurm_spank_task_init_privileged argc=2 remote=1
[2025-02-11T09:27:29.022] [3.batch] debug:  spank_skeleton: argv[0] = [plugin-arg1]
[2025-02-11T09:27:29.022] [3.batch] debug:  spank_skeleton: argv[1] = [plugin-arg2]
[2025-02-11T09:27:29.022] [3.batch] debug:  spank_skeleton: <- slurm_spank_task_init_privileged rc=0
[2025-02-11T09:27:29.022] [3.batch] debug2: spank: spank_skeleton.so: task_init_privileged = 0
[2025-02-11T09:27:29.022] [3.batch] debug:  spank_skeleton: -> slurm_spank_task_post_fork argc=2 remote=1
[2025-02-11T09:27:29.022] [3.batch] debug:  spank_skeleton: argv[0] = [plugin-arg1]
[2025-02-11T09:27:29.022] [3.batch] debug:  spank_skeleton: argv[1] = [plugin-arg2]
[2025-02-11T09:27:29.022] [3.batch] debug:  spank_skeleton: <- slurm_spank_task_post_fork rc=0
[2025-02-11T09:27:29.022] [3.batch] debug2: spank: spank_skeleton.so: task_post_fork = 0
```

## License

[GPL-3.0](https://github.com/qiskit-community/spank-plugins/blob/main/LICENSE)
