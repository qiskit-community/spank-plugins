# FAQ

## How can I check the version of a deployed `spank_qrmi.so`?

Every `spank_qrmi.so` build embeds version information directly into the binary, so you can check it on a running Slurm node without executing the plugin or restarting `slurmd`.

### Using `strings`

```shell-session
$ strings /path/to/spank_qrmi.so | grep QRMI
SPANK_QRMI_VERSION=0.11.0;QRMI_CRATE_VERSION=0.24.0;QRMI_GIT_HASH=66b69cec87de
```

### Using `readelf`

```shell-session
$ readelf -p .version_info /path/to/spank_qrmi.so

String dump of section '.version_info':
  [     0]  SPANK_QRMI_VERSION=0.11.0;QRMI_CRATE_VERSION=0.24.0;QRMI_GIT_HASH=66b69cec87de
```

### What each field means

| Field | Meaning |
|---|---|
| `SPANK_QRMI_VERSION` | Version of this plugin (`spank_qrmi`), taken from [`VERSION.txt`](../plugins/spank_qrmi/VERSION.txt) at build time. |
| `QRMI_CRATE_VERSION` | Version of the QRMI Rust crate that this build was linked against (from QRMI's own `Cargo.toml`). |
| `QRMI_GIT_HASH` | The exact git commit of QRMI that was checked out and built, useful when `QRMI_GIT_TAG` points at a moving branch such as `main`. |

This is especially useful when diagnosing issues caused by a version mismatch between the deployed `spank_qrmi.so` and the QRMI Python package used by client workloads — you can confirm exactly what's on disk without needing to rebuild or trace through job logs.

### Notes

- If `QRMI_GIT_HASH` shows `unknown`, the build most likely used `-DQRMI_ROOT=<path>` to point at a local QRMI checkout that isn't a git repository (e.g. an extracted tarball).
- If neither `strings` nor `readelf` show a `.version_info` section, the binary may have been built before this feature was introduced, or the section may have been removed by a full `strip -s` pass in the deployment pipeline. Re-run `strip --strip-debug` instead, or add `--keep-section=.version_info` to the `strip`/`objcopy` invocation, to preserve it.
