//! Proxy settings and state management

use local_ip_address::local_ip;
use crate::error::{AppError, AppResult};

/// Proxy settings configuration
#[derive(Debug, Clone)]
pub struct ProxySettings {
    pub ip: String,
    pub port: u16,
}

impl ProxySettings {
    /// Create new proxy settings with automatic IP detection
    pub fn new(port: u16, custom_ip: Option<String>) -> AppResult<Self> {
        let ip = match custom_ip {
            Some(ip) => ip,
            None => {
                let local_ip = local_ip()
                    .map_err(|e| AppError::LocalIpError { reason: e.to_string() })?;
                local_ip.to_string()
            }
        };

        Ok(Self { ip, port })
    }

    /// Get the proxy string in format "ip:port"
    pub fn to_proxy_string(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }

}

