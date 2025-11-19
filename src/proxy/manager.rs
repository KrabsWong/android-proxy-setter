//! Proxy management operations

use std::thread;
use std::time::Duration;
use colored::*;
use crate::error::{AppError, AppResult};
use crate::adb::commands::{AdbCommand, execute_adb_command, execute_adb_command_string};
use crate::proxy::settings::ProxySettings;

/// Set proxy on Android device
pub fn set_proxy(settings: &ProxySettings) -> AppResult<()> {
    let proxy_string = settings.to_proxy_string();

    println!(
        "Preparing to set Android device proxy to {}:{}",
        settings.ip.green(),
        settings.port.to_string().green()
    );

    // Clear existing proxy settings first
    println!("Clearing existing proxy settings...");
    let _ = clear_proxy_internal(); // Ignore errors for clearing

    // Wait briefly to ensure clearing is complete
    thread::sleep(Duration::from_millis(500));

    // Set new proxy
    println!(
        "Setting new proxy to {}:{}",
        settings.ip.green(),
        settings.port.to_string().green()
    );

    execute_adb_command(AdbCommand::SetProxy(proxy_string.clone()))?;

    // Wait for setting to take effect
    thread::sleep(Duration::from_millis(500));

    // Verify proxy settings
    verify_proxy_settings(&proxy_string)?;

    println!(
        "{}",
        "✅ Successfully set Android device proxy!".green().bold()
    );

    Ok(())
}

/// Clear proxy settings on Android device
pub fn clear_proxy() -> AppResult<()> {
    println!("{}", "Clearing Android device proxy settings...".yellow());

    clear_proxy_internal()?;

    // Wait for clearing to take effect
    thread::sleep(Duration::from_millis(500));

    // Verify proxy is cleared
    verify_proxy_cleared()?;

    println!(
        "{}",
        "✅ Successfully cleared Android device proxy!".green().bold()
    );

    Ok(())
}

/// View current proxy settings
pub fn view_proxy() -> AppResult<()> {
    println!(
        "{}",
        "Checking current Android device proxy settings...".blue()
    );

    let proxy_setting = get_current_proxy_setting()?;

    println!("\n{}", "=== Current Proxy Settings ===".blue().bold());
    if proxy_setting.is_empty() || proxy_setting == ":0" {
        println!("Global HTTP Proxy: {}", "Not set".red());
    } else {
        // Split the proxy setting into IP and port
        if let Some((ip, port)) = proxy_setting.split_once(':') {
            println!("Global HTTP Proxy: {}", proxy_setting.green());
            println!("IP Address: {}", ip.green());
            println!("Port: {}", port.green());
        } else {
            println!("Global HTTP Proxy: {}", proxy_setting.green());
            println!("(Unable to parse IP and port separately)");
        }
    }

    println!("\nPress Enter to continue...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(())
}

/// View current proxy settings without waiting for user input
pub fn view_proxy_direct() -> AppResult<()> {
    let proxy_setting = get_current_proxy_setting()?;

    println!("Current Android Proxy Settings:");
    if proxy_setting.is_empty() || proxy_setting == ":0" {
        println!("Global HTTP Proxy: {}", "Not set".red());
    } else {
        // Split the proxy setting into IP and port
        if let Some((ip, port)) = proxy_setting.split_once(':') {
            println!("Global HTTP Proxy: {}", proxy_setting.green());
            println!("IP Address: {}", ip.green());
            println!("Port: {}", port.green());
        } else {
            println!("Global HTTP Proxy: {}", proxy_setting.green());
        }
    }

    Ok(())
}

/// Get proxy information as a string (for GUI mode)
pub fn get_proxy_info() -> AppResult<String> {
    let proxy_setting = get_current_proxy_setting()?;

    let mut info = String::from("Current Proxy Settings:\n");
    if proxy_setting.is_empty() || proxy_setting == ":0" {
        info.push_str("Global HTTP Proxy: Not set");
    } else {
        // Split the proxy setting into IP and port
        if let Some((ip, port)) = proxy_setting.split_once(':') {
            info.push_str(&format!("Global HTTP Proxy: {}\n", proxy_setting));
            info.push_str(&format!("IP Address: {}\n", ip));
            info.push_str(&format!("Port: {}", port));
        } else {
            info.push_str(&format!("Global HTTP Proxy: {}\n", proxy_setting));
            info.push_str("(Unable to parse IP and port separately)");
        }
    }

    Ok(info)
}

// Internal helper functions

fn clear_proxy_internal() -> AppResult<()> {
    execute_adb_command(AdbCommand::ClearProxy)
        .map_err(|e| AppError::proxy_clear_failed(e.to_string()))
        .map(|_| ())
}

fn get_current_proxy_setting() -> AppResult<String> {
    execute_adb_command_string(AdbCommand::GetProxy)
        .map_err(|e| AppError::proxy_get_failed(e.to_string()))
}

fn verify_proxy_settings(expected_proxy: &str) -> AppResult<()> {
    println!("Verifying proxy settings...");
    let current_proxy = get_current_proxy_setting()?;

    if current_proxy == expected_proxy {
        println!("Current proxy settings: {}", current_proxy.green());
    } else {
        println!(
            "{}",
            "⚠️ Proxy settings may be incorrect, please verify manually"
                .yellow()
                .bold()
        );
        println!("Expected: {}", expected_proxy.green());
        println!("Actual: {}", current_proxy.yellow());
    }

    Ok(())
}

fn verify_proxy_cleared() -> AppResult<()> {
    println!("Verifying proxy settings...");
    let current_proxy = get_current_proxy_setting()?;

    if current_proxy.is_empty() || current_proxy == ":0" {
        println!("Current proxy settings: {}", "Not set".green());
    } else {
        println!("Current proxy settings: {}", current_proxy);
        println!(
            "{}",
            "⚠️ Proxy settings may not be cleared properly, please verify manually"
                .yellow()
                .bold()
        );
    }

    Ok(())
}