# Skeleton SPANK Plugin in Rust

## Prerequisites

* Rust 1.85.1 or above
* Slurm header & library
  * slurm/slurm.h must be under /usr/include

## How to build

```shell-session
. ~/.cargo/env
cargo clean
cargo build --release
```

## License

[GPL-3.0](https://github.com/qiskit-community/spank-plugins/blob/main/LICENSE)
