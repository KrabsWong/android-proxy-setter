use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use local_ip_address::local_ip;
use std::io::{self, Write};
use std::process::Command;
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// GUI imports
use winit::event_loop::{ControlFlow, EventLoop};
use tray_icon::{TrayIconBuilder, menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem}};
use tray_icon::icon::Icon as TrayIconIcon;
use image;
use rfd::MessageDialog;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Proxy server port
    #[arg(short, long, default_value_t = 8083)]
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

    /// Run in GUI mode with system tray icon
    #[arg(short, long)]
    gui: bool,
}

// Global state for the proxy settings
struct ProxyState {
    ip: String,
    port: u16,
}

// 修改为使用普通全局变量而不是OnceCell，并在函数内部存储到Box里
static mut TRAY_ICON: Option<Box<dyn std::any::Any>> = None;

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
    
    // Create shared proxy state
    let proxy_state = Arc::new(Mutex::new(ProxyState {
        ip: match &args.ip {
            Some(ip) => ip.clone(),
            None => {
                let local_ip = local_ip().context("Unable to get local IP address")?;
                local_ip.to_string()
            }
        },
        port: args.port,
    }));
    
    // Choose between CLI and GUI mode
    if args.gui {
        run_gui_mode(proxy_state)
    } else {
        run_cli_mode(args, current_proxy_setting)
    }
}

fn run_cli_mode(args: Args, current_proxy_setting: String) -> Result<()> {
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
        println!("3. {}", "View current proxy settings".blue());
        println!("4. {}", "Exit".yellow());
        
        print!("\nEnter your choice (1-4): ");
        io::stdout().flush()?;
        
        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        
        match choice.trim() {
            "1" => set_proxy(&args)?,
            "2" => clear_proxy()?,
            "3" => view_proxy()?,
            "4" => {
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

fn run_gui_mode(proxy_state: Arc<Mutex<ProxyState>>) -> Result<()> {
    println!("{}", "Starting in GUI mode with system tray icon...".green());
    
    // Create event loop
    let event_loop = EventLoop::new();
    
    // Create tray menu
    let tray_menu = Menu::new();
    
    // Create menu items
    let set_proxy_item = MenuItem::new("Set Proxy", true, None);
    let clear_proxy_item = MenuItem::new("Clear Proxy", true, None);
    let view_proxy_item = MenuItem::new("View Proxy Settings", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    
    // Add items to menu
    let _ = tray_menu.append(&set_proxy_item);
    let _ = tray_menu.append(&clear_proxy_item);
    let _ = tray_menu.append(&view_proxy_item);
    let _ = tray_menu.append(&PredefinedMenuItem::separator());
    let _ = tray_menu.append(&quit_item);
    
    // Create tray icon
    // 从resources目录加载PNG图标
    let icon = include_bytes!("../resources/icon.png");
    let icon = image::load_from_memory(icon)
        .context("Failed to load icon")?;
    let icon = icon.to_rgba8();
    let (icon_width, icon_height) = icon.dimensions();
    
    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_icon(TrayIconIcon::from_rgba(icon.into_raw(), icon_width, icon_height).unwrap())
        .with_tooltip("Android Proxy Manager")
        .build()
        .context("Failed to create tray icon")?;
    
    // 安全地存储tray_icon到全局变量
    unsafe {
        TRAY_ICON = Some(Box::new(tray_icon));
    }
    
    // Create a channel for menu events
    let (tx, rx) = std::sync::mpsc::channel();
    
    // Store menu item IDs for later use
    let set_proxy_id = set_proxy_item.id();
    let clear_proxy_id = clear_proxy_item.id();
    let view_proxy_id = view_proxy_item.id();
    let quit_id = quit_item.id();
    
    // Handle menu events
    let menu_channel = MenuEvent::receiver();
    thread::spawn(move || {
        while let Ok(event) = menu_channel.recv() {
            if event.id == set_proxy_id {
                let _ = tx.send("set");
            } else if event.id == clear_proxy_id {
                let _ = tx.send("clear");
            } else if event.id == view_proxy_id {
                let _ = tx.send("view");
            } else if event.id == quit_id {
                let _ = tx.send("quit");
            }
        }
    });
    
    // Clone proxy_state for the event loop
    let proxy_state_clone = Arc::clone(&proxy_state);
    
    // Run the event loop
    event_loop.run(move |_, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        
        // Check for menu events
        if let Ok(cmd) = rx.try_recv() {
            match cmd {
                "set" => {
                    let state = proxy_state_clone.lock().unwrap();
                    let ip = state.ip.clone();
                    let port = state.port;
                    drop(state); // 立即释放锁
                    
                    thread::spawn(move || {
                        // 创建一个具有所需值的Args对象
                        let args = Args {
                            port,
                            ip: Some(ip),
                            set: false,
                            clear: false,
                            gui: false,
                        };
                        
                        if let Err(e) = set_proxy(&args) {
                            show_error_dialog(&format!("Failed to set proxy: {}", e));
                        } else {
                            show_info_dialog("Successfully set proxy!");
                        }
                    });
                }
                "clear" => {
                    thread::spawn(move || {
                        if let Err(e) = clear_proxy() {
                            show_error_dialog(&format!("Failed to clear proxy: {}", e));
                        } else {
                            show_info_dialog("Successfully cleared proxy!");
                        }
                    });
                }
                "view" => {
                    thread::spawn(move || {
                        match get_proxy_info() {
                            Ok(info) => show_info_dialog(&info),
                            Err(e) => show_error_dialog(&format!("Failed to get proxy info: {}", e)),
                        }
                    });
                }
                "quit" => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            }
        }
    });
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
    thread::sleep(Duration::from_millis(500));

    // Set new proxy
    println!("Setting new proxy to {}:{}", ip.green(), port.to_string().green());
    let set_proxy = Command::new("adb")
        .args(["shell", "settings", "put", "global", "http_proxy", &format!("{ip}:{port}")])
        .output()
        .context("Failed to set HTTP proxy")?;

    if !set_proxy.status.success() {
        let error = String::from_utf8_lossy(&set_proxy.stderr);
        anyhow::bail!("Failed to set proxy: {}", error);
    }

    // Wait for a short period to ensure the setting operation is completed
    thread::sleep(Duration::from_millis(500));

    // Verify proxy settings
    println!("Verifying proxy settings...");
    let verify_proxy = Command::new("adb")
        .args(["shell", "settings", "get", "global", "http_proxy"])
        .output()
        .context("Failed to verify proxy settings")?;
    
    if verify_proxy.status.success() {
        let proxy_setting = String::from_utf8_lossy(&verify_proxy.stdout).trim().to_string();
        println!("Current proxy settings: {}", proxy_setting.green());
        
        if proxy_setting == format!("{ip}:{port}") {
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

    // Wait for a short period to ensure the clearing operation is completed
    thread::sleep(Duration::from_millis(500));

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

fn view_proxy() -> Result<()> {
    println!("{}", "Checking current Android device proxy settings...".blue());
    
    let proxy_settings = Command::new("adb")
        .args(["shell", "settings", "get", "global", "http_proxy"])
        .output()
        .context("Failed to get HTTP proxy settings")?;

    if !proxy_settings.status.success() {
        let error = String::from_utf8_lossy(&proxy_settings.stderr);
        anyhow::bail!("Failed to get proxy settings: {}", error);
    }

    let proxy_setting = String::from_utf8_lossy(&proxy_settings.stdout).trim().to_string();
    
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
    io::stdin().read_line(&mut input)?;
    
    Ok(())
}

// Function to get proxy info as a string for GUI mode
fn get_proxy_info() -> Result<String> {
    let proxy_settings = Command::new("adb")
        .args(["shell", "settings", "get", "global", "http_proxy"])
        .output()
        .context("Failed to get HTTP proxy settings")?;

    if !proxy_settings.status.success() {
        let error = String::from_utf8_lossy(&proxy_settings.stderr);
        anyhow::bail!("Failed to get proxy settings: {}", error);
    }

    let proxy_setting = String::from_utf8_lossy(&proxy_settings.stdout).trim().to_string();
    
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

// Helper functions for GUI dialogs
fn show_info_dialog(message: &str) {
    MessageDialog::new()
        .set_title("Android Proxy Manager")
        .set_description(message)
        .set_level(rfd::MessageLevel::Info)
        .show();
}

fn show_error_dialog(message: &str) {
    MessageDialog::new()
        .set_title("Android Proxy Manager - Error")
        .set_description(message)
        .set_level(rfd::MessageLevel::Error)
        .show();
}
