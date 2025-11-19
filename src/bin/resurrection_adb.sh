#!/bin/bash

# resurrection_adb.sh - Forcefully restart ADB server
# This script kills all running ADB processes and restarts the ADB server

set -euo pipefail

# Color codes for better output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if ADB is available
check_adb_availability() {
    if ! command -v adb >/dev/null 2>&1; then
        print_error "ADB not found in PATH. Please ensure Android SDK is installed."
        exit 1
    fi
}

# Function to kill ADB processes
kill_adb_processes() {
    print_info "Searching for ADB processes..."

    # Use pgrep with full command matching to avoid matching unrelated processes
    local pids
    pids=$(pgrep -f "^adb" 2>/dev/null || true)

    if [ -z "$pids" ]; then
        print_info "No running ADB processes found."
        return 0
    fi

    print_info "Found ADB process PIDs: $pids"
    print_info "Terminating ADB processes..."

    local killed_count=0
    local failed_count=0

    # Kill processes more safely with individual error handling
    for pid in $pids; do
        if kill -9 "$pid" 2>/dev/null; then
            print_info "Terminated PID: $pid"
            killed_count=$((killed_count + 1))
        else
            print_warning "Failed to terminate PID: $pid"
            failed_count=$((failed_count + 1))
        fi
    done

    # Wait a moment for processes to fully terminate
    sleep 1

    # Check if any ADB processes remain
    local remaining_pids
    remaining_pids=$(pgrep -f "^adb" 2>/dev/null || true)

    if [ -n "$remaining_pids" ]; then
        print_warning "Some ADB processes might still be running: $remaining_pids"
        return 1
    else
        print_success "All ADB processes terminated successfully ($killed_count killed, $failed_count failed)."
        return 0
    fi
}

# Function to start ADB server
start_adb_server() {
    print_info "Starting ADB server..."

    # Start ADB server with error handling
    if adb start-server 2>/dev/null; then
        print_success "ADB server started successfully."

        # Verify server is working by listing devices
        if timeout 10s adb devices >/dev/null 2>&1; then
            print_success "ADB server verification successful."
            return 0
        else
            print_warning "ADB server started but device listing timed out or failed."
            return 1
        fi
    else
        print_error "Failed to start ADB server."
        return 1
    fi
}

# Main execution
main() {
    print_info "Starting ADB resurrection process..."

    # Check if ADB is available
    check_adb_availability

    # Kill existing ADB processes
    if ! kill_adb_processes; then
        print_warning "Some ADB processes may not have been terminated properly."
    fi

    # Start ADB server
    if start_adb_server; then
        print_success "ADB resurrection completed successfully."
        exit 0
    else
        print_error "ADB resurrection failed. Please check ADB installation and try again."
        exit 1
    fi
}

# Run main function
main "$@"
