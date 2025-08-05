# Skeleton SPANK Plugin in Rust

> [!CAUTION]
> This module is marked deprecated in a future update, and then removed. Use [skeleton](../../skeleton).

## Prerequisites

* Rust 1.85.1 or above
* Slurm header & library
  * slurm/slurm.h must be available under /usr/include
  * libslurm.so must be available under /usr/lib64 or /usr/lib/x86_64-linux-gnu

## How to build

```shell-session
. ~/.cargo/env
cargo clean
cargo build --release
```

## License

[GPL-3.0](https://github.com/qiskit-community/spank-plugins/blob/main/LICENSE)
