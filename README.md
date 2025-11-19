# Android Proxy Setting Tool

This is a tool to set the network proxy of an Android device connected to the computer to the local IP and specified port. It provides both interactive command-line interface and direct command-line options for managing proxy settings.

## Prerequisites

- Rust development environment installed
- ADB installed and added to system PATH
- Android device connected to the computer with USB debugging enabled

<img width="500" alt="List devices and select action" src="https://github.com/user-attachments/assets/e1ec8d15-a354-47b4-84f8-47c6204349a1" />

<br />

<img width="500" alt="Set global http proxy and get the response" src="https://github.com/user-attachments/assets/29f82b92-a2e8-47f4-b167-a1b8bd8f8034" />

<br />

<img width="500" alt="Gui mode" src="https://github.com/user-attachments/assets/577c5016-e1eb-4c17-ab45-285a5219f052" />


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

### Manual Installation

If you prefer manual installation:

```bash
# Clone the repository
git clone https://github.com/KrabsWong/android_proxy_setter.git
cd android_proxy_setter

# Build the project
cargo build --release

# Install using the script
./install.sh
```

### Development Installation

For development purposes:

```bash
# Build development version
make dev

# Or use cargo directly
cargo build
```

The compiled executable is located at `target/release/android_proxy_setter` (release) or `target/debug/android_proxy_setter` (debug).

## Usage

After installation, you can use the tool with convenient aliases:

### Using Aliases (Recommended)

```bash
# Interactive mode (shows menu with options)
aps

# Set proxy with default port (8083)
aps-set

# Set proxy with specific port
aps-set --port 8080

# Clear proxy settings
aps-clear

# View current proxy settings
aps-view

# Restart ADB server
aps-restart
```

### Using Full Command

You can also use the full command name:

```bash
# Interactive mode
android_proxy_setter

# Direct commands
android_proxy_setter --set
android_proxy_setter --clear
android_proxy_setter --set --port 8080
```

### Available Aliases

- `aps` - Interactive mode (same as `android_proxy_setter`)
- `aps-set` - Set proxy directly
- `aps-clear` - Clear proxy directly
- `aps-view` - View current proxy settings
- `aps-restart` - Restart ADB server

### Command Line Arguments

- `-p, --port <PORT>`: Set the proxy port (default is 8083)
- `-i, --ip <IP>`: Manually specify the IP address (default is to automatically get the local IP)
- `-s, --set`: Skip interactive mode and directly set proxy
- `-c, --clear`: Skip interactive mode and directly clear proxy
- `-h, --help`: Display help information
- `-V, --version`: Display version information

### Alternative Method to Clear Proxy Settings

If you prefer to use ADB directly to clear proxy settings, you can run:

```bash
adb shell settings put global http_proxy :0
```

## Development Commands

This project includes a Makefile for common development tasks:

```bash
# Build release version
make build

# Build development version
make dev

# Run tests
make test

# Check code
make check

# Format code
make fmt

# Lint code
make lint

# Clean build artifacts
make clean

# Show all available commands
make help
```

## Features

- Automatic restart ADB
- Automatic detection of local IP address
- Support for multiple connected Android devices
- Interactive CLI menu for easy proxy management
- Direct command-line options for scripting and automation
- Verification of proxy settings after changes
- Colored output for better readability in CLI mode


## Notes

- Ensure the Android device and computer are on the same network
- Some Android devices may require different settings; this tool uses the most common global HTTP proxy setting method
- Some applications may ignore system proxy settings
