//! Proxy management operations

use std::net::IpAddr;
use std::thread;
use std::time::Duration;

use dialoguer::console::style;
use local_ip_address::local_ip;

use crate::adb::commands::{
    clear_proxy as adb_clear_proxy, get_proxy as adb_get_proxy, set_proxy as adb_set_proxy,
};
use crate::error::{AppError, AppResult};

const VERIFY_ATTEMPTS: usize = 3;
const VERIFY_RETRY_DELAY: Duration = Duration::from_millis(100);

/// Set proxy on one Android device and verify the resulting value.
pub fn set_proxy(proxy: &str, device: &str) -> AppResult<()> {
    let previous = get_current_proxy_setting(device)?;

    println!("Setting Android proxy to {}", style(proxy).green());
    let operation = adb_set_proxy(proxy, device).and_then(|_| verify_proxy_value(proxy, device));
    if let Err(operation_error) = operation {
        if let Err(rollback_error) = restore_proxy(&previous, device) {
            return Err(AppError::ProxyRollbackFailed {
                operation_error: operation_error.to_string(),
                rollback_error: rollback_error.to_string(),
            });
        }
        return Err(operation_error);
    }

    println!(
        "{}",
        style("✅ Android proxy set successfully.").green().bold()
    );
    Ok(())
}

/// Clear proxy settings on one Android device and verify the resulting value.
pub fn clear_proxy(device: &str) -> AppResult<()> {
    println!("{}", style("Clearing Android proxy settings...").yellow());
    adb_clear_proxy(device)?;

    let actual = read_proxy_until(device, is_proxy_unset)?;
    if !is_proxy_unset(&actual) {
        return Err(AppError::ProxyVerificationFailed {
            expected: "not set".to_string(),
            actual,
        });
    }

    println!(
        "{}",
        style("✅ Android proxy cleared successfully.")
            .green()
            .bold()
    );
    Ok(())
}

pub fn view_proxy(device: &str) -> AppResult<()> {
    let proxy = get_current_proxy_setting(device)?;
    print_proxy(&proxy, "Current Android Proxy Settings:");
    Ok(())
}

pub fn get_current_proxy_setting(device: &str) -> AppResult<String> {
    adb_get_proxy(device)
}

pub fn resolve_proxy_address(custom_ip: Option<IpAddr>, port: u16) -> AppResult<String> {
    let ip = match custom_ip {
        Some(ip) => ip,
        None => local_ip().map_err(|error| AppError::LocalIpError {
            reason: error.to_string(),
        })?,
    };

    Ok(match ip {
        IpAddr::V4(ip) => format!("{ip}:{port}"),
        IpAddr::V6(ip) => format!("[{ip}]:{port}"),
    })
}

fn verify_proxy_value(expected: &str, device: &str) -> AppResult<()> {
    let actual = read_proxy_until(device, |actual| actual == expected)?;
    if actual != expected {
        return Err(AppError::ProxyVerificationFailed {
            expected: expected.to_string(),
            actual,
        });
    }

    println!("Verified proxy setting: {}", style(actual).green());
    Ok(())
}

fn restore_proxy(previous: &str, device: &str) -> AppResult<()> {
    if is_proxy_unset(previous) {
        adb_clear_proxy(device)?;
        let actual = read_proxy_until(device, is_proxy_unset)?;
        if !is_proxy_unset(&actual) {
            return Err(AppError::ProxyVerificationFailed {
                expected: "not set".to_string(),
                actual,
            });
        }
    } else {
        adb_set_proxy(previous, device)?;
        verify_proxy_value(previous, device)?;
    }
    Ok(())
}

fn read_proxy_until(device: &str, matches_expected: impl Fn(&str) -> bool) -> AppResult<String> {
    let mut last_value = String::new();
    for attempt in 0..VERIFY_ATTEMPTS {
        last_value = get_current_proxy_setting(device)?;
        if matches_expected(&last_value) {
            break;
        }
        if attempt + 1 < VERIFY_ATTEMPTS {
            thread::sleep(VERIFY_RETRY_DELAY);
        }
    }
    Ok(last_value)
}

fn is_proxy_unset(proxy: &str) -> bool {
    matches!(proxy.trim(), "" | ":0" | "null")
}

fn split_proxy(proxy: &str) -> Option<(&str, &str)> {
    let (host, port) = proxy.rsplit_once(':')?;
    let host = host
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(host);
    Some((host, port))
}

fn print_proxy(proxy: &str, heading: &str) {
    println!("\n{}", style(heading).blue().bold());
    if is_proxy_unset(proxy) {
        println!("Global HTTP Proxy: {}", style("Not set").red());
        return;
    }

    println!("Global HTTP Proxy: {}", style(proxy).green());
    if let Some((ip, port)) = split_proxy(proxy) {
        println!("IP Address: {}", style(ip).green());
        println!("Port: {}", style(port).green());
    } else {
        println!("{}", style("Unable to parse the proxy address.").yellow());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_android_unset_values() {
        for value in ["", ":0", "null", " null\n"] {
            assert!(is_proxy_unset(value));
        }
    }

    #[test]
    fn splits_ipv4_and_ipv6_proxies() {
        assert_eq!(
            split_proxy("192.168.1.2:8017"),
            Some(("192.168.1.2", "8017"))
        );
        assert_eq!(
            split_proxy("[2001:db8::1]:8017"),
            Some(("2001:db8::1", "8017"))
        );
    }

    #[test]
    fn formats_ipv4_and_ipv6_proxy_addresses() {
        assert_eq!(
            resolve_proxy_address(Some("192.168.1.2".parse().unwrap()), 8017).unwrap(),
            "192.168.1.2:8017"
        );
        assert_eq!(
            resolve_proxy_address(Some("2001:db8::1".parse().unwrap()), 8017).unwrap(),
            "[2001:db8::1]:8017"
        );
    }
}
