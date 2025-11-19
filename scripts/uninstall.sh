#!/bin/bash

# Android Proxy Setter Uninstallation Script
# This script removes the tool from user's PATH and cleans up aliases

set -euo pipefail

# Color codes for output
RED="\033[0;31m"
GREEN="\033[0;32m"
YELLOW="\033[1;33m"
BLUE="\033[0;34m"
NC="\033[0m" # No Color

# Function to print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Get the script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_NAME="android-proxy-setter"
BINARY_NAME="android_proxy_setter"

# Detect user's shell
detect_shell() {
    local shell_name="${SHELL##*/}"
    case "$shell_name" in
        bash)
            echo "bash"
            ;;
        zsh)
            echo "zsh"
            ;;
        fish)
            echo "fish"
            ;;
        *)
            echo "bash"  # Default to bash
            ;;
    esac
}

# Get shell config file path
get_shell_config() {
    local shell_type="$1"
    case "$shell_type" in
        bash)
            echo "$HOME/.bashrc"
            ;;
        zsh)
            echo "$HOME/.zshrc"
            ;;
        fish)
            echo "$HOME/.config/fish/config.fish"
            ;;
    esac
}

# Get installation directory
get_install_dir() {
    echo "$HOME/.local/bin"
}

# Remove binary
remove_binary() {
    local install_dir="$1"
    local binary_path="$install_dir/$BINARY_NAME"

    if [[ -f "$binary_path" ]]; then
        rm "$binary_path"
        print_success "Removed binary: $binary_path"
    else
        print_info "Binary not found: $binary_path"
    fi
}

# Remove from PATH and aliases
remove_from_config() {
    local shell_type="$1"
    local config_file="$2"
    local install_dir="$3"

    if [[ ! -f "$config_file" ]]; then
        print_info "Config file not found: $config_file"
        return 0
    fi

    print_info "Cleaning up $config_file..."

    # Create a temporary file
    local temp_file
    temp_file=$(mktemp)

    case "$shell_type" in
        bash|zsh)
            # Remove PATH entry and aliases
            grep -v "export PATH=\"$install_dir:" "$config_file" | \
            grep -v "# Android Proxy Setter aliases" | \
            grep -v "alias aps=" > "$temp_file"
            ;;
        fish)
            # Remove PATH entry and aliases
            grep -v "set -gx PATH \"$install_dir\"" "$config_file" | \
            grep -v "# Android Proxy Setter aliases" | \
            grep -v "alias aps=" > "$temp_file"
            ;;
    esac

    # Replace the original file
    mv "$temp_file" "$config_file"

    print_success "Removed configuration from $config_file"
}

# Check if installation directory is empty and remove if so
cleanup_install_dir() {
    local install_dir="$1"

    if [[ -d "$install_dir" ]]; then
        if [[ -z "$(ls -A "$install_dir")" ]]; then
            rmdir "$install_dir"
            print_success "Removed empty installation directory: $install_dir"
        else
            print_info "Installation directory not empty, keeping: $install_dir"
        fi
    fi
}

# Main uninstallation function
main() {
    print_info "Uninstalling Android Proxy Setter..."

    # Detect shell
    local shell_type
    shell_type=$(detect_shell)
    print_info "Detected shell: $shell_type"

    # Get shell config file
    local config_file
    config_file=$(get_shell_config "$shell_type")
    print_info "Using config file: $config_file"

    # Get installation directory
    local install_dir
    install_dir=$(get_install_dir)

    # Remove binary
    remove_binary "$install_dir"

    # Remove from config
    remove_from_config "$shell_type" "$config_file" "$install_dir"

    # Cleanup installation directory
    cleanup_install_dir "$install_dir"

    print_success ""
    print_success "🗑️  Uninstallation completed successfully!"
    print_success ""
    print_success "The following have been removed:"
    print_success "  - Binary: $install_dir/$BINARY_NAME"
    print_success "  - PATH configuration"
    print_success "  - Shell aliases (aps, aps-set, etc.)"
    print_success ""
    print_success "To complete the cleanup, please restart your terminal"
    print_success "or run: source $config_file"
}

# Run main function
main "$@"