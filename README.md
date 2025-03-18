# Android Proxy Setting Tool

This is a tool to set the network proxy of an Android device connected to the computer to the local IP and specified port. It provides both interactive command-line interface, direct command-line options, and a graphical user interface (GUI) with system tray icon for managing proxy settings.

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

```bash
# Clone the repository
git clone https://github.com/KrabsWong/android_proxy_setter.git
cd android_proxy_setter

# Build the project
cargo build --release
```

The compiled executable is located at `target/release/android_proxy_setter`.

## Usage

The tool offers three modes of operation:

### 1. Interactive CLI Mode

By default, the tool runs in interactive mode, providing a menu with the following options:

1. Set global proxy
2. Clear global proxy
3. View current proxy settings
4. Exit

```bash
# Run in interactive mode
./target/release/android_proxy_setter
```

### 2. Direct Command Mode

You can also use command-line flags to perform actions directly without the interactive menu:

```bash
# Set proxy with default port (8083)
./target/release/android_proxy_setter --set

# Set proxy with specific port
./target/release/android_proxy_setter --set --port 8080

# Clear proxy settings
./target/release/android_proxy_setter --clear
```

### 3. GUI Mode with System Tray Icon

The tool now supports a graphical user interface mode with a system tray icon for easy access:

```bash
# Run in GUI mode
./target/release/android_proxy_setter --gui
```

In GUI mode, you can:

- Set proxy settings directly from the system tray menu
- Clear proxy settings
- View current proxy settings
- Receive notifications via dialog boxes upon success or failure

### Command Line Arguments

- `-p, --port <PORT>`: Set the proxy port (default is 8083)
- `-i, --ip <IP>`: Manually specify the IP address (default is to automatically get the local IP)
- `-s, --set`: Skip interactive mode and directly set proxy
- `-c, --clear`: Skip interactive mode and directly clear proxy
- `-g, --gui`: Run in GUI mode with system tray icon
- `-h, --help`: Display help information
- `-V, --version`: Display version information

### Alternative Method to Clear Proxy Settings

If you prefer to use ADB directly to clear proxy settings, you can run:

```bash
adb shell settings put global http_proxy :0
```

## Features

- Automatic detection of local IP address
- Support for multiple connected Android devices
- Interactive CLI menu for easy proxy management
- GUI mode with system tray integration for desktop environments
- Direct command-line options for scripting and automation
- Verification of proxy settings after changes
- Colored output for better readability in CLI mode
- Notification dialogs in GUI mode

## Packaging and Distribution

### macOS

To create a macOS application bundle:

```bash
# Install cargo-bundle
cargo install cargo-bundle

# Build the application bundle
cargo bundle --release
```

The bundled application will be available at `target/release/bundle/osx/Android Proxy Setter.app`.

To create a DMG file for easy distribution:

```bash
create-dmg "target/release/bundle/osx/Android Proxy Setter.app"
```

## Notes

- Ensure the Android device and computer are on the same network
- Some Android devices may require different settings; this tool uses the most common global HTTP proxy setting method
- Some applications may ignore system proxy settings
- GUI mode requires a desktop environment that supports system tray icons
