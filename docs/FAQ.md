# FAQ

## How to override environment variable values defined in qrmi_config.json

When a user submits a Slurm job, they can override the environment variable values defined in `qrmi_config.json`.

Set the value of the environment variable by prefixing its name with the resource ID, as shown below. These environment variables will be propagated by Slurm to the SPANK plugin runtime (e.g., `slurmstepd`) and to the job runtime when executing this Slurm job.

Example: Overriding `QRMI_IBM_QRS_IAM_APIKEY` and `QRMI_IBM_QRS_SERVICE_CRN` to allow a user to access ibm_brussels.

```bash
$ export ibm_brussels_QRMI_IBM_QRS_IAM_APIKEY=<api-key>
$ export ibm_brussels_QRMI_IBM_QRS_SERVICE_CRN=<crn>
$ sbatch <your job script>
```
