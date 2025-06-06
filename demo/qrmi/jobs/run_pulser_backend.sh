#!/bin/bash

#SBATCH --job-name=pasqal_job
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=1
#SBATCH --qpu=FRESNEL

# Your script goes here
source /shared/pyenv/bin/activate
srun python /shared/spank-plugins/primitives/python/pulser_qrmi_backend/examples/pasqal/pulser_backend.py
