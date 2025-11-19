//! ADB command execution utilities

use std::process::{Command, Output};
use crate::error::{AppError, AppResult};

/// ADB command types
#[derive(Debug, Clone)]
pub enum AdbCommand {
    GetProxy,
    SetProxy(String),
    ClearProxy,
    GetDevices,
}

impl AdbCommand {
    /// Convert the command to its string representation
    pub fn to_args(&self) -> Vec<String> {
        match self {
            AdbCommand::GetProxy => vec![
                "shell".to_string(),
                "settings".to_string(),
                "get".to_string(),
                "global".to_string(),
                "http_proxy".to_string(),
            ],
            AdbCommand::SetProxy(proxy) => vec![
                "shell".to_string(),
                "settings".to_string(),
                "put".to_string(),
                "global".to_string(),
                "http_proxy".to_string(),
                proxy.clone(),
            ],
            AdbCommand::ClearProxy => vec![
                "shell".to_string(),
                "settings".to_string(),
                "put".to_string(),
                "global".to_string(),
                "http_proxy".to_string(),
                ":0".to_string(),
            ],
            AdbCommand::GetDevices => vec!["devices".to_string()],
        }
    }

    /// Get the command description for error messages
    pub fn description(&self) -> String {
        match self {
            AdbCommand::GetProxy => "get proxy settings".to_string(),
            AdbCommand::SetProxy(proxy) => format!("set proxy to {}", proxy),
            AdbCommand::ClearProxy => "clear proxy settings".to_string(),
            AdbCommand::GetDevices => "get connected devices".to_string(),
        }
    }
}

/// Execute an ADB command and return the output
pub fn execute_adb_command(command: AdbCommand) -> AppResult<Output> {
    let args = command.to_args();
    let description = command.description();

    let output = Command::new("adb")
        .args(&args)
        .output()
        .map_err(|e| AppError::adb_command_failed(&description, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::adb_command_failed(&description,
            std::io::Error::new(std::io::ErrorKind::Other, stderr.to_string())));
    }

    Ok(output)
}

/// Execute an ADB command and return the stdout as a string
pub fn execute_adb_command_string(command: AdbCommand) -> AppResult<String> {
    let output = execute_adb_command(command)?;
    let result = String::from_utf8(output.stdout)
        .map_err(|e| AppError::Utf8Error { source: e })?
        .trim()
        .to_string();

    Ok(result)
}

