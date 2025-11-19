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

    # Check if config file already has PATH entry for this directory
    case "$shell_type" in
        bash|zsh)
            if grep -q "export PATH=\"$install_dir:" "$config_file" 2>/dev/null; then
                print_info "PATH entry already exists in $config_file"
                return 0
            fi
            ;;
        fish)
            if grep -q "set -gx PATH \"$install_dir\"" "$config_file" 2>/dev/null; then
                print_info "PATH entry already exists in $config_file"
                return 0
            fi
            ;;
    esac

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

# Check for command conflicts and identify existing tool commands
check_command_conflicts() {
    local prefix="$1"
    local commands=("$prefix" "$prefix-set" "$prefix-clear" "$prefix-view" "$prefix-restart")
    local conflicts=()
    local existing_tool_commands=()
    local current_tool_path="$HOME/.local/bin/android_proxy_setter"

    for cmd in "${commands[@]}"; do
        if type "$cmd" >/dev/null 2>&1; then
            # Check if this command is already our tool
            local is_our_tool=false

            # Check if it's an alias pointing to our tool
            local alias_def
            alias_def=$(alias "$cmd" 2>/dev/null || echo "")
            if [[ "$alias_def" == *"android_proxy_setter"* ]]; then
                is_our_tool=true
            fi

            # Check if it's a function that calls our tool
            local func_def
            func_def=$(type "$cmd" 2>/dev/null | grep -q "android_proxy_setter" && echo "found" || echo "")
            if [[ -n "$func_def" ]]; then
                is_our_tool=true
            fi

            # Check if it's the actual binary path
            local cmd_path
            cmd_path=$(command -v "$cmd" 2>/dev/null || echo "")
            if [[ "$cmd_path" == "$current_tool_path" ]]; then
                is_our_tool=true
            fi

            if [[ "$is_our_tool" == "true" ]]; then
                existing_tool_commands+=("$cmd")
            else
                conflicts+=("$cmd")
            fi
        fi
    done

    # If all commands are already our tool, return special code
    if [[ ${#existing_tool_commands[@]} -eq ${#commands[@]} ]]; then
        return 2  # All commands already exist and are our tool
    fi

    # If there are conflicts with external commands, return error
    if [[ ${#conflicts[@]} -gt 0 ]]; then
        return 1
    fi

    return 0  # No conflicts
}

# Get command prefix from user
get_command_prefix() {
    local default_prefix="aps"

    # Check for conflicts
    check_command_conflicts "$default_prefix"
    local conflict_status=$?

    case $conflict_status in
        0)
            # No conflicts, use default prefix
            echo "$default_prefix"
            return 0
            ;;
        1)
            # Conflicts with external commands, need user input
            print_warning "The default command prefix 'aps' conflicts with existing commands:"
            print_info "Please choose a different prefix for the Android Proxy Setter commands."
            echo ""
            print_info "Commands will be created as: <prefix>, <prefix>-set, <prefix>-clear, etc."
            echo ""

            while true; do
                read -r -p "Enter command prefix (default: $default_prefix): " user_prefix

                # Use default if empty
                if [[ -z "$user_prefix" ]]; then
                    user_prefix="$default_prefix"
                fi

                # Validate prefix
                if [[ ! "$user_prefix" =~ ^[a-zA-Z][a-zA-Z0-9_-]*$ ]]; then
                    print_error "Invalid prefix. Must start with a letter and contain only letters, numbers, hyphens, and underscores."
                    continue
                fi

                # Check for conflicts with new prefix
                check_command_conflicts "$user_prefix"
                local new_conflict_status=$?

                case $new_conflict_status in
                    0)
                        # No conflicts with new prefix
                        print_success "Using command prefix: $user_prefix"
                        echo "$user_prefix"
                        return 0
                        ;;
                    1)
                        # Still conflicts with external commands
                        print_error "Prefix '$user_prefix' still has conflicts. Please choose a different prefix."
                        ;;
                    2)
                        # All commands already exist and are our tool
                        print_success "Commands with prefix '$user_prefix' are already set up for this tool."
                        echo "$user_prefix"
                        return 0
                        ;;
                esac
            done
            ;;
        2)
            # All commands already exist and are our tool
            print_info "Commands with prefix '$default_prefix' are already set up for this tool."
            echo "$default_prefix"
            return 0
            ;;
    esac
}

# Create shell aliases
create_aliases() {
    local shell_type="$1"
    local config_file="$2"
    local prefix="$3"

    # Check if all commands are already set up for this tool
    check_command_conflicts "$prefix"
    local conflict_status=$?

    if [[ $conflict_status -eq 2 ]]; then
        print_info "All commands with prefix '$prefix' are already set up for this tool, skipping alias creation"
        return 0
    fi

    print_info "Creating shell aliases with prefix: $prefix..."

    # Check if aliases already exist in config file and point to our tool
    if grep -q "# Android Proxy Setter aliases" "$config_file" 2>/dev/null; then
        # Check if the aliases in config file actually point to our tool
        local all_aliases_correct=true
        local commands=("$prefix" "$prefix-set" "$prefix-clear" "$prefix-view" "$prefix-restart")

        for cmd in "${commands[@]}"; do
            # Extract the alias definition from config file
            local alias_line
            alias_line=$(grep "^alias $cmd=" "$config_file" 2>/dev/null || echo "")

            if [[ -n "$alias_line" ]]; then
                # Check if the alias points to android_proxy_setter
                if [[ "$alias_line" != *"android_proxy_setter"* ]]; then
                    all_aliases_correct=false
                    print_warning "Alias '$cmd' in config file does not point to android_proxy_setter"
                    break
                fi
            else
                all_aliases_correct=false
                print_warning "Alias '$cmd' not found in config file"
                break
            fi
        done

        if [[ "$all_aliases_correct" == "true" ]]; then
            print_info "Aliases already exist in $config_file and point to our tool, skipping creation"
            return 0
        else
            print_warning "Aliases exist in $config_file but some don't point to our tool, will recreate them"
            # Remove the existing alias section to recreate it
            sed -i.bak '/^# Android Proxy Setter aliases/,/^alias.*restart.*android_proxy_setter/d' "$config_file" 2>/dev/null || true
        fi
    fi

    case "$shell_type" in
        bash|zsh)
            cat >> "$config_file" << EOF

# Android Proxy Setter aliases
alias ${prefix}='android_proxy_setter'
alias ${prefix}-set='android_proxy_setter --set'
alias ${prefix}-clear='android_proxy_setter --clear'
alias ${prefix}-view='android_proxy_setter --view'
alias ${prefix}-restart='android_proxy_setter --restart-adb'
EOF
            ;;
        fish)
            cat >> "$config_file" << EOF

# Android Proxy Setter aliases
alias ${prefix}='android_proxy_setter'
alias ${prefix}-set='android_proxy_setter --set'
alias ${prefix}-clear='android_proxy_setter --clear'
alias ${prefix}-view='android_proxy_setter --view'
alias ${prefix}-restart='android_proxy_setter --restart-adb'
EOF
            ;;
    esac

    print_success "Created aliases: ${prefix}, ${prefix}-set, ${prefix}-clear, ${prefix}-view, ${prefix}-restart"
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

    # Get command prefix
    local prefix
    prefix=$(get_command_prefix)

    # Create aliases
    create_aliases "$shell_type" "$config_file" "$prefix"

    print_success ""
    print_success "🎉 Installation completed successfully!"
    print_success ""
    print_success "Available commands:"
    print_success "  ${prefix}           - Interactive mode (same as android_proxy_setter)"
    print_success "  ${prefix}-set       - Set proxy directly"
    print_success "  ${prefix}-clear     - Clear proxy directly"
    print_success "  ${prefix}-view      - View current proxy settings"
    print_success "  ${prefix}-restart   - Restart ADB server"
    print_success ""
    print_success "To start using immediately, run:"
    print_success "  source $config_file"
    print_success ""
    print_success "Or simply restart your terminal."
}

# Run main function
main "$@"