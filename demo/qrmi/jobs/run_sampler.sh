#!/bin/bash

#SBATCH --job-name=sampler_job
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=1
#SBATCH --qpu=test_heron,test_eagle
# #SBATCH --qpu=alt_marrakesh
# #SBATCH --qpu=alt_marrakesh,test_heron

# Your script goes here
source /shared/pyenv/bin/activate
srun python /shared/spank-plugins/primitives/python/qiskit_qrmi_primitives/examples/ibm/sampler.py
