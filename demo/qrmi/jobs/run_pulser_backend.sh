#!/bin/bash

#SBATCH --job-name=pasqal_job
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=1
#SBATCH --qpu=FRESNEL

# Your script goes here
source /shared/pyenv/bin/activate
srun python /shared/qrmi/examples/pulser_backend/pulser_backend.py
