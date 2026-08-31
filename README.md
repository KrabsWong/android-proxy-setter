# Android Proxy Setter

A Rust command-line tool for managing the global HTTP proxy on an Android device through ADB. It supports an interactive menu and direct commands for scripts.

## Prerequisites

- Rust toolchain
- ADB available on `PATH`
- An Android device with USB debugging enabled

## Installation

```bash
git clone https://github.com/KrabsWong/android-proxy-setter.git
cd android-proxy-setter
make install
```

The installer builds the release binary, installs it to `~/.local/bin`, updates the active Bash, Zsh, or Fish configuration, and creates the `aps` aliases. Restart the terminal after installation, or source the configuration file printed by the installer.

## Usage

Run `aps` to open the interactive menu. Use `↑`/`↓` to select an action, `Enter` to run it, and `Esc` or `q` to cancel. The program exits after one action.

The installer also creates these direct aliases:

- `aps-set` — set the proxy
- `aps-clear` — clear the proxy
- `aps-view` — view the current proxy
- `aps-restart` — restart the ADB server
- `aps-help` — show the command summary

### Options

- `-p, --port <PORT>` — proxy port; defaults to `8017`
- `-i, --ip <IP>` — proxy IP; automatically detects the local IP when omitted
- `-d, --device <SERIAL>` — target device; required when multiple ready devices are connected
- `-s, --set` — set the proxy without opening the menu
- `-c, --clear` — clear the proxy without opening the menu
- `--view` — show the current proxy without opening the menu
- `--restart-adb` — restart the ADB server
- `--help-commands` — show the installed aliases and common options

All options can also be passed to `aps`. For example:

```bash
aps --set --ip 192.168.1.10 --port 8017 --device <SERIAL>
```

## Uninstallation

```bash
make uninstall
```

## Behavior and limitations

- Ensure the Android device and computer are on the same network
- Proxy changes are verified; a failed set operation attempts to restore the previous value
- ADB commands time out instead of hanging indefinitely
- The tool changes Android's `global http_proxy` setting
- Some applications may ignore system proxy settings
