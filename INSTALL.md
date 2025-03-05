# Installation

This document describes how to setup development environment and the plugins developed in this project.


## Setup Local Development Environment

### Jump To:
- [Pre-requisites](#pre-requisites)
- [Creating Docker-based Slurm Cluster](#creating-docker-based-slurm-cluster)
- [Building and installing our SPANK Plugins](#building-and-installing-our-spank-plugins)
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
git clone https://github.com/giovtorres/slurm-docker-cluster.git
cd slurm-docker-cluster
```

#### 3. Cloning qiskit-community/spank-plugins

```bash
mkdir shared
pushd shared
git clone git@github.com:qiskit-community/spank-plugins.git
popd
```

#### 4. Applying a patch to slurm-docker-cluster

```bash
patch -p1 < ./shared/spank-plugins/demo/qrun/slurm-docker-cluster/file.patch
```

#### 5. Building containers

```bash
docker compose build --no-cache
```

#### 6. Creating a directory for MinIO S3 Bucket

```bash
mkdir minio
```

#### 7. Starting a cluster

```bash
docker compose up -d
```

#### 8. Creating a S3 bucket for testing

- Open http://localhost:9001 with your browser
- Login with minioadmin / minioadmin
- Click 'Create a Bucket' hyperlink in Object Browser tab
- Specify `slurm-qrun` as Bucket Name.
- Click 'Create Bucket'

#### 9. Starting Slurm Cluster

```bash
docker compose up -d
```

> [!NOTE]
> Ensure that the following seven containers are running on the PC.
>
> - daapi ([Direct Access API Simulator](./daa_sim/README.md) node)
> - c2 (Compute Node #2)
> - c1 (Compute Node #1)
> - slurmctld (Central Management Node)
> - slurmdbd (Slurm DB Node)
> - slurm-docker-cluster-minio-1 (S3 Bucket Node, used by Direct Access)
> - mysql (Database node)

Slurm Cluster is now set up as shown.

<p align="center">
  <img src="./docs/images/slurm-docker-cluster.png" width="640">
</p>


### Building and installing our SPANK Plugins


> [!NOTE]
> The following explanation assumes:
> - using Slurm Docker Cluster set up above. If you use other existing environments, do the equivalent.
> - building code on `slurmctld` node. Other nodes are also acceptable.


1. Building [QRUN](./commands/qrun/README.md)

```bash
% docker exec -it slurmctld bash

[root@slurmctld /]# source ~/.cargo/env
[root@slurmctld /]# cd /shared/spank-plugins/commands/qrun
[root@slurmctld /]# cargo build --release
```

2. Building SPANK Plugin - [skeleton](./plugins/skeleton/README.md)

```bash
[root@slurmctld /]# cd /shared/spank-plugins/plugins/skeleton
[root@slurmctld /]# mkdir build
[root@slurmctld /]# pushd build
[root@slurmctld /]# cmake ..
[root@slurmctld /]# make
[root@slurmctld /]# popd
```

3. Building SPANK Plugin - [spank_ibm_qrun](./plugins/spank_ibm_qrun/README.md)

```bash
[root@slurmctld /]# cd /shared/spank-plugins/plugins/spank_ibm_qrun
[root@slurmctld /]# mkdir build
[root@slurmctld /]# pushd build
[root@slurmctld /]# cmake ..
[root@slurmctld /]# make
[root@slurmctld /]# popd
```

4. Installing SPANK Plugins

> [!NOTE]
> The plugstack.conf file and the plugin library must be available on the node where the user executes the `sbatch` command and on the compute node where the QRUN command is executed.

Create `/etc/slurm/plugstack.conf` if not exists and add the following lines:

```bash
optional /shared/spank-plugins/plugins/skeleton/build/spank_skeleton.so
optional /shared/spank-plugins/plugins/spank_ibm_qrun/build/spank_ibm_qrun.so
```

5. Checking SPANK Plugins installation

If you complete above step, you must see additional options of `sbatch` like below.

```bash
[root@slurmctld /]# sbatch --help

Options provided by plugins:
      --skeleton-option=value Option for spank-skeleton.
      --q-backend=name        Name of Qiskit backend.
      --q-primitive=type      Qiskit primitive type(sampler or estimator).

```

6. Install QRUN Command

Login to compute nodes (`c1` and `c2` in above slurm docker cluster example).

```bash
% docker exec -it c1 bash
[root@c1 /]# ln -s /shared/spank-plugins/commands/qrun/target/release/qrun /usr/local/bin/
[root@c1 /]# exit
% docker exec -it c2 bash
[root@c2 /]# ln -s /shared/spank-plugins/commands/qrun/target/release/qrun /usr/local/bin/
[root@c2 /]# exit
```

7. Checking QRUN command

Login to compute nodes (`c1` and `c2 in above slurm docker cluster example).

```bash
[root@c1 /]# which qrun
/usr/local/bin/qrun

[root@c1 /]# qrun --help
QRUN - Command to run Qiskit Primitive jobs

Usage: qrun [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Qiskit Primitive Unified Bloc(PUB)s file

Options:
  -r, --results <RESULTS>            Result output file
      --http-timeout <HTTP_TIMEOUT>  HTTP request timeout in seconds [default: 60]
  -h, --help                         Print help
  -V, --version                      Print version
[root@c1 /]#
```

### Running examples of primitive job in Slurm Cluster

1. Loging in to slurmctld node

```bash
% docker exec -it slurmctld bash
```

2. Going to demo directory

```bash
[root@slurmctld /]# cd /shared/spank-plugins/demo/jobs
```

3. Running Sampler job

```bash
[root@slurmctld /]# sbatch run_sampler.sh
```

4. Running Estimator job

```bash
[root@slurmctld /]# sbatch run_estimator.sh
```
 
5. Checking primitive results

Once above scripts are completed, you must find `/data/sampler_output.json` and `/data/estimator_output.json` as described in above scripts.

For example,
```bash
[root@slurmctld /]# cat /data/estimator_output.json
{
  "metadata": {
    "version": 2
  },
  "results": [
    {
      "data": {
        "evs": [
          0.004016745250604636,
          0.0,
          0.0025120469911992793,
          0.0,
          0.9937272990440552,
          0.9919255461264646
        ],
        "stds": [
          0.0,
          0.0,
          0.0,
          0.0,
          0.0,
          0.0
        ]
      },
      "metadata": {
        "circuit_metadata": {},
        "simulator_metadata": {
          "max_gpu_memory_mb": 0,
          "max_memory_mb": 23995,
          "omp_enabled": true,
          "parallel_experiments": 1,
          "time_taken_execute": 0.006059958,
          "time_taken_parameter_binding": 0.000015292
        },
        "target_precision": 0.0
      }
    }
  ]
}
```

## END OF DOCUMENT
