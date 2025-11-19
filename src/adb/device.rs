//! Device management and ADB availability checking

use std::process::Command;
use crate::error::{AppError, AppResult};
use crate::adb::commands::{AdbCommand, execute_adb_command_string};

/// Check if ADB is available in the system PATH
pub fn check_adb_availability() -> AppResult<String> {
    let output = Command::new("adb")
        .arg("version")
        .output()
        .map_err(|_| AppError::AdbNotFound)?;

    if !output.status.success() {
        return Err(AppError::AdbNotFound);
    }

    let version_output = String::from_utf8_lossy(&output.stdout);
    let version_line = version_output
        .lines()
        .next()
        .unwrap_or("Unknown")
        .to_string();

    Ok(version_line)
}

/// Get list of connected Android devices
pub fn get_connected_devices() -> AppResult<Vec<String>> {
    let output = execute_adb_command_string(AdbCommand::GetDevices)?;

    let devices: Vec<String> = output
        .lines()
        .skip(1) // Skip header line
        .filter(|line| !line.trim().is_empty() && !line.contains("List of devices attached"))
        .map(|line| line.split('\t').next().unwrap_or(line).trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    if devices.is_empty() {
        return Err(AppError::NoDevicesConnected);
    }

    Ok(devices)
}

/// Check if ADB server is running
pub fn is_adb_running() -> bool {
    let output = Command::new("pgrep")
        .arg("adb")
        .output();

    match output {
        Ok(output) => {
            let pids: Vec<u32> = String::from_utf8_lossy(&output.stdout)
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            !pids.is_empty()
        }
        Err(_) => false,
    }
}

/// Restart ADB server using the included shell script
pub fn restart_adb_server() -> AppResult<()> {
    const SCRIPT_CONTENT: &str = include_str!("../bin/resurrection_adb.sh");

    let status = Command::new("sh")
        .arg("-c")
        .arg(SCRIPT_CONTENT)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(AppError::adb_command_failed(
            "restart ADB server",
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to restart ADB server")
        ))
    }
}