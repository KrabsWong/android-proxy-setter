//! Custom error types for the application

use thiserror::Error;

/// Application result type
pub type AppResult<T> = Result<T, AppError>;

/// Main application error type
#[derive(Error, Debug)]
pub enum AppError {
    #[error("ADB command failed ({command}): {source}")]
    AdbCommandFailed {
        command: String,
        source: std::io::Error,
    },

    #[error("ADB command timed out after {seconds}s: {command}")]
    AdbCommandTimedOut { command: String, seconds: u64 },

    #[error("ADB not found or not in PATH")]
    AdbNotFound,

    #[error("No ready Android devices found ({details}); check the USB connection, debugging authorization, and `adb devices`")]
    NoDevicesConnected { details: String },

    #[error(
        "Multiple Android devices are connected ({devices}); use --device <SERIAL> to choose one"
    )]
    MultipleDevicesConnected { devices: String },

    #[error("Android device '{serial}' was not found or is not ready")]
    DeviceNotAvailable { serial: String },

    #[error("Failed to get local IP address: {reason}")]
    LocalIpError { reason: String },

    #[error("Proxy verification failed: expected '{expected}', got '{actual}'")]
    ProxyVerificationFailed { expected: String, actual: String },

    #[error("Proxy operation failed ({operation_error}); rollback also failed ({rollback_error})")]
    ProxyRollbackFailed {
        operation_error: String,
        rollback_error: String,
    },

    #[error("I/O error: {source}")]
    IoError { source: std::io::Error },
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
}
