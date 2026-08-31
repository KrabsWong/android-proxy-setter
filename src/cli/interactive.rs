//! Interactive command-line interface

use dialoguer::{console::style, theme::ColorfulTheme, Select};

use crate::adb::commands::restart_adb_server;
use crate::config::args::{Args, DEFAULT_PROXY_PORT};
use crate::error::AppResult;
use crate::proxy::manager::{
    clear_proxy, get_current_proxy_setting, resolve_proxy_address, set_proxy, view_proxy,
};

pub fn run_interactive_mode(args: Args, device: &str) -> AppResult<()> {
    let current_proxy = get_current_proxy_setting(device)?;
    println!("\n{}", style("Android Proxy").green().bold());
    if args.device.is_some() {
        println!("Target  {}", style(device).cyan());
    }
    println!(
        "Proxy   {}",
        if matches!(current_proxy.trim(), "" | ":0" | "null") {
            style("Not set").red()
        } else {
            style(current_proxy.as_str()).green()
        }
    );

    let labels = [
        format!("Set global proxy (port {})", args.port),
        "Clear global proxy".to_string(),
        "View current proxy settings".to_string(),
        "Restart ADB server".to_string(),
        "Exit".to_string(),
    ];
    println!(
        "\n{}",
        style("↑↓ select  •  Enter confirm  •  Esc cancel").dim()
    );
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Action")
        .items(&labels)
        .default(0)
        .clear(true)
        .interact_opt()
        .map_err(|error| std::io::Error::other(error.to_string()))?;

    match selection {
        Some(0) => {
            let proxy = resolve_proxy_address(args.ip, args.port)?;
            set_proxy(&proxy, device)
        }
        Some(1) => clear_proxy(device),
        Some(2) => view_proxy(device),
        Some(3) => {
            restart_adb_server()?;
            println!("ADB server restarted successfully.");
            Ok(())
        }
        Some(4) | None => Ok(()),
        Some(_) => unreachable!("dialoguer returned an invalid menu index"),
    }
}

pub fn show_available_commands() {
    println!(
        "\n{}",
        style("=== Android Proxy Setter - Available Commands ===")
            .green()
            .bold()
    );
    println!(
        "\n{}",
        style("Default shell aliases (after installation):").yellow()
    );
    println!("  aps                               - Interactive mode");
    println!("  aps-set                           - Set proxy directly");
    println!("  aps-clear                         - Clear proxy directly");
    println!("  aps-view                          - View current proxy settings");
    println!("  aps-restart                       - Restart ADB server");
    println!("  aps-help                          - Show this command list");

    println!("\n{}", style("Options:").blue());
    println!(
        "  --port <PORT>                     - Specify proxy port (default: {DEFAULT_PROXY_PORT})"
    );
    println!("  --ip <IP_ADDRESS>                 - Specify IP address (auto-detected if omitted)");
    println!("  --device <SERIAL>                 - Select a device when multiple are connected");

    println!("\n{}", style("Installation:").blue());
    println!("  make install                      - Build and install");
    println!("  make uninstall                    - Remove installation");
}
