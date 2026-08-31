//! Android Proxy Setter - manage Android HTTP proxy settings through ADB.

mod adb;
mod cli;
mod config;
mod error;
mod proxy;

use std::process::ExitCode;

use clap::Parser;

use crate::adb::commands::restart_adb_server;
use crate::adb::device::resolve_device;
use crate::cli::{run_interactive_mode, show_available_commands};
use crate::config::args::Args;
use crate::error::AppResult;
use crate::proxy::manager::{clear_proxy, resolve_proxy_address, set_proxy, view_proxy};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> AppResult<()> {
    let args = Args::parse();

    if args.help_commands {
        show_available_commands();
        return Ok(());
    }

    if args.restart_adb {
        restart_adb_server()?;
        println!("ADB server restarted successfully.");
        return Ok(());
    }

    let device = resolve_device(args.device.as_deref())?;

    if args.set {
        let proxy = resolve_proxy_address(args.ip, args.port)?;
        return set_proxy(&proxy, &device);
    }
    if args.clear {
        return clear_proxy(&device);
    }
    if args.view {
        return view_proxy(&device);
    }

    run_interactive_mode(args, &device)
}
