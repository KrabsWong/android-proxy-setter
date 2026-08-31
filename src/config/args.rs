//! Command-line argument parsing

use std::net::IpAddr;

use clap::{ArgGroup, Parser};

pub const DEFAULT_PROXY_PORT: u16 = 8017;

/// Parsed command-line arguments.
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Manage Android HTTP proxy settings through ADB",
    long_about = None,
    group(
        ArgGroup::new("action")
            .args(["set", "clear", "restart_adb", "view", "help_commands"])
            .multiple(false)
    )
)]
pub struct Args {
    /// Proxy server port
    #[arg(
        short,
        long,
        default_value_t = DEFAULT_PROXY_PORT,
        value_parser = clap::value_parser!(u16).range(1..)
    )]
    pub port: u16,

    /// Manually specify IP address, automatically get if not specified
    #[arg(short, long)]
    pub ip: Option<IpAddr>,

    /// Target device serial from `adb devices` (required when multiple devices are connected)
    #[arg(short, long)]
    pub device: Option<String>,

    /// Skip interactive mode and directly set proxy
    #[arg(short, long)]
    pub set: bool,

    /// Skip interactive mode and directly clear proxy
    #[arg(short, long)]
    pub clear: bool,

    /// Skip interactive mode and directly restart ADB server
    #[arg(long)]
    pub restart_adb: bool,

    /// Show available commands and aliases
    #[arg(long)]
    pub help_commands: bool,

    /// Skip interactive mode and directly view proxy settings
    #[arg(long)]
    pub view: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_expected_default_port() {
        let args = Args::try_parse_from(["android_proxy_setter"]).unwrap();
        assert_eq!(args.port, 8017);
    }

    #[test]
    fn rejects_conflicting_actions() {
        let result = Args::try_parse_from(["android_proxy_setter", "--set", "--clear"]);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_invalid_ip_address() {
        let result = Args::try_parse_from(["android_proxy_setter", "--ip", "not-an-ip"]);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_port_zero() {
        let result = Args::try_parse_from(["android_proxy_setter", "--port", "0"]);
        assert!(result.is_err());
    }
}
