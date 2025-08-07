#!/bin/bash

#SBATCH --job-name=qiskit_primitive_input_job
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=1
#SBATCH --qpu=ibm_sherbrooke

# Your script goes here
source /shared/pyenv/bin/activate
srun task_runner ibm_sherbrooke /shared/qrmi/bin/task_runner/examples/qiskit/estimator_input_ibm_sherbrooke.json
