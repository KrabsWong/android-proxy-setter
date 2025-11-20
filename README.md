# Android Proxy Setting Tool

A command-line tool written in Rust to manage HTTP proxy settings on connected Android devices through ADB. The tool provides both interactive CLI and direct command-line options for proxy management.

## Prerequisites

- Rust development environment installed
- ADB installed and added to system PATH
- Android device connected to the computer with USB debugging enabled

## Installation

### Quick Installation (Recommended)

```bash
# Clone the repository
git clone https://github.com/KrabsWong/android_proxy_setter.git
cd android_proxy_setter

# Build and install to your PATH
make install
```

This will:
- Build the release version
- Install the binary to `~/.local/bin/`
- Add the directory to your PATH
- Create convenient shell aliases

## Usage

### Available Commands
- `aps-help` - Show available commands and aliases
- `aps` - Interactive mode
- `aps-set` - Set proxy directly
- `aps-clear` - Clear proxy directly
- `aps-view` - View current proxy settings
- `aps-restart` - Restart ADB server

### Command Line Arguments

- `-p, --port <PORT>`: Set the proxy port (default is 8083)
- `-i, --ip <IP>`: Manually specify the IP address (default is to automatically get the local IP)
- `-s, --set`: Skip interactive mode and directly set proxy
- `-c, --clear`: Skip interactive mode and directly clear proxy
- `--restart-adb`: Skip interactive mode and directly restart ADB server
- `--view`: Skip interactive mode and directly view proxy settings
- `--help-commands`: Show available commands and aliases
- `-h, --help`: Display help information
- `-V, --version`: Display version information

### Alternative Method to Clear Proxy Settings

If you prefer to use ADB directly to clear proxy settings, you can run:

```bash
adb shell settings put global http_proxy :0
```

## Project Architecture

This project has been refactored from a single-file architecture to a modular design:

```
src/
├── main.rs              # Application entry point
├── cli/
│   ├── mod.rs           # CLI module exports
│   └── interactive.rs   # Interactive mode implementation
├── config/
│   ├── mod.rs           # Configuration module exports
│   └── args.rs          # Command-line argument parsing
├── proxy/
│   ├── mod.rs           # Proxy module exports
│   ├── manager.rs       # Proxy management logic
│   └── settings.rs      # Proxy settings handling
└── adb/
    ├── mod.rs           # ADB module exports
    ├── device.rs        # Device management
    └── commands.rs      # ADB command execution
```

## Features

- Automatic restart ADB
- Automatic detection of local IP address
- Support for multiple connected Android devices
- Interactive CLI menu for easy proxy management
- Direct command-line options for scripting and automation
- Verification of proxy settings after changes
- Colored output for better readability in CLI mode
- Modular architecture for maintainability
- Help command to show available aliases and options


## Notes

- Ensure the Android device and computer are on the same network
- Some Android devices may require different settings; this tool uses the most common global HTTP proxy setting method
- Some applications may ignore system proxy settings
- This tool is now CLI-only and does not include GUI functionality
