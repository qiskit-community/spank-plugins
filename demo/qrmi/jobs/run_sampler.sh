#!/bin/bash

#SBATCH --job-name=qiskit_sampler_job
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=1
#SBATCH --qpu=ibm_sherbrooke,ibm_brisbane

# Your script goes here
source /shared/pyenv/bin/activate
srun python /shared/qrmi/examples/qiskit_primitives/ibm/sampler.py
