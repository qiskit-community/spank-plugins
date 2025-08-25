# A tool to validate Slurm plugin symbols

This is a verification tool that checks whether a given Slurm plugin library file contains the required metadata symbols.

## Build

```bash
mkdir build
cd build
cmake ..
make
```

## Run

Assuming `libslurmfull.so` is available under `/lib64/slurm` directory,

```bash
LD_LIBRARY_PATH=/lib64/slurm:$LD_LIBRARY_PATH ./test <plugin library>
```

Example:

If SPANK plugin is specified to validate, `type` should be `spank`.

```bash
$ LD_LIBRARY_PATH=/lib64/slurm:$LD_LIBRARY_PATH build/test ../../spank_qrmi/build/spank_qrmi.so
Valid Slurm plugin library. name=spank_qrmi, type=spank, version=25.5.2
```
