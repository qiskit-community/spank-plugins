#!/bin/bash

#SBATCH --job-name=pasqal_job
#SBATCH --output=/data/job_%j.out
#SBATCH --error=/data/job_%j.out
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=1
#SBATCH --qpu=PASQAL_LOCAL

# Your script goes here
source /shared/pyenv/bin/activate
python /shared/qrmi/examples/pulser_backend/pasqal/send_pasqal_job_local.py
