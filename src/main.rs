//! Android Proxy Setter - A tool to manage Android device proxy settings
//!
//! This application provides a command-line interface for managing
//! HTTP proxy settings on connected Android devices through ADB.

// Internal modules
mod cli;
mod adb;
mod proxy;
mod config;
mod error;

// Re-exports for cleaner usage
use crate::config::args::parse_args;
use crate::error::AppResult;
use crate::adb::device::{check_adb_availability, get_connected_devices, is_adb_running, restart_adb_server};
use crate::proxy::manager::{get_proxy_info, view_proxy_direct};
use crate::cli::run_cli_mode;

fn main() -> AppResult<()> {
    // Parse command-line arguments
    let args = parse_args();

    // For view-only mode, skip all initialization and directly show proxy info
    if args.view {
        return view_proxy_only();
    }

    // Check if ADB is running and restart if necessary
    check_and_restart_adb()?;

    // Check ADB availability
    check_adb_environment()?;

    // Check device connection status
    check_device_connection()?;

    // Get current proxy settings for display
    let current_proxy_setting = get_current_proxy_setting()?;

    // Run CLI mode
    run_cli_mode(args, current_proxy_setting)
}

/// Check if ADB is running and restart if necessary
fn check_and_restart_adb() -> AppResult<()> {
    if !is_adb_running() {
        println!("Adb is not running, I will restart it directly.\n----------------------------");
        restart_adb_server()?;
        println!("Adb is restarted, \n----------------------------");
    }
    Ok(())
}

/// Check ADB environment and availability
fn check_adb_environment() -> AppResult<()> {
    println!("Checking if ADB is available...");
    let version_info = check_adb_availability()?;
    println!("ADB version information: {}", version_info);
    Ok(())
}

/// Check device connection status
fn check_device_connection() -> AppResult<()> {
    println!("Checking device connection status...");
    let devices = get_connected_devices()?;

    println!("Device list:");
    for device in &devices {
        println!("  - {}", device);
    }

    println!("Detected {} connected devices", devices.len());
    Ok(())
}

/// Get current proxy settings
fn get_current_proxy_setting() -> AppResult<String> {
    Ok(get_proxy_info()
        .map(|info| {
            // Extract just the proxy setting from the info string
            if info.contains("Not set") {
                String::new()
            } else if let Some(line) = info.lines().find(|line| line.contains("Global HTTP Proxy")) {
                line.split(": ").nth(1).unwrap_or("").to_string()
            } else {
                String::new()
            }
        })
        .unwrap_or_else(|_| String::new()))
}

/// View proxy settings only, without any initialization checks
fn view_proxy_only() -> AppResult<()> {
    view_proxy_direct()
}

