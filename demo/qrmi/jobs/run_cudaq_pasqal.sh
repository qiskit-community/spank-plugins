#!/bin/bash

#SBATCH --job-name=cudaq_pasqal_job
#SBATCH --output=/data/job_%j.out
#SBATCH --error=/data/job_%j.out
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=1
#SBATCH --qpu=EMU_FREE

source /shared/pyenv/bin/activate
source /root/.cudaq/set_env.sh
export PYTHONPATH="/root/.cudaq${PYTHONPATH:+:${PYTHONPATH}}"

srun python /shared/qrmi/examples/qrmi/python/cudaq/pasqal.py
