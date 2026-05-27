#!/bin/bash

#SBATCH --job-name=qiskit_iqm
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=1
#SBATCH --qpu=garnet_mock

# Your script goes here
source /shared/pyenv_iqm/bin/activate
srun python /shared/qrmi/examples/qiskit_primitives/iqm/iqm_example.py
