#!/usr/bin/env bash

set -e

# Detect OS
OS=""
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="linux"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    OS="mac"
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
    OS="windows"
else
    echo "Unsupported OS: $OSTYPE"
    exit 1
fi

# Backup current PATH and bash environment
ORIGINAL_PATH=$PATH
ORIGINAL_BASH=$(command -v bash)

backup_bash_environment() {
    ORIGINAL_PATH=$PATH
    ORIGINAL_BASH=$(command -v bash)
}

restore_bash_environment() {
    export PATH="$ORIGINAL_PATH"
    if ! command -v bash &> /dev/null; then
        if [[ "$OS" == "windows" ]]; then
            if [ -f "/c/Program Files/Git/bin/bash.exe" ]; then
                export PATH="/c/Program Files/Git/bin:$PATH"
            elif [ -f "/c/Windows/System32/bash.exe" ]; then
                export PATH="/c/Windows/System32:$PATH"
            else
                echo "Error: Unable to restore bash. Please restart or fix manually."
                exit 1
            fi
        else
            echo "Error: Unable to restore bash. Please check the installation."
            exit 1
        fi
    fi
}

check_winget_installed() {
    powershell.exe -Command "Get-Command winget" &> /dev/null
}

winget_is_installed() {
    local app_name=$1
    powershell.exe -Command "
        \$wingetList = winget list --source winget | Select-String -Pattern '$app_name'
        if (\$wingetList) {
            exit 0
        } else {
            exit 1
        }
    " &> /dev/null
}

winget_install_if_not_present() {
    local app_name=$1
    local winget_id=$2

    if winget_is_installed "$app_name"; then
        echo "$app_name is already installed."
    else
        echo "$app_name is not installed. Installing..."
        powershell.exe -Command "
            Start-Process winget -ArgumentList 'install --id $winget_id --silent --accept-package-agreements --accept-source-agreements --force' -Verb RunAs -Wait
        " || {
            echo "Warning: Failed to install $app_name using winget."
            exit 1
        }
        echo "$app_name installation completed."
    fi
}

install_docker_and_kubectl_windows() {
    if ! check_winget_installed; then
        echo "winget is not available. Please update Windows to a version with winget support."
        exit 1
    fi

    winget_install_if_not_present "Docker CLI" "Docker.DockerDesktop"
    winget_install_if_not_present "Kubernetes CLI (kubectl)" "Kubernetes.kubectl"
}

install_docker_and_kubectl_mac() {
    brew list docker &> /dev/null || brew install docker docker-compose
    brew list kubectl &> /dev/null || brew install kubectl
}

install_docker_and_kubectl_linux() {
    if ! command -v docker &> /dev/null; then
        sudo apt-get install -y docker.io docker-compose &> /dev/null || sudo yum install -y docker docker-compose &> /dev/null || {
            echo "Failed to install Docker CLI. Please install manually."
            exit 1
        }
    fi

    if ! command -v kubectl &> /dev/null; then
        sudo apt-get install -y kubectl &> /dev/null || sudo yum install -y kubectl &> /dev/null || {
            echo "Failed to install Kubernetes CLI. Please install manually."
            exit 1
        }
    fi
}

install_dependencies() {
    local gcc_install=$1
    local clang_install=$2

    check_and_install gcc "$gcc_install" "gcc --version"
    check_and_install clang "$clang_install" "clang --version"
}

install_windows() {
    if [ ! -d "C:\msys64" ]; then
        echo "Installing MSYS2 using winget..."
        winget_install_if_not_present "MSYS2" "MSYS2.MSYS2"
    fi

    winget_install_if_not_present "GCC" "GNU.GCC"
    winget_install_if_not_present "LLVM" "LLVM.LLVM"

    install_docker_and_kubectl_windows
    restore_bash_environment
}

install_mac() {
    if ! command -v brew &> /dev/null; then
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    fi

    install_dependencies "brew install gcc" "brew install clang"
    install_docker_and_kubectl_mac
}

install_linux() {
    sudo apt-get update -y &> /dev/null || sudo yum update -y &> /dev/null
    install_dependencies "sudo apt-get install -y gcc" "sudo apt-get install -y clang"
    install_docker_and_kubectl_linux
}

# Backup environment before installations
backup_bash_environment

# Execute platform-specific installation steps
case $OS in
    mac) install_mac ;;
    linux) install_linux ;;
    windows) install_windows ;;
    *) echo "Unsupported OS"; exit 1 ;;
esac

# Final restoration of bash after all installations
restore_bash_environment

echo "Installation and setup complete!"

if [[ "$OS" == "windows" ]]; then
    echo "RECOMMENDATION: Please restart your computer to ensure all changes take effect."
fi
