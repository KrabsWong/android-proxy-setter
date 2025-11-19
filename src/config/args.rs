//! Command-line argument parsing

use clap::Parser;

/// Command-line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Proxy server port
    #[arg(short, long, default_value_t = 8083)]
    pub port: u16,

    /// Manually specify IP address, automatically get if not specified
    #[arg(short, long)]
    pub ip: Option<String>,

    /// Skip interactive mode and directly set proxy
    #[arg(short, long)]
    pub set: bool,

    /// Skip interactive mode and directly clear proxy
    #[arg(short, long)]
    pub clear: bool,

}

/// Parse command-line arguments
pub fn parse_args() -> Args {
    Args::parse()
}