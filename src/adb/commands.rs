//! ADB command execution utilities

use std::fs::{remove_file, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

use crate::error::{AppError, AppResult};

const ADB_COMMAND_TIMEOUT: Duration = Duration::from_secs(15);
static CAPTURE_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn list_devices() -> AppResult<String> {
    execute_adb_string(&["devices"], None, "get connected devices")
}

pub fn get_proxy(device: &str) -> AppResult<String> {
    execute_adb_string(
        &["shell", "settings", "get", "global", "http_proxy"],
        Some(device),
        "get proxy settings",
    )
}

pub fn set_proxy(proxy: &str, device: &str) -> AppResult<()> {
    execute_adb(
        &["shell", "settings", "put", "global", "http_proxy", proxy],
        Some(device),
        &format!("set proxy to {proxy}"),
    )?;
    Ok(())
}

pub fn clear_proxy(device: &str) -> AppResult<()> {
    execute_adb(
        &["shell", "settings", "put", "global", "http_proxy", ":0"],
        Some(device),
        "clear proxy settings",
    )?;
    Ok(())
}

pub fn restart_adb_server() -> AppResult<()> {
    execute_adb(&["kill-server"], None, "stop ADB server")?;
    execute_adb(&["start-server"], None, "start ADB server")?;
    Ok(())
}

fn execute_adb(args: &[&str], device: Option<&str>, description: &str) -> AppResult<Output> {
    let mut adb = Command::new("adb");
    if let Some(serial) = device {
        adb.args(["-s", serial]);
    }

    adb.args(args);
    let output = execute_process(adb, description).map_err(|error| match error {
        AppError::AdbCommandFailed { source, .. }
            if source.kind() == std::io::ErrorKind::NotFound =>
        {
            AppError::AdbNotFound
        }
        error => error,
    })?;

    if !output.status.success() {
        return Err(AppError::adb_command_failed(
            description,
            std::io::Error::other(String::from_utf8_lossy(&output.stderr).trim()),
        ));
    }

    Ok(output)
}

fn execute_adb_string(args: &[&str], device: Option<&str>, description: &str) -> AppResult<String> {
    let output = execute_adb(args, device, description)?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn execute_process(mut command: Command, description: &str) -> AppResult<Output> {
    let mut stdout = CaptureFile::new("stdout")
        .map_err(|error| AppError::adb_command_failed(description, error))?;
    let mut stderr = CaptureFile::new("stderr")
        .map_err(|error| AppError::adb_command_failed(description, error))?;
    let mut child = command
        .stdout(Stdio::from(stdout.try_clone().map_err(|error| {
            AppError::adb_command_failed(description, error)
        })?))
        .stderr(Stdio::from(stderr.try_clone().map_err(|error| {
            AppError::adb_command_failed(description, error)
        })?))
        .spawn()
        .map_err(|error| AppError::adb_command_failed(description, error))?;
    let started_at = Instant::now();

    loop {
        let status = match child.try_wait() {
            Ok(status) => status,
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(AppError::adb_command_failed(description, error));
            }
        };

        if let Some(status) = status {
            return Ok(Output {
                status,
                stdout: stdout
                    .read()
                    .map_err(|error| AppError::adb_command_failed(description, error))?,
                stderr: stderr
                    .read()
                    .map_err(|error| AppError::adb_command_failed(description, error))?,
            });
        }

        if started_at.elapsed() >= ADB_COMMAND_TIMEOUT {
            let _ = child.kill();
            let _ = child.wait();
            return Err(AppError::AdbCommandTimedOut {
                command: description.to_string(),
                seconds: ADB_COMMAND_TIMEOUT.as_secs(),
            });
        }

        thread::sleep(Duration::from_millis(50));
    }
}

struct CaptureFile {
    file: File,
    path: PathBuf,
}

impl CaptureFile {
    fn new(label: &str) -> std::io::Result<Self> {
        for _ in 0..100 {
            let sequence = CAPTURE_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "android-proxy-setter-{}-{sequence}-{label}",
                std::process::id()
            ));
            let mut options = OpenOptions::new();
            options.read(true).write(true).create_new(true);
            #[cfg(unix)]
            options.mode(0o600);
            match options.open(&path) {
                Ok(file) => return Ok(Self { file, path }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "could not create a unique ADB output capture file",
        ))
    }

    fn try_clone(&self) -> std::io::Result<File> {
        self.file.try_clone()
    }

    fn read(&mut self) -> std::io::Result<Vec<u8>> {
        self.file.seek(SeekFrom::Start(0))?;
        let mut output = Vec::new();
        self.file.read_to_end(&mut output)?;
        Ok(output)
    }
}

impl Drop for CaptureFile {
    fn drop(&mut self) {
        let _ = remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drains_large_subprocess_output_without_blocking() {
        let mut command = Command::new("awk");
        command.args(["BEGIN { for (i=0; i<20000; i++) print \"abcdefghij\" }"]);
        let output = execute_process(command, "generate test output").unwrap();

        assert!(output.status.success());
        assert!(output.stdout.len() > 200_000);
    }
}
