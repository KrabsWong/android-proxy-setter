# Android Proxy Setting Tool

This is a simple command-line tool to set the network proxy of an Android device connected to the computer to the local IP and specified port.

## Prerequisites

- Rust development environment installed
- ADB installed and added to system PATH
- Android device connected to the computer with USB debugging enabled

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

### Basic Usage

```bash
# Use the default port 8083
./target/release/android_proxy_setter

# Specify the port
./target/release/android_proxy_setter --port 8888
```

### Command Line Arguments

- `-p, --port <PORT>`: Set the proxy port (default is 8083)
- `-i, --ip <IP>`: Manually specify the IP address (default is to automatically get the local IP)
- `-h, --help`: Display help information
- `-V, --version`: Display version information

### Clear Proxy Settings

To clear the proxy settings on the Android device, you can run the following command:

```bash
adb shell settings put global http_proxy :0
```

## Notes

- Ensure the Android device and computer are on the same network
- Some Android devices may require different settings; this tool uses the most common global HTTP proxy setting method
- Some applications may ignore system proxy settings
