#!/bin/bash

#SBATCH --job-name=sampler_job
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=1
#SBATCH --q-backend=fake_brisbane
#SBATCH --q-primitive=sampler

# Your script goes here
sbcast -f /shared/spank-plugins/demo/qrun/pubs/sampler_input.json /data/sampler_input.json
srun qrun /data/sampler_input.json --results /data/sampler_output.json
