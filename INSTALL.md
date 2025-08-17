# Installation

This document describes how to setup QRMI and its Spank plugin on your Slurm Cluster


## Pre-requisites

- [Spank plugin](./plugins/spank_qrmi#prerequisites)
- [QRMI](https://github.com/qiskit-community/qrmi/blob/main/INSTALL.md#prerequisites)


## Steps

### 1. Build and install QRMI

Refer [How to build & install QRMI Python package](https://github.com/qiskit-community/qrmi/blob/main/INSTALL.md#how-to-build--install-qrmi-python-package), and create a wheel file for distribution.

### 2. Build and install SPANK Plugin

Refer [README](https://github.com/ohtanim/spank-plugins/tree/set_qrmi_loglevel/plugins/spank_qrmi) and install `plugstack.conf`, `qrmi_config.json` and `spank_qrmi.so`.

### 3. Running examples of primitive job in Slurm Cluster

Sample jobs are available in [this directory](./demo/qrmi/jobs). Make sure to update the paths for the Python virtual environment and the sample script to match your local setup.

## END OF DOCUMENT
