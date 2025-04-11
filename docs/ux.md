HPC user experience, HPC developer experience and usage patterns
================================================================

## Content

- [Principles](#principles)
- [Connecting physical resources to slurm resoures and how to use them](#connecting-physical-resources-to-slurm-resoures-and-how-to-use-them)
  - [HPC admin scope](#hpc-admin-scope)
  - [HPC user scope](#hpc-user-scope)
  - [HPC application scope](#hpc-application-scope)
  - [Backend specifics](#backend-specifics)
    - [IBM Direct Access API](#ibm-direct-access-api)
    - [Qiskit Runtime Service](#qiskit-runtime-service)
- [Examples](#examples)
  - [Running jobs with dependencies](#running-jobs-with-dependencies)
  - [Running a job with several slurm QPU resources](#running-a-job-with-several-slurm-qpu-resources)
  - [Running primitives directly](#running-primitives-directly)
  - [Other workflow tools](#other-workflow-tools)

See [Overview](./overview.md) for a glossary of terms.

## Principles

Slurm QPU resource definitions determine what physical resources can be used by slurm jobs.
User source code should be agnostic to specific backend instances and even backend types as far as possible.
This keeps source code portable while the QPU seletion criteria are part of the resource definition (which is considered configuration as opposed to source code).
The source code does not have to take care resp. is not involved in resource reservation handling (that is done when slurm jobs are assigned QPU resources and start running, if applicable on the backend) or execution modes like sessions (these are automatically in place while the job is running, if applicable on the backend).
This makes the source code more portable between similar QPU resource types through different backend access methods (such as IBM's Direct Access API and IBM's Qiskit Runtime service through IBM Quantum Platform).
All backend types (such as IBM's Direct Access API, IBM's Qiskit Runtime service, or Pasqal's backends) follow these principles.

## Connecting physical resources to slurm resoures and how to use them

Note the exact syntax is subject to change -- this is a sketch of the UX at this time.

### HPC admin scope

HPC administrators configure, what physical resources can be provided to slurm jobs.
Note sensitivethe config could contain sensitive information and m

```
# slurm quauntum plugin configuration

# DA backend
name=da-local-backend                                                    \
url=https://da-endpoint.my-local.domain/                                 \
da_crn=crn:v1:bluemix:public:quantum-computing:us-east:a/43aac17...      \
da_apikey_file=/root/da_apikey                                           \
s3_url=https://s3.my-local.domain/...                                    \
s3_accesstoken_file=/root/s3_accesstoken

# QRS backends
name=ibm_fez                                                             \
type=qiskit-runtime-service-ibmcloud

name=ibm_marrakesh                                                       \
type=qiskit-runtime-service-ibmcloud
```

See the specific sections of the backend type for details on the parameters.

In `slurm.conf`, qpu generic resources can be assigned to some or all nodes for usage:
```
...
GresTypes=qpu,name
NodeName=node[1-5000] Gres=qpu,name:ibm_fez
...
```

### HPC user scope

HPC users submit jobs using QPU resources that are tied to slurm QPU resources.
The name attribute references what the HPC administrator has defined.
Mid-term, backend selection can be based on criteria other than a predefined name which refers to a spefific backend (e.g. by capacity and error rate qualifiers which help downselect between the defined set of backends)

Slurm qpu resources given an identifier (in this example: *my_qpu_resource*) that can be referenced by applications e.g. to distinguish between several assigned resources.

```shell
#SBATCH --time=100
#SBATCH --output=<LOGS_PATH>
#SBATCH --qpu=my_qpu_resource
#SBATCH --qpu-name=ibm_fez
#SBATCH --... # other options

srun ...
```

There might be additional environment variables required, depending on the backend type.

### HPC application scope

HPC applications refer to the slurm QPU resources assigned to the slurm job.

(Please check whether this flow clicks. We want to reference the slurm resource name, in case several resources are defined; also we ideally avoid specifying whether it's a DirectAccess or QiskitRuntimeService (Pasqal code is probably not very portable to other types at this time); also it seems we wanted to avoid a backend object and go with target instead)

```python
from qrmi import IBMDirectAccessSamplerV2 as SamplerV2

# Generate transpiler target from backend configuration & properties
target = get_target(slurm_resource_name=“my_qpu_resource”)

# The circuit and observable need to be transformed to only use instructions
# supported by the QPU (referred to as instruction set architecture (ISA) circuits).
# We'll use the transpiler to do this.
pm = generate_preset_pass_manager(
    optimization_level=1,
    target=target,
)

sampler = SamplerV2(target=target)
```

### Backend specifics
#### IBM Direct Access API
##### HPC admin scope
Configuration of Direct Access API backends (HPC admin scope) includes endpoints and credentials to the Direct Access endpoint, authentication services as well as the S3 endpoint.
Specifically, this includes:

* IBM Cloud API key for creating bearer tokens
* endpoint of Direct Access API
* S3 bucket and access details

Access credentials should not visible to HPC users or other non-privileged users on the system.
Therefore, sensitive data is put in separate files which can be access protected accordingly.

Note that all users share the same access and hence need appropriate user vetting before getting access to quantum resources.

##### HPC user scope
Execution lanes are not exposed to the HPC administrator or user directly.
Instead, mid term, there can be two different modes that HPC users can specify:

* `exclusive=true` specifies that no other jobs can use the resource at the same time. An exclusive mode job gets all execution lanes and can not run at the same time as a non-exclusive job
* `exclusive=false` allows other jobs to run in parallel. In that case, there can be as many jobs as there are execution lanes at the same time, and the job essentially only gets one lane

#### Qiskit Runtime Service
##### HPC user scope

It is expected, that users specify additional access details in environment variables.
Specifically, this includes

* Qiskit Runtime service instance (CRN, Cloud Resource Name)
* Endpoint for Qiskit Runtime (unless auto-detected from the CRN)
* API key which has access to the CRN
* S3 instance, bucket and access token/credentials for data transfers

This determines under which user and service instance the Qiskit Runtime service is used
Accordingly, IBM Quantum Platform's scheduling considers the user's and service instance's capabilities for scheduling.

At this time, users have to provide the above details (no shared cluster-wide Quantum access).

#### Pasqal

tbd.

## Examples

### Running jobs with dependencies

FIXME: show example with 1 classical job => 1 quantum job (python pseudo code)=> 1 classical job.
Main topic: show dependencies

### Running a job with several slurm QPU resources

FIXME: show example (quantum only, python, is good enough) where several backends are defined, referenced and used
Main topic: show how ids play an important role in that case

### Running primitives directly

FIXME: show example of qrun -- same SBATCH, but different executable.
Main topic: present qrun as an option
FIXME: define/finalize qrun at some time (parameters etc)

### Other workflow tools

FIXME: show how other workflow tooling could play into that