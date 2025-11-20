//! Interactive command-line interface

use std::io::{self, Write};
use colored::*;
use crate::error::AppResult;
use crate::config::args::Args;
use crate::proxy::manager::{set_proxy, clear_proxy, view_proxy};
use crate::proxy::settings::ProxySettings;
use crate::adb::device::restart_adb_server;

/// Run the interactive CLI mode
pub fn run_cli_mode(args: Args, current_proxy_setting: String) -> AppResult<()> {
    // Interactive mode or direct action based on flags
    if args.set {
        let settings = ProxySettings::new(args.port, args.ip)?;
        set_proxy(&settings)?;
    } else if args.clear {
        clear_proxy()?;
    } else if args.restart_adb {
        restart_adb_server()?;
    } else if args.help_commands {
        show_available_commands()?;
    } else if args.view {
        view_proxy()?;
    } else {
        run_interactive_mode(current_proxy_setting, args)?;
    }

    Ok(())
}

/// Run the interactive menu
fn run_interactive_mode(current_proxy_setting: String, args: Args) -> AppResult<()> {
    println!("\n{}", "=== Android Proxy Manager ===".green().bold());
    println!(
        "Current proxy setting: {}",
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
    println!("4. {}", "Restart ADB".yellow());
    println!("5. {}", "Exit".purple());

    print!("\nEnter your choice (1-5): ");
    io::stdout().flush()?;

    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;

    match choice.trim() {
        "1" => {
            let settings = ProxySettings::new(args.port, args.ip)?;
            set_proxy(&settings)?;
        }
        "2" => clear_proxy()?,
        "3" => view_proxy()?,
        "4" => restart_adb_server()?,
        "5" => {
            println!("{}", "Exiting...".yellow());
            return Ok(());
        }
        _ => {
            println!("{}", "Invalid choice. Exiting...".red());
            return Ok(());
        }
    }

    Ok(())
}

/// Show available commands and aliases
pub fn show_available_commands() -> AppResult<()> {
    use colored::*;

    println!("\n{}", "=== Android Proxy Setter - Available Commands ===".green().bold());

    println!("\n{}", "Shell aliases (after installation):".yellow());
    println!("  aps                               - Interactive mode");
    println!("  aps-set                           - Set proxy directly");
    println!("  aps-clear                         - Clear proxy directly");
    println!("  aps-view                          - View current proxy settings");
    println!("  aps-restart                       - Restart ADB server");

    println!("\n{}", "Options:".blue());
    println!("  --port <PORT>                     - Specify proxy port (default: 8083)");
    println!("  --ip <IP_ADDRESS>                 - Specify IP address (auto-detected if not specified)");

    println!("\n{}", "Installation:".blue());
    println!("  make install                      - Build and install");
    println!("  make uninstall                    - Remove installation");

    Ok(())
}