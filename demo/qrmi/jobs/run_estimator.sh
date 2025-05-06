#!/bin/bash

#SBATCH --job-name=estimator_job
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=1
#SBATCH --qpu=test_heron,test_eagle
# #SBATCH --qpu=alt_marrakesh
# #SBATCH --qpu=alt_marrakesh,test_heron

# Your script goes here
source /shared/pyenv/bin/activate
srun python /shared/spank-plugins/primitives/python/examples/ibm/estimator.py
