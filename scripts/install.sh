#!/bin/bash
set -e

# Installation script for local-search CLI tool

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
INSTALL_DIR="/usr/local/bin"
GITHUB_REPO="nnanto/local-search"

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to detect architecture
get_architecture() {
    local arch=$(uname -m)
    local os=$(uname -s)
    
    case $os in
        Linux*)
            case $arch in
                x86_64) echo "linux-x86_64" ;;
                *) print_error "Unsupported architecture: $arch on $os"; exit 1 ;;
            esac
            ;;
        Darwin*)
            case $arch in
                x86_64) echo "macos-x86_64" ;;
                arm64) echo "macos-aarch64" ;;
                *) print_error "Unsupported architecture: $arch on $os"; exit 1 ;;
            esac
            ;;
        *)
            print_error "Unsupported operating system: $os"
            exit 1
            ;;
    esac
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Main installation function
install_local-search() {
    local arch=$(get_architecture)
    local archive_name="local-search-${arch}.tar.gz"
    local download_url="https://github.com/${GITHUB_REPO}/releases/latest/download/${archive_name}"
    
    print_status "Detected architecture: $arch"
    print_status "Download URL: $download_url"
    
    # Check for required tools
    if ! command_exists curl; then
        print_error "curl is required but not installed."
        exit 1
    fi
    
    if ! command_exists tar; then
        print_error "tar is required but not installed."
        exit 1
    fi
    
    # Create temporary directory
    local tmp_dir=$(mktemp -d)
    cd "$tmp_dir"
    
    # Download and extract
    print_status "Downloading local-search..."
    if ! curl -sL "$download_url" | tar xz; then
        print_error "Failed to download or extract local-search"
        exit 1
    fi
    
    # Install binary
    print_status "Installing to $INSTALL_DIR..."
    
    # Check if we can write to install directory
    if [[ ! -w "$INSTALL_DIR" ]]; then
        print_warning "No write permission to $INSTALL_DIR, trying with sudo..."
        if ! sudo mv local-search "$INSTALL_DIR/local-search"; then
            print_error "Failed to install local-search to $INSTALL_DIR"
            exit 1
        fi
    else
        if ! mv local-search "$INSTALL_DIR/local-search"; then
            print_error "Failed to install local-search to $INSTALL_DIR"
            exit 1
        fi
    fi
    
    # Make executable
    if [[ ! -w "$INSTALL_DIR/local-search" ]]; then
        sudo chmod +x "$INSTALL_DIR/local-search"
    else
        chmod +x "$INSTALL_DIR/local-search"
    fi
    
    # Cleanup
    cd /
    rm -rf "$tmp_dir"
    
    print_status "local-search installed successfully!"
    print_status "Try running: local-search --help"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --install-dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        --repo)
            GITHUB_REPO="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --install-dir DIR    Installation directory (default: /usr/local/bin)"
            echo "  --repo REPO          GitHub repository (default: nnanto/local-search)"
            echo "  -h, --help           Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Run installation
install_local-search