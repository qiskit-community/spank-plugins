# Installation

## Setup Local Development Environment

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
> - daapi
> - c2
> - c1
> - slurmctld
> - slurmdbd
> - slurm-docker-cluster-minio-1
> - mysql

Slurm Cluster was set up as shown.

<p align="center">
  <img src="./docs/images/slurm-docker-cluster.png" width="640">
</p>


### Building and installing our SPANK Plugins

The following explanation assumes the Slurm Docker Cluster set up above. If you use other existing environments, do the equivalent.

1. Building QRUN command and SPANK Plugins

> [!NOTE]
> Following description assumes to log in to `slurmctld` node and build there. Other nodes are also acceptable.

```bash
% docker exec -it slurmctld bash

[root@slurmctld /]# source ~/.cargo/env
[root@slurmctld /]# cd /shared/spank-plugins/commands/qrun
[root@slurmctld /]# cargo build --release

[root@slurmctld /]# cd /shared/spank-plugins/plugins/skeleton
[root@slurmctld /]# mkdir build
[root@slurmctld /]# pushd build
[root@slurmctld /]# cmake ..
[root@slurmctld /]# make
[root@slurmctld /]# popd

[root@slurmctld /]# cd /shared/spank-plugins/plugins/spank_ibm_qrun
[root@slurmctld /]# mkdir build
[root@slurmctld /]# pushd build
[root@slurmctld /]# cmake ..
[root@slurmctld /]# make
[root@slurmctld /]# popd
```

2. Installing SPANK Plugins

Create `/etc/slurm/plugstack.conf` if not exists and add the following lines:

```bash
optional /shared/spank-plugins/plugins/skeleton/build/spank_skeleton.so
optional /shared/spank-plugins/plugins/spank_ibm_qrun/build/spank_ibm_qrun.so
```

> [!NOTE]
> The plugstack.conf file and the plugin library must be available on the node where the user executes the `sbatch` command and on the compute node where the QRUN command is executed.


3. Checking SPANK Plugins installation

If you complete above step, you must see additional options of `sbatch` like below.

```bash
[root@slurmctld /]# sbatch --help

Options provided by plugins:
      --skeleton-option=value Option for spank-skeleton.
      --q-backend=name        Name of Qiskit backend.
      --q-primitive=type      Qiskit primitive type(sampler or estimator).

```

4. Install QRUN Command

Login to compute nodes (`c1` and `c2 in above slurm docker cluster example).

```bash
% docker exec -it c1 bash
[root@c1 /]# ln -s /shared/spank-plugins/commands/qrun/target/release/qrun /usr/local/bin/
[root@c1 /]# exit
% docker exec -it c2 bash
[root@c2 /]# ln -s /shared/spank-plugins/commands/qrun/target/release/qrun /usr/local/bin/
[root@c2 /]# exit
```

5. Checking QRUN command

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

1. Login to slurmctld node

```bash
% docker exec -it slurmctld bash
```

2. Goto demo directory

```bash
[root@slurmctld /]# cd /shared/spank-plugins/demo/jobs
[root@slurmctld /]# sbatch run_sampler.sh
[root@slurmctld /]# sbatch run_estimator.sh
```
 
Once above scripts are completed, you must find `/data/sampler_output.json` and `/data/estimator_output.json` as described in above scripts.


## END OF DOCUMENT
