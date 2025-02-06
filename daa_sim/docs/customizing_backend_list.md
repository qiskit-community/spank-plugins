# Customizing backend list

You can override the default backend list by specifying the `$.backends` property as follows: `module` property is a python module containing the backend class specified by the `clazz` property.

```yaml
backends:
  - module: qiskit_ibm_runtime.fake_provider.backends
    clazz: FakeTorino
  - module: qiskit_ibm_runtime.fake_provider.backends
    clazz: FakeKawasaki
  - module: qiskit_ibm_runtime.fake_provider.backends
    clazz: FakeAlgiers
```

By specifying above, you will find `fake_torino`, `fake_kawasaki` and `fake_algiers` in the `GET /v1/backends` API response.
