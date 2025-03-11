use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use local_ip_address::local_ip;
use std::io::{self, Write};
use std::process::Command;
use std::thread;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Proxy server port
    #[arg(short, long, default_value_t = 8888)]
    port: u16,

    /// Manually specify IP address, automatically get if not specified
    #[arg(short, long)]
    ip: Option<String>,

    /// Skip interactive mode and directly set proxy
    #[arg(short, long)]
    set: bool,

    /// Skip interactive mode and directly clear proxy
    #[arg(short, long)]
    clear: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Check if ADB is available
    println!("Checking if ADB is available...");
    let adb_version = Command::new("adb")
        .arg("version")
        .output()
        .context("Unable to execute ADB command, please ensure ADB is installed and added to PATH")?;
    
    if !adb_version.status.success() {
        anyhow::bail!("Failed to execute ADB command, please ensure ADB is installed and added to PATH");
    }
    
    let version_output = String::from_utf8_lossy(&adb_version.stdout);
    println!("ADB version information: {}", version_output.lines().next().unwrap_or("Unknown"));
    
    // Check device connection status
    println!("Checking device connection status...");
    let devices = Command::new("adb")
        .args(["devices"])
        .output()
        .context("Unable to get the list of connected devices")?;
    
    let devices_output = String::from_utf8_lossy(&devices.stdout);
    println!("Device list:\n{}", devices_output);
    
    let connected_devices: Vec<&str> = devices_output
        .lines()
        .skip(1)
        .filter(|line| !line.trim().is_empty() && !line.contains("List of devices attached"))
        .collect();
    
    if connected_devices.is_empty() {
        anyhow::bail!("No connected Android devices detected, please ensure the device is connected and USB debugging is enabled");
    }
    
    println!("Detected {} connected devices", connected_devices.len());

    // Get current proxy settings
    let current_proxy = Command::new("adb")
        .args(["shell", "settings", "get", "global", "http_proxy"])
        .output()
        .context("Failed to get current proxy settings")?;
    
    let current_proxy_setting = String::from_utf8_lossy(&current_proxy.stdout).trim().to_string();
    
    // Interactive mode or direct action based on flags
    if args.set {
        set_proxy(&args)?;
    } else if args.clear {
        clear_proxy()?;
    } else {
        // Interactive mode
        println!("\n{}", "=== Android Proxy Manager ===".green().bold());
        println!("Current proxy setting: {}", 
            if current_proxy_setting.is_empty() || current_proxy_setting == ":0" { 
                "Not set".red() 
            } else { 
                current_proxy_setting.green() 
            }
        );
        println!("\nPlease select an option:");
        println!("1. {}", "Set global proxy".green());
        println!("2. {}", "Clear global proxy".red());
        println!("3. {}", "Exit".yellow());
        
        print!("\nEnter your choice (1-3): ");
        io::stdout().flush()?;
        
        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        
        match choice.trim() {
            "1" => set_proxy(&args)?,
            "2" => clear_proxy()?,
            "3" => {
                println!("{}", "Exiting...".yellow());
                return Ok(());
            }
            _ => {
                println!("{}", "Invalid choice. Exiting...".red());
                return Ok(());
            }
        }
    }
    
    Ok(())
}

fn set_proxy(args: &Args) -> Result<()> {
    // Get local IP address
    let ip = match &args.ip {
        Some(ip) => ip.clone(),
        None => {
            println!("Getting local IP address...");
            let local_ip = local_ip().context("Unable to get local IP address")?;
            println!("Local IP address obtained: {}", local_ip);
            local_ip.to_string()
        }
    };
    
    let port = args.port;
    
    println!("Preparing to set Android device proxy to {}:{}", ip.green(), port.to_string().green());
    
    // Clear existing proxy settings to ensure new settings take effect
    println!("Clearing existing proxy settings...");
    let clear_proxy = Command::new("adb")
        .args(["shell", "settings", "put", "global", "http_proxy", ":0"])
        .output()
        .context("Failed to clear existing HTTP proxy")?;

    if !clear_proxy.status.success() {
        let error = String::from_utf8_lossy(&clear_proxy.stderr);
        println!("Warning: Failed to clear existing proxy: {}", error);
    }

    // Wait for a short period to ensure the clearing operation is completed
    thread::sleep(std::time::Duration::from_millis(500));

    // Set new proxy
    println!("Setting new proxy to {}:{}", ip.green(), port.to_string().green());
    let set_proxy = Command::new("adb")
        .args(["shell", "settings", "put", "global", "http_proxy", &format!("{}:{}", ip, port)])
        .output()
        .context("Failed to set HTTP proxy")?;

    if !set_proxy.status.success() {
        let error = String::from_utf8_lossy(&set_proxy.stderr);
        anyhow::bail!("Failed to set proxy: {}", error);
    }

    // Wait for a short period to ensure the setting operation is completed
    thread::sleep(std::time::Duration::from_millis(500));

    // Verify proxy settings
    println!("Verifying proxy settings...");
    let verify_proxy = Command::new("adb")
        .args(["shell", "settings", "get", "global", "http_proxy"])
        .output()
        .context("Failed to verify proxy settings")?;
    
    if verify_proxy.status.success() {
        let proxy_setting = String::from_utf8_lossy(&verify_proxy.stdout).trim().to_string();
        println!("Current proxy settings: {}", proxy_setting.green());
        
        if proxy_setting == format!("{}:{}", ip, port) {
            println!("{}", "✅ Successfully set Android device proxy!".green().bold());
        } else {
            println!("{}", "⚠️ Proxy settings may be incorrect, please verify manually".yellow().bold());
        }
    } else {
        println!("{}", "⚠️ Unable to verify proxy settings, please verify manually".yellow().bold());
    }
    
    Ok(())
}

fn clear_proxy() -> Result<()> {
    println!("{}", "Clearing Android device proxy settings...".yellow());
    
    let clear_proxy = Command::new("adb")
        .args(["shell", "settings", "put", "global", "http_proxy", ":0"])
        .output()
        .context("Failed to clear HTTP proxy")?;

    if !clear_proxy.status.success() {
        let error = String::from_utf8_lossy(&clear_proxy.stderr);
        anyhow::bail!("Failed to clear proxy: {}", error);
    }

    // Verify proxy settings
    println!("Verifying proxy settings...");
    let verify_proxy = Command::new("adb")
        .args(["shell", "settings", "get", "global", "http_proxy"])
        .output()
        .context("Failed to verify proxy settings")?;
    
    if verify_proxy.status.success() {
        let proxy_setting = String::from_utf8_lossy(&verify_proxy.stdout).trim().to_string();
        
        if proxy_setting.is_empty() || proxy_setting == ":0" {
            println!("{}", "✅ Successfully cleared Android device proxy!".green().bold());
        } else {
            println!("Current proxy settings: {}", proxy_setting);
            println!("{}", "⚠️ Proxy settings may not be cleared properly, please verify manually".yellow().bold());
        }
    } else {
        println!("{}", "⚠️ Unable to verify proxy settings, please verify manually".yellow().bold());
    }
    
    Ok(())
}
