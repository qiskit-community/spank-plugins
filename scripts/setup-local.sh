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
    brew list composer &> /dev/null || brew install composer
}

print_version() {
    local cmd=$1
    if command -v "$cmd" &> /dev/null; then
        echo "$cmd version: $($cmd --version | head -n 1)"
    else
        echo "$cmd is not installed."
    fi
}

install_docker_compose_fallback() {
    echo "Installing Docker Compose via the official Docker repository..."

    # Download the Docker Compose binary
    sudo mkdir -p /usr/local/lib/docker/cli-plugins
    sudo curl -SL "https://github.com/docker/compose/releases/download/v2.17.3/docker-compose-linux-x86_64" -o /usr/local/lib/docker/cli-plugins/docker-compose

    # Set the correct permissions
    sudo chmod +x /usr/local/lib/docker/cli-plugins/docker-compose

    echo "Docker Compose v2 installed successfully from the official Docker repository."
    docker compose version
}

install_linux_dependencies() {
    if command -v apt-get &> /dev/null; then
        echo "Detected Debian/Ubuntu-based system."
        sudo apt-get update -y || {
            echo "Failed to update package lists. Please check your network or apt-get configuration."
            exit 1
        }

        # Installing each package separately to detect specific issues
        for package in gcc clang curl docker.io; do
            echo "Installing $package..."
            sudo apt-get install -y "$package" || {
                echo "Failed to install $package. Please check your package manager or manually install it."
                exit 1
            }
        done

        # Try to install docker-compose-plugin first; if it fails, use fallback
        echo "Installing docker-compose-plugin..."
        if ! sudo apt-get install -y docker-compose-plugin; then
            echo "Failed to locate docker-compose-plugin. Falling back to manual installation."
            install_docker_compose_fallback
        fi

        # Install Composer
        echo "Installing Composer..."
        sudo apt-get install -y composer || {
            echo "Failed to install Composer. Please install it manually."
            exit 1
        }

    elif command -v yum &> /dev/null; then
        echo "Detected RedHat/CentOS/Fedora-based system."
        sudo yum update -y &> /dev/null
        sudo yum install -y gcc clang curl docker docker-compose composer &> /dev/null || {
            echo "Failed to install dependencies with yum. Please install manually."
            exit 1
        }

    elif command -v pacman &> /dev/null; then
        echo "Detected Arch-based system."
        sudo pacman -Syu --noconfirm &> /dev/null
        sudo pacman -S --noconfirm gcc clang curl docker docker-compose composer &> /dev/null || {
            echo "Failed to install dependencies with pacman. Please install manually."
            exit 1
        }

    else
        echo "Unsupported Linux distribution. Please install dependencies manually."
        exit 1
    fi

    echo "Basic dependencies installed successfully."
    print_version gcc
    print_version clang
    print_version curl
    print_version docker
    print_version docker-compose
    print_version composer
}

install_kubectl_linux() {
    if ! command -v kubectl &> /dev/null; then
        echo "Installing kubectl using package manager or fallback to direct download..."

        if command -v apt-get &> /dev/null; then
            sudo apt-get install -y kubectl &> /dev/null && {
                echo "kubectl installed successfully using apt-get."
                return 0
            }
        elif command -v yum &> /dev/null; then
            sudo yum install -y kubectl &> /dev/null && {
                echo "kubectl installed successfully using yum."
                return 0
            }
        fi

        # Fallback to direct download if package manager fails
        echo "Package manager installation failed. Falling back to direct download."
        curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl" || {
            echo "Failed to download kubectl. Please install it manually."
            exit 1
        }

        sudo install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl || {
            echo "Failed to install kubectl. Please install it manually."
            exit 1
        }

        echo "kubectl installed successfully via direct download."
    else
        echo "kubectl is already installed."
        kubectl version --client --output=yaml
    fi
}

install_docker_and_kubectl_linux() {
    install_linux_dependencies
    install_kubectl_linux
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

    install_docker_and_kubectl_mac
}

install_linux() {
    sudo apt-get update -y &> /dev/null || sudo yum update -y &> /dev/null
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
