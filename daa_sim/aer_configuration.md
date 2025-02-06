# Configuring AerSimulator

You can specify `backend_options` and `run_options` for AerSimulator in two ways like below.
For more details of AerSimulator options, refer [this document](https://qiskit.github.io/qiskit-aer/stubs/qiskit_aer.AerSimulator.html#).

## Specify those options in your Job parameter

When you create Job parameter JSON to be stored to S3 compatible storage, you can specify `backend_options` and `run_options` under the `options` block. Following example specifies `backend_options`.
```json
{
  "pubs": [
    [
      "OPENQASM 3.0; ...",
      [
        {
          "IIIIIIIIIIIIIZIIIIIIIIIIIII": 1.0
        },

      ]
    ]
  ],
  "version": 2,
  "support_qiskit": false,
  "options": {
    "default_shots": 5000,
    "backend_options": {
      "max_parallel_threads": 5,
      "max_parallel_experiments": 1,
      "max_parallel_shots": 1,
    }
  }
}
```

## Specify those options in configuration file

Those specifications will be applied to all jobs run on this API simulator and can be overridden by the values specified in Job parameter JSON at runtime as described in above.
```yaml
aer_options:
  # AerSimulator backend_options for EstimatorV2.
  estimatorV2:
    backend_options:
      max_parallel_threads: 5
      max_parallel_experiments: 1
      max_parallel_shots: 1

  # AerSimulator backend_options for SamplerV2.
  samplerV2:
    backend_options:
      method: "matrix_product_state"

```
