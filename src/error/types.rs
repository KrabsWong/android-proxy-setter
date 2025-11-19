//! Custom error types for the application

use thiserror::Error;

/// Application result type
pub type AppResult<T> = Result<T, AppError>;

/// Main application error type
#[derive(Error, Debug)]
pub enum AppError {
    #[error("ADB command failed: {command}")]
    AdbCommandFailed {
        command: String,
        source: std::io::Error,
    },

    #[error("ADB not found or not in PATH")]
    AdbNotFound,

    #[error("No connected Android devices found")]
    NoDevicesConnected,

    #[error("Failed to get local IP address: {reason}")]
    LocalIpError {
        reason: String,
    },

    #[error("Failed to clear proxy: {reason}")]
    ProxyClearFailed {
        reason: String,
    },

    #[error("Failed to get proxy settings: {reason}")]
    ProxyGetFailed {
        reason: String,
    },

    #[error("I/O error: {source}")]
    IoError {
        source: std::io::Error,
    },

    #[error("UTF-8 conversion error: {source}")]
    Utf8Error {
        #[from]
        source: std::string::FromUtf8Error,
    },
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::IoError { source: error }
    }
}

impl AppError {
    /// Create a new ADB command failed error
    pub fn adb_command_failed(command: &str, source: std::io::Error) -> Self {
        AppError::AdbCommandFailed {
            command: command.to_string(),
            source,
        }
    }

    /// Create a new proxy clear failed error
    pub fn proxy_clear_failed(reason: impl Into<String>) -> Self {
        AppError::ProxyClearFailed {
            reason: reason.into(),
        }
    }

    /// Create a new proxy get failed error
    pub fn proxy_get_failed(reason: impl Into<String>) -> Self {
        AppError::ProxyGetFailed {
            reason: reason.into(),
        }
    }
}