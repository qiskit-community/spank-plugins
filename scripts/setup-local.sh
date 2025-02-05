#!/bin/bash

set -e

echo "Detecting OS..."

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

echo "Detected OS: $OS"
echo "----------------------------------------------"

check_and_install() {
    TOOL=$1
    INSTALL_CMD=$2
    VERSION_CMD=$3

    if command -v $TOOL &> /dev/null; then
        echo "$TOOL is already installed. Version: $($VERSION_CMD)"
    else
        echo "Installing $TOOL..."
        eval "$INSTALL_CMD"
    fi
    echo "----------------------------------------------"
}

install_docker_compose() {
    echo "Checking and installing Docker Compose..."
    echo "----------------------------------------------"
    if command -v docker-compose &> /dev/null; then
        echo "Docker Compose is already installed. Version: $(docker-compose --version)"
    else
        echo "Installing Docker Compose..."
        if [[ "$OS" == "linux" ]]; then
            sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
            sudo chmod +x /usr/local/bin/docker-compose
        elif [[ "$OS" == "mac" ]]; then
            brew install docker-compose
        elif [[ "$OS" == "windows" ]]; then
            pacman -S --noconfirm docker-compose
        fi
        echo "Docker Compose installed. Version: $(docker-compose --version)"
    fi
    echo "----------------------------------------------"
}

install_dependencies() {
    echo "Installing core dependencies..."
    echo "----------------------------------------------"

    # Core tools
    check_and_install gcc "$1" "gcc --version"
    check_and_install clang "$2" "clang --version"
    check_and_install docker "$3" "docker --version"
    install_docker_compose
    echo "----------------------------------------------"
}

install_rancher_and_codelite() {
    echo "Installing Rancher Desktop and CodeLite (with error handling)..."
    echo "----------------------------------------------"

    # Rancher Desktop installation
    echo "Installing Rancher Desktop..."
    if ! eval "$1"; then
        echo "Warning: Failed to install Rancher Desktop. Skipping..."
    else
        echo "Rancher Desktop installed successfully."
    fi
    echo "----------------------------------------------"

    # CodeLite installation
    echo "Installing CodeLite..."
    if ! eval "$2"; then
        echo "Warning: Failed to install CodeLite. Skipping..."
    else
        echo "CodeLite installed successfully."
    fi
    echo "----------------------------------------------"
}

install_mac() {
    # Install Homebrew if not installed
    if ! command -v brew &> /dev/null; then
        echo "Homebrew not found. Installing Homebrew..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    fi

    install_dependencies "brew install gcc" "brew install clang" "brew install docker-cli"

    install_rancher_and_codelite \
        "brew install --cask rancher" \
        "brew install codelite"
}

install_linux() {
    # Update package lists
    sudo apt-get update -y || sudo yum update -y

    install_dependencies "sudo apt-get install -y gcc || sudo yum install -y gcc" \
                         "sudo apt-get install -y clang || sudo yum install -y clang" \
                         "sudo apt-get install -y docker.io || sudo yum install -y docker"

    install_rancher_and_codelite \
        "curl -s https://download.opensuse.org/repositories/isv:/Rancher:/stable/debian/Rancher.key | sudo apt-key add - && \
        sudo bash -c 'echo \"deb https://download.opensuse.org/repositories/isv:/Rancher:/stable/debian/ /\" > /etc/apt/sources.list.d/rancher.list' && \
        sudo apt-get update -y && sudo apt-get install -y rancher-desktop" \
        "sudo apt-get install -y codelite || sudo yum install -y codelite"
}

install_windows() {
    # MSYS2 installation for Linux-like environment
    if ! command -v pacman &> /dev/null; then
        echo "Downloading and installing MSYS2..."
        curl -LO https://repo.msys2.org/distrib/x86_64/msys2-x86_64-latest.exe
        ./msys2-x86_64-latest.exe
    fi

    install_dependencies "pacman -S --noconfirm mingw-w64-x86_64-gcc" \
                         "pacman -S --noconfirm mingw-w64-x86_64-clang" \
                         "pacman -S --noconfirm mingw-w64-x86_64-docker"

    install_rancher_and_codelite \
        "curl -LO 'https://github.com/rancher-sandbox/rancher-desktop/releases/latest/download/Rancher.Desktop.Setup.exe' && ./Rancher.Desktop.Setup.exe /silent" \
        "pacman -S --noconfirm mingw-w64-x86_64-codelite"
}

setup_codelite() {
    echo "Setting up CodeLite with the installed C compiler..."
    echo "----------------------------------------------"

    if [[ "$OS" == "mac" || "$OS" == "linux" ]]; then
        CODELITE_CONFIG_FILE="$HOME/.config/codelite/codelite.conf"
    elif [[ "$OS" == "windows" ]]; then
        CODELITE_CONFIG_FILE="/c/Users/$USER/AppData/Roaming/codelite/codelite.conf"
    fi

    if [[ -f "$CODELITE_CONFIG_FILE" ]]; then
        echo "Found CodeLite config at $CODELITE_CONFIG_FILE. Adjusting settings..."
        # Add custom configuration setup here if needed
    fi
    echo "----------------------------------------------"
}

install_cli_tools() {
    echo "Installing optional CLI tools for testing C programs..."
    echo "----------------------------------------------"

    check_and_install cmake "brew install cmake || sudo apt-get install -y cmake || sudo yum install -y cmake || pacman -S --noconfirm mingw-w64-x86_64-cmake" "cmake --version"
}

# Execute platform-specific installation steps
case $OS in
    mac) install_mac ;;
    linux) install_linux ;;
    windows) install_windows ;;
    *) echo "Unsupported OS"; exit 1 ;;
esac

setup_codelite
install_cli_tools

echo "Installation and setup complete!"
echo "----------------------------------------------"
echo "To test C programs from the CLI, use gcc/clang as follows:"
echo "gcc -o program program.c && ./program"
echo "You can run Docker commands using 'docker', 'docker-compose', and launch Rancher Desktop as needed."
