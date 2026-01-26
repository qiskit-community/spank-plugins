# Installation

This document describes how to setup development environment and the plugins developed in this project.


## Setup Local Development Environment

### Jump To:
- [Pre-requisites](#pre-requisites)
- [Creating Docker-based Slurm Cluster](#creating-docker-based-slurm-cluster)
- [Building and installing QRMI and SPANK Plugins](#building-and-installing-qrmi-and-spank-plugins)
- [Running examples of primitive job in Slurm Cluster](#running-examples-of-primitive-job-in-slurm-cluster)


### Pre-requisites

- [Podman](https://podman.io/getting-started/installation.html) or [Docker](https://docs.docker.com/get-docker/) installed. You can use [Rancher Desktop](https://rancherdesktop.io/) instead of installing Docker on your PC.


### Creating Docker-based Slurm Cluster

You can skip below steps if you already have Slurm Cluster for development.

#### 1. Creating your workspace on your PC
```bash
mkdir -p <YOUR WORKSPACE>
cd <YOUR WORKSPACE>
```

#### 2. Cloning Slurm Docker Cluster git repository 

```bash
git clone -b 0.9.0 https://github.com/giovtorres/slurm-docker-cluster.git
cd slurm-docker-cluster
```

#### 3. Cloning qiskit-community/spank-plugins and qiskit-community/qrmi

```bash
mkdir shared
pushd shared
git clone https://github.com/qiskit-community/spank-plugins.git
git clone https://github.com/qiskit-community/qrmi.git
popd
```

#### 4. Applying a patch to slurm-docker-cluster

```bash
patch -p1 < ./shared/spank-plugins/demo/qrmi/slurm-docker-cluster/file.patch
```

Rocky Linux 9 is used as default. If you want to another  operating system, apply additional patch.

##### CentOS Stream 9

```bash
patch -p1 < ./shared/spank-plugins/demo/qrmi/slurm-docker-cluster/centos9.patch
```

##### CentOS Stream 10

```bash
patch -p1 < ./shared/spank-plugins/demo/qrmi/slurm-docker-cluster/centos10.patch
```

#### 5. Building containers

```bash
docker compose build --no-cache
```
* Need to install `docker-compose` for the `podman` users.
* For example, `brew install docker-compose` for a MAC user

#### 6. Starting a cluster

```bash
docker compose up -d
```

> [!NOTE]
> Ensure that the following 6 containers are running on the PC.
>
> - c2 (Compute Node #2)
> - c1 (Compute Node #1)
> - slurmctld (Central Management Node)
> - slurmdbd (Slurm DB Node)
> - login (Login Node)
> - mysql (Database node)

Slurm Cluster is now set up as shown.

<p align="center">
  <img src="../../../docs/images/slurm-docker-cluster.png" width="640">
</p>


### Building and installing QRMI and SPANK Plugins


> [!NOTE]
> The following explanation assumes:
> - building code on `c1` node. Other nodes are also acceptable.


1. Login to c1 container
```bash
% docker exec -it c1 bash
```

2. Creating python virtual env under shared volume

```bash
[root@c1 /]# python3.12 -m venv /shared/pyenv
[root@c1 /]# source /shared/pyenv/bin/activate
[root@c1 /]# pip install --upgrade pip
```

3. Building and installing [QRMI](https://github.com/qiskit-community/qrmi/blob/main/INSTALL.md)

```bash
[root@c1 /]# source ~/.cargo/env
[root@c1 /]# cd /shared/qrmi
[root@c1 /]# pip install -r requirements-dev.txt
[root@c1 /]# maturin build --release
[root@c1 /]# pip install /shared/qrmi/target/wheels/qrmi-*.whl
```

4. Building [SPANK Plugin](../../../plugins/spank_qrmi/README.md)

```bash
[root@c1 /]# cd /shared/spank-plugins/plugins/spank_qrmi
[root@c1 /]# mkdir build
[root@c1 /]# cd build
[root@c1 /]# cmake ..
[root@c1 /]# make
```
Which will install the QRMI from the [GitHub repo](https://github.com/qiskit-community/qrmi).

If you are building locally for development it may be easier to build the QRMI from source, mounted at `/shared/qrmi` as per this guide.
```bash
[root@c1 /]# cd /shared/spank-plugins/plugins/spank_qrmi
[root@c1 /]# mkdir build
[root@c1 /]# cd build
[root@c1 /]# cmake -DQRMI_ROOT=/shared/qrmi ..
[root@c1 /]# make
```

For pasqal-local resources make sure to build the spank plugin with munge support:
```bash
[root@c1 /]# cd /shared/spank-plugins/plugins/spank_qrmi
[root@c1 /]# mkdir build
[root@c1 /]# cd build
[root@c1 /]# cmake -DENABLE_MUNGE=ON ..
[root@c1 /]# make
```


5. Creating qrmi_config.json

Refer to [this example](https://github.com/qiskit-community/spank-plugins/blob/main/plugins/spank_qrmi/qrmi_config.json.example) and describe your environment.
Then, create a file under `/etc/slurm` or another location accessible to the Slurm daemons.

6. Installing SPANK Plugins

Create `/etc/slurm/plugstack.conf` if not exists and add the following lines:
```bash
optional /shared/spank-plugins/plugins/spank_qrmi/build/spank_qrmi.so /etc/slurm/qrmi_config.json
```

Above example assumes you create `qrmi_config.json` under `/etc/slurm` directory.

> [!NOTE]
> When you setup your own slurm cluster, `plugstack.conf`, `qrmi_config.json` and above plugin libraries need to be installed on the machines that execute slurmd (compute nodes) as well as on the machines that execute job allocation utilities such as salloc, sbatch, etc (login nodes). Refer [SPANK documentation](https://slurm.schedmd.com/spank.html#SECTION_CONFIGURATION) for more details.

7. Checking SPANK Plugins installation

If you complete above step, you must see additional options of `sbatch` like below.

```bash
[root@c1 /]# sbatch --help

Options provided by plugins:
      --qpu=names             Comma separated list of QPU resources to use.
```

### Running examples of primitive job in Slurm Cluster

1. Loging in to login node

```bash
% docker exec -it login bash
```

2. Running Sampler job

```bash
[root@login /]# sbatch /shared/spank-plugins/demo/qrmi/jobs/run_sampler.sh
```
 
3. Running Estimator job

```bash
[root@login /]# sbatch /shared/spank-plugins/demo/qrmi/jobs/run_estimator.sh
```

4. Running Pasqal job

```bash
[root@login /]# sbatch /shared/spank-plugins/demo/qrmi/jobs/run_pulser_backend.sh
```

5. Checking primitive results

Once above scripts are completed, you must find `slurm-{job_id}.out` in the current directory.

For example,
```bash
[root@login /]# cat slurm-81.out
{'backend_name': 'test_eagle'}
>>> Observable: ['IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII...',
 'IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII...',
 'IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII...',
 'IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII...',
 'IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII...',
 'IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII...',
 'IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII...',
 'IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII...',
 'IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII...',
 'IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII...',
 'IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII...',
 'IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII...',
 'IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII...',
 'IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII...',
 'IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII...', ...]
>>> Circuit ops (ISA): OrderedDict([('rz', 2724), ('sx', 1185), ('ecr', 576), ('x', 288)])
>>> Job ID: 0b1965a6-7473-4efc-aea2-6e2f1c843e5b
>>> Job Status: JobStatus.RUNNING
>>> PrimitiveResult([PubResult(data=DataBin(evs=np.ndarray(<shape=(), dtype=float64>), stds=np.ndarray(<shape=(), dtype=float64>), ensemble_standard_error=np.ndarray(<shape=(), dtype=float64>)), metadata={'shots': 4096, 'target_precision': 0.015625, 'circuit_metadata': {}, 'resilience': {}, 'num_randomizations': 32})], metadata={'dynamical_decoupling': {'enable': False, 'sequence_type': 'XX', 'extra_slack_distribution': 'middle', 'scheduling_method': 'alap'}, 'twirling': {'enable_gates': False, 'enable_measure': True, 'num_randomizations': 'auto', 'shots_per_randomization': 'auto', 'interleave_randomizations': True, 'strategy': 'active-accum'}, 'resilience': {'measure_mitigation': True, 'zne_mitigation': False, 'pec_mitigation': False}, 'version': 2})
  > Expectation value: 0.16554467382152394
  > Metadata: {'shots': 4096, 'target_precision': 0.015625, 'circuit_metadata': {}, 'resilience': {}, 'num_randomizations': 32}
```

### Running serialized jobs using the qrmi_task_runner Slurm Cluster

It is possible to run JSON-serialized jobs directly using a commandline utility called qrmi_task runner.
See [the docs](https://github.com/qiskit-community/qrmi/blob/main/bin/task_runner/README.md) for that tool for details.

```bash
[root@login /]# sbatch /shared/spank-plugins/demo/qrmi/jobs/run_task.sh
```

## END OF DOCUMENT
