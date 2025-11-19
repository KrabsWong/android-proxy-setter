#!/bin/bash

# Android Proxy Setter Installation Script
# This script installs the tool to user's PATH and creates shell aliases

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

# Get the script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_NAME="android-proxy-setter"
BINARY_NAME="android_proxy_setter"

# Check if the project is built
check_build() {
    if [[ ! -f "$PROJECT_ROOT/target/release/$BINARY_NAME" ]]; then
        print_error "Release binary not found. Please build the project first:"
        echo "  cargo build --release"
        exit 1
    fi
    print_success "Found release binary: $PROJECT_ROOT/target/release/$BINARY_NAME"
}

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

# Create installation directory
create_install_dir() {
    local install_dir="$HOME/.local/bin"
    if [[ ! -d "$install_dir" ]]; then
        mkdir -p "$install_dir"
        print_success "Created installation directory: $install_dir"
    fi
    echo "$install_dir"
}

# Install the binary
install_binary() {
    local install_dir="$1"
    local binary_path="$install_dir/$BINARY_NAME"

    # Copy the binary
    cp "$PROJECT_ROOT/target/release/$BINARY_NAME" "$binary_path"
    chmod +x "$binary_path"

    print_success "Installed binary to: $binary_path"
}

# Add to PATH if not already present
add_to_path() {
    local install_dir="$1"
    local shell_type="$2"
    local config_file="$3"

    # Check if PATH already contains the install directory
    if echo "$PATH" | grep -q "$install_dir"; then
        print_info "PATH already contains installation directory"
        return 0
    fi

    print_info "Adding $install_dir to PATH in $config_file"

    case "$shell_type" in
        bash|zsh)
            echo "export PATH=\"$install_dir:\$PATH\"" >> "$config_file"
            ;;
        fish)
            echo "set -gx PATH \"$install_dir\" \$PATH" >> "$config_file"
            ;;
    esac

    print_success "Added to PATH. Please restart your shell or run: source $config_file"
}

# Create shell aliases
create_aliases() {
    local shell_type="$1"
    local config_file="$2"

    print_info "Creating shell aliases..."

    case "$shell_type" in
        bash|zsh)
            cat >> "$config_file" << 'EOF'

# Android Proxy Setter aliases
alias aps='android_proxy_setter'
alias aps-set='android_proxy_setter --set'
alias aps-clear='android_proxy_setter --clear'
alias aps-view='android_proxy_setter --view'
alias aps-restart='android_proxy_setter --restart-adb'
EOF
            ;;
        fish)
            cat >> "$config_file" << 'EOF'

# Android Proxy Setter aliases
alias aps='android_proxy_setter'
alias aps-set='android_proxy_setter --set'
alias aps-clear='android_proxy_setter --clear'
alias aps-view='android_proxy_setter --view'
alias aps-restart='android_proxy_setter --restart-adb'
EOF
            ;;
    esac

    print_success "Created aliases: aps, aps-set, aps-clear, aps-view, aps-restart"
}

# Main installation function
main() {
    print_info "Installing Android Proxy Setter..."

    # Check if project is built
    check_build

    # Detect shell
    local shell_type
    shell_type=$(detect_shell)
    print_info "Detected shell: $shell_type"

    # Get shell config file
    local config_file
    config_file=$(get_shell_config "$shell_type")
    print_info "Using config file: $config_file"

    # Create installation directory
    local install_dir
    install_dir=$(create_install_dir)

    # Install binary
    install_binary "$install_dir"

    # Add to PATH
    add_to_path "$install_dir" "$shell_type" "$config_file"

    # Create aliases
    create_aliases "$shell_type" "$config_file"

    print_success ""
    print_success "🎉 Installation completed successfully!"
    print_success ""
    print_success "Available commands:"
    print_success "  aps           - Interactive mode (same as android_proxy_setter)"
    print_success "  aps-set       - Set proxy directly"
    print_success "  aps-clear     - Clear proxy directly"
    print_success "  aps-view      - View current proxy settings"
    print_success "  aps-restart   - Restart ADB server"
    print_success ""
    print_success "To start using immediately, run:"
    print_success "  source $config_file"
    print_success ""
    print_success "Or simply restart your terminal."
}

# Run main function
main "$@"