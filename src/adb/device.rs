//! Android device discovery and selection.

use crate::adb::commands::list_devices;
use crate::error::{AppError, AppResult};

/// Get list of connected Android devices
fn get_connected_devices() -> AppResult<Vec<String>> {
    let output = list_devices()?;

    let entries = parse_device_entries(&output);
    let devices: Vec<String> = entries
        .iter()
        .filter(|entry| entry.state == "device")
        .map(|entry| entry.serial.clone())
        .collect();

    if devices.is_empty() {
        let details = if entries.is_empty() {
            "adb returned no devices".to_string()
        } else {
            entries
                .iter()
                .map(|entry| format!("{}: {}", entry.serial, entry.state))
                .collect::<Vec<_>>()
                .join(", ")
        };
        return Err(AppError::NoDevicesConnected { details });
    }

    Ok(devices)
}

#[derive(Debug, PartialEq)]
struct DeviceEntry {
    serial: String,
    state: String,
}

fn parse_device_entries(output: &str) -> Vec<DeviceEntry> {
    output
        .lines()
        .skip_while(|line| !line.starts_with("List of devices attached"))
        .skip(1)
        .filter_map(|line| {
            let mut columns = line.split_whitespace();
            let serial = columns.next()?;
            let state = columns.next()?;
            Some(DeviceEntry {
                serial: serial.to_string(),
                state: state.to_string(),
            })
        })
        .collect()
}

/// Resolve a usable device, requiring an explicit serial when more than one is ready.
pub fn resolve_device(requested_serial: Option<&str>) -> AppResult<String> {
    let devices = get_connected_devices()?;

    if let Some(serial) = requested_serial {
        return devices
            .into_iter()
            .find(|device| device == serial)
            .ok_or_else(|| AppError::DeviceNotAvailable {
                serial: serial.to_string(),
            });
    }

    if devices.len() > 1 {
        return Err(AppError::MultipleDevicesConnected {
            devices: devices.join(", "),
        });
    }

    devices
        .into_iter()
        .next()
        .ok_or_else(|| AppError::NoDevicesConnected {
            details: "adb returned no devices".to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_device_states() {
        let output = "List of devices attached\nready-1\tdevice product:foo\noffline-1\toffline\nunauthorized-1\tunauthorized\nready-2 device\n";
        assert_eq!(
            parse_device_entries(output),
            vec![
                DeviceEntry {
                    serial: "ready-1".to_string(),
                    state: "device".to_string(),
                },
                DeviceEntry {
                    serial: "offline-1".to_string(),
                    state: "offline".to_string(),
                },
                DeviceEntry {
                    serial: "unauthorized-1".to_string(),
                    state: "unauthorized".to_string(),
                },
                DeviceEntry {
                    serial: "ready-2".to_string(),
                    state: "device".to_string(),
                },
            ]
        );
    }
}
