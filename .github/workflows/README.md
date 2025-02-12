# Workflows for CI/CD

[![SPANK Plugins](https://img.shields.io/badge/SPANK-Plugins-blue)](https://slurm.schedmd.com/spank.html) [![HPC Slurm](https://img.shields.io/badge/HPC-Slurm-green)](https://slurm.schedmd.com/) [![made-with-C](https://img.shields.io/badge/Made%20with-C-1f425f.svg)](https://en.wikipedia.org/wiki/C_(programming_language))

# 🛠 CI/CD Workflows

This repository includes **GitHub Actions workflows** to ensure code quality, enforce linting, and run tests. These
workflows are located in the `.github/workflows/` directory and are automatically triggered when a **pull request (PR)
is created** or **code is pushed to the main branch**. Additionally, some workflows allow manual execution.

## 📌 Available Workflows

### **1️⃣ Linting Workflow**

- **File:** `.github/workflows/linting.yml`
- **Purpose:** Runs linters and static analysis tools to enforce coding standards for **C, Rust, Python, and Shell
  scripts**.
- **Triggers:**
    - On every **pull request** to `main`.
- **Main Steps:**
    - Install linting tools
    - Run `clang-tidy` for C code
    - Run `clippy` for Rust code
    - Run `ruff` and `mypy` for Python code
    - Run `shellcheck` and `shfmt` for shell scripts

### **2️⃣ Build & Test Workflow**

- **File:** `.github/workflows/build-test.yml`
- **Purpose:** Builds the project and runs tests using **MySQL and SLURM**.
- **Triggers:**
    - On every **pull request** to `main`
    - On **push** to `main`
    - Can also be triggered **manually** via the GitHub UI (`workflow_dispatch`).
- **Main Steps:**
    - Set up a test SLURM cluster
    - Run automated tests (Work In Progress)

## 🚀 How CI/CD Works

1. **Pull Request Created** → Triggers both `Linting` and `Build & Test` workflows.
2. **Push to `main` Branch** → Triggers the `Build & Test` workflow.
3. **Manual Trigger (`workflow_dispatch`)** → Allows manually starting the `Build & Test` workflow.

## 🛠 Detailed Workflow Steps

### 🔹 **Linting Workflow Steps**

| Step Name             | Description                                                            |
|-----------------------|------------------------------------------------------------------------|
| Checkout repository   | Fetches the latest code                                                |
| Install linting tools | Installs `clang-tidy`, `clippy`, `ruff`, `mypy`, `shellcheck`, `shfmt` |
| Set up C environment  | Prepares the `build` directory for linting                             |
| Run `clang-tidy`      | Lints C code                                                           |
| Run Rust `clippy`     | Lints Rust code                                                        |
| Run `ruff`            | Lints Python code                                                      |
| Run `mypy`            | Checks Python type annotations                                         |
| Report linting status | Displays final linting results                                         |

### 🔹 **Build & Test Workflow Steps**

| Step Name                | Description                                   |
|--------------------------|-----------------------------------------------|
| Checkout code            | Fetches the latest code                       |
| Setup MySQL service      | Starts MySQL container for testing            |
| Setup Test SLURM Cluster | Configures SLURM cluster for scheduling tasks |
| Run tests                | Executes the test suite                       |

---

By using these workflows, we **enforce code quality, catch errors early, and ensure that all commits follow best
practices** before merging into `main`. 🚀
