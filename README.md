# Android Proxy Setting Tool

This is a command-line tool to set the network proxy of an Android device connected to the computer to the local IP and specified port. It provides both interactive and direct command-line options for managing proxy settings.

## Prerequisites

- Rust development environment installed
- ADB installed and added to system PATH
- Android device connected to the computer with USB debugging enabled

<img width="500" alt="List devices and select action" src="https://github.com/user-attachments/assets/1342d15d-d7bb-4597-ac55-14b757fbfd0a" />

<img width="500" alt="Set global http proxy and get the response" src="https://github.com/user-attachments/assets/30a109d7-3c54-437f-b3de-ae55cc6d66cc" />

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

### Interactive Mode

By default, the tool runs in interactive mode, providing a menu with the following options:

1. Set global proxy
2. Clear global proxy
3. View current proxy settings
4. Exit

```bash
# Run in interactive mode
./target/release/android_proxy_setter
```

### Direct Command Mode

You can also use command-line flags to perform actions directly without the interactive menu:

```bash
# Set proxy with default port (8888)
./target/release/android_proxy_setter --set

# Set proxy with specific port
./target/release/android_proxy_setter --set --port 8080

# Clear proxy settings
./target/release/android_proxy_setter --clear
```

### Command Line Arguments

- `-p, --port <PORT>`: Set the proxy port (default is 8888)
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

## Features

- Automatic detection of local IP address
- Support for multiple connected Android devices
- Interactive menu for easy proxy management
- Direct command-line options for scripting and automation
- Verification of proxy settings after changes
- Colored output for better readability

## Notes

- Ensure the Android device and computer are on the same network
- Some Android devices may require different settings; this tool uses the most common global HTTP proxy setting method
- Some applications may ignore system proxy settings
