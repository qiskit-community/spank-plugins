# FAQ

## How can I check the version of a deployed `spank_qrmi.so`?

Every `spank_qrmi.so` build embeds version information directly into the binary, so you can check it on a running Slurm node without executing the plugin or restarting `slurmd`.

### Using `strings`

```shell-session
$ strings /path/to/spank_qrmi.so | grep SPANK_QRMI
SPANK_QRMI_BUILD_VERSION=0.11.0;SPANK_QRMI_GIT_HASH=0dac1793b013
```

### Using `readelf`

```shell-session
$ readelf -p .version_info /path/to/spank_qrmi.so

String dump of section '.version_info':
  [     0]  SPANK_QRMI_BUILD_VERSION=0.11.0;SPANK_QRMI_GIT_HASH=0dac1793b013
```

### What each field means

| Field | Meaning |
|---|---|
| `SPANK_QRMI_BUILD_VERSION` | Version of this plugin (`spank_qrmi`), taken from the `project(spank_qrmi VERSION x.y.z ...)` declaration in [`CMakeLists.txt`](../plugins/spank_qrmi/CMakeLists.txt) at build time. |
| `SPANK_QRMI_GIT_HASH` | The exact git commit of the `spank-plugins` repo this build came from, resolved via `git rev-parse --short=12 HEAD` at build time. |

This is especially useful when diagnosing issues caused by a version mismatch between the deployed `spank_qrmi.so` and the QRMI Python package used by client workloads — you can confirm exactly what's on disk without needing to rebuild or trace through job logs.

### What about the QRMI crate version?

QRMI embeds its own version/git-hash marker into the same `.version_info` section via its own crate, so it shows up alongside `spank_qrmi`'s marker in the same linked `.so`:

```shell-session
$ strings spank_qrmi.so | grep -E "SPANK_QRMI|QRMI_BUILD"
SPANK_QRMI_BUILD_VERSION=0.11.0;SPANK_QRMI_GIT_HASH=0dac1793b013
QRMI_BUILD_VERSION:0.24.0;QRMI_GIT_HASH:0dac1793b013
```

The two are independent: `SPANK_QRMI_*` describes this plugin binary itself, `QRMI_BUILD_VERSION`/`GIT_HASH` describes the QRMI crate it was linked against. See QRMI's own FAQ for details on the latter.

### Notes

- If `SPANK_QRMI_GIT_HASH` shows `unknown`, the build most likely happened outside a git checkout (e.g. a tarball/zip export of the repo with no `.git` directory).
- If neither `strings` nor `readelf` show a `.version_info` section, the binary may have been built before this feature was introduced, or the section may have been removed by a full `strip -s` pass in the deployment pipeline. Re-run `strip --strip-debug` instead, or add `--keep-section=.version_info` to the `strip`/`objcopy` invocation, to preserve it.
- `SPANK_QRMI_BUILD_VERSION` needs to be bumped by hand in the `project(...  VERSION x.y.z)` line in `CMakeLists.txt` as part of a release PR; it isn't derived from git tags.
