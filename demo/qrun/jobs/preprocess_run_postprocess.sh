#!/bin/bash

#SBATCH --job-name=preprocess_run_postprocess_job
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=1
#SBATCH --q-backend=fake_brisbane
#SBATCH --q-primitive=sampler

# Your script goes here
source ~/venv/bin/activate
srun sampler_preprocessing.py /data/sampler_input.json
srun qrun /data/sampler_input.json --results /data/sampler_output.json
srun sampler_postprocessing.py /data/sampler_output.json /data/counts.json
