#!/usr/bin/env bash

set -euo pipefail

readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m'
readonly BLOCK_START='# >>> Android Proxy Setter >>>'
readonly BLOCK_END='# <<< Android Proxy Setter <<<'
readonly BINARY_NAME='android_proxy_setter'
readonly COMMAND_PREFIX='aps'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

info() { printf "%b[INFO]%b %s\n" "$BLUE" "$NC" "$*" >&2; }
success() { printf "%b[SUCCESS]%b %s\n" "$GREEN" "$NC" "$*" >&2; }
error() { printf "%b[ERROR]%b %s\n" "$RED" "$NC" "$*" >&2; }

detect_shell() {
    case "${SHELL##*/}" in
        bash|zsh|fish) printf '%s\n' "${SHELL##*/}" ;;
        *)
            error "Unsupported shell '${SHELL##*/}'. Supported shells: bash, zsh, fish."
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

validate_managed_block() {
    local config_file="$1"
    awk -v start="$BLOCK_START" -v end="$BLOCK_END" '
        $0 == start { if (managed) invalid = 1; managed = 1; next }
        $0 == end { if (!managed) invalid = 1; managed = 0; next }
        END { exit invalid || managed ? 1 : 0 }
    ' "$config_file" || {
        error "Managed block markers are incomplete in $config_file; no changes were made."
        exit 1
    }
}

has_conflicting_prefix() {
    local config_file="$1"
    awk -v start="$BLOCK_START" -v end="$BLOCK_END" -v prefix="$COMMAND_PREFIX" '
        $0 == start { managed = 1; next }
        $0 == end { managed = 0; next }
        !managed && $0 ~ "^alias " prefix "(=|-(set|clear|view|restart|help)=)" { found = 1 }
        !managed && $0 ~ "^" prefix "(-(set|clear|view|restart|help))?\\(\\)" { found = 1 }
        END { exit found ? 0 : 1 }
    ' "$config_file"
}

write_managed_config() {
    local config_file="$1"
    local shell_type="$2"
    local install_dir="$3"
    local temp_file config_target link_target
    local symlink_count=0
    config_target="$config_file"
    while [[ -L "$config_target" ]]; do
        symlink_count=$((symlink_count + 1))
        [[ $symlink_count -le 20 ]] || {
            error "Too many symbolic links while resolving $config_file."
            exit 1
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

    {
        printf '%s\n' "$BLOCK_START"
        if [[ "$shell_type" == fish ]]; then
            printf 'contains -- "%s" $PATH; or set -gx PATH "%s" $PATH\n' "$install_dir" "$install_dir"
        else
            printf 'case ":$PATH:" in *":%s:"*) ;; *) export PATH="%s:$PATH" ;; esac\n' "$install_dir" "$install_dir"
        fi
        printf "alias %s='android_proxy_setter'\n" "$COMMAND_PREFIX"
        printf "alias %s-set='android_proxy_setter --set'\n" "$COMMAND_PREFIX"
        printf "alias %s-clear='android_proxy_setter --clear'\n" "$COMMAND_PREFIX"
        printf "alias %s-view='android_proxy_setter --view'\n" "$COMMAND_PREFIX"
        printf "alias %s-restart='android_proxy_setter --restart-adb'\n" "$COMMAND_PREFIX"
        printf "alias %s-help='android_proxy_setter --help-commands'\n" "$COMMAND_PREFIX"
        printf '%s\n' "$BLOCK_END"
    } >> "$temp_file"
    command mv "$temp_file" "$config_target"
    trap - RETURN
}

main() {
    local shell_type config_file install_dir binary_source resolved_command command_name

    [[ $# -eq 0 ]] || {
        error "The installer does not accept arguments. It installs the 'aps' aliases."
        exit 1
    }

    shell_type="$(detect_shell)"
    config_file="$(shell_config "$shell_type")"
    install_dir="$HOME/.local/bin"
    binary_source="$PROJECT_ROOT/target/release/$BINARY_NAME"

    [[ -f "$binary_source" ]] || {
        error "Release binary not found. Run 'cargo build --release' first."
        exit 1
    }

    mkdir -p "$install_dir" "$(dirname "$config_file")"
    touch "$config_file"
    validate_managed_block "$config_file"
    if has_conflicting_prefix "$config_file"; then
        error "The 'aps' aliases conflict with existing shell configuration."
        exit 1
    fi
    for command_name in aps aps-set aps-clear aps-view aps-restart aps-help; do
        resolved_command="$(command -v "$command_name" 2>/dev/null || true)"
        if [[ -n "$resolved_command" ]] && [[ "$resolved_command" != "$install_dir/$BINARY_NAME" ]]; then
            error "Command '$command_name' already resolves to $resolved_command."
            exit 1
        fi
    done
    install -m 755 "$binary_source" "$install_dir/$BINARY_NAME"

    write_managed_config "$config_file" "$shell_type" "$install_dir"

    success "Installed $BINARY_NAME to $install_dir."
    info "Installed aliases: aps, aps-set, aps-clear, aps-view, aps-restart, aps-help."
    info "Restart your terminal or run: source $config_file"
}

main "$@"
