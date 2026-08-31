#!/usr/bin/env bash

set -euo pipefail

readonly GREEN='\033[0;32m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m'
readonly BLOCK_START='# >>> Android Proxy Setter >>>'
readonly BLOCK_END='# <<< Android Proxy Setter <<<'
readonly BINARY_NAME='android_proxy_setter'

info() { printf "%b[INFO]%b %s\n" "$BLUE" "$NC" "$*"; }
success() { printf "%b[SUCCESS]%b %s\n" "$GREEN" "$NC" "$*"; }

detect_shell() {
    case "${SHELL##*/}" in
        bash|zsh|fish) printf '%s\n' "${SHELL##*/}" ;;
        *)
            info "Unsupported shell '${SHELL##*/}'. Supported shells: bash, zsh, fish."
            return 1
            ;;
    esac
}

shell_config() {
    case "$1" in
        bash) printf '%s\n' "$HOME/.bashrc" ;;
        zsh) printf '%s\n' "$HOME/.zshrc" ;;
        fish) printf '%s\n' "$HOME/.config/fish/config.fish" ;;
    esac
}

remove_managed_block() {
    local config_file="$1"
    local temp_file config_target link_target
    local symlink_count=0

    [[ -f "$config_file" ]] || return 0
    if ! awk -v start="$BLOCK_START" -v end="$BLOCK_END" '
        $0 == start { if (managed) invalid = 1; managed = 1; next }
        $0 == end { if (!managed) invalid = 1; managed = 0; next }
        END { exit invalid || managed ? 1 : 0 }
    ' "$config_file"; then
        info "Managed block markers are incomplete in $config_file; shell configuration was left unchanged."
        return 1
    fi

    config_target="$config_file"
    while [[ -L "$config_target" ]]; do
        symlink_count=$((symlink_count + 1))
        [[ $symlink_count -le 20 ]] || {
            info "Too many symbolic links while resolving $config_file; no changes were made."
            return 1
        }
        link_target="$(readlink "$config_target")"
        if [[ "$link_target" == /* ]]; then
            config_target="$link_target"
        else
            config_target="$(dirname "$config_target")/$link_target"
        fi
    done
    config_target="$(cd "$(dirname "$config_target")" && pwd -P)/$(basename "$config_target")"
    temp_file="$(mktemp "${config_target}.tmp.XXXXXX")"
    trap 'rm -f "${temp_file:-}"' RETURN
    cp -p "$config_target" "$temp_file"
    awk -v start="$BLOCK_START" -v end="$BLOCK_END" '
        $0 == start { managed = 1; next }
        $0 == end { managed = 0; next }
        !managed { print }
    ' "$config_file" > "$temp_file"
    command mv "$temp_file" "$config_target"
    trap - RETURN
}

main() {
    local shell_type config_file install_dir binary_path
    shell_type="$(detect_shell)"
    config_file="$(shell_config "$shell_type")"
    install_dir="$HOME/.local/bin"
    binary_path="$install_dir/$BINARY_NAME"

    remove_managed_block "$config_file"

    if [[ -f "$binary_path" ]]; then
        rm "$binary_path"
        success "Removed $binary_path."
    else
        info "Binary is already absent: $binary_path"
    fi

    success "Removed managed shell configuration from $config_file."

    rmdir "$install_dir" 2>/dev/null || true

    info "Restart your terminal or run: source $config_file"
}

main "$@"
