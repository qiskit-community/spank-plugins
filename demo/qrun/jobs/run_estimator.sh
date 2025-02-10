#!/bin/bash

#SBATCH --job-name=my_estimator_job
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=1
#SBATCH --q-backend=fake_cairo
#SBATCH --q-primitive=estimator

# Your script goes here
sbcast -f /shared/spank-plugins/demo/qrun/pubs/estimator_input.json /data/estimator_input.json
srun qrun /data/estimator_input.json
