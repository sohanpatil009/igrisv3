// src/file_share/connection_types.rs - Shared data structures for unified connection system

use std::time::Instant;
use serde::{Deserialize, Serialize};

use super::config::OperatingSystem;
use super::relay::DeviceRegistration;
use super::discovery::DiscoveredDevice;

/// A common structure for passing device information between components
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceInfo {
    pub device_id: String,
    pub hostname: String,
    pub label: String,
    pub os: OperatingSystem,
    pub ip_address: String,
    pub bridge_port: u16,
}

impl From<DeviceRegistration> for DeviceInfo {
    fn from(reg: DeviceRegistration) -> Self {
        DeviceInfo {
            device_id: reg.device_id,
            hostname: reg.hostname,
            label: reg.label,
            os: reg.os,
            ip_address: reg.ip_address,
            bridge_port: reg.bridge_port,
        }
    }
}

impl From<DiscoveredDevice> for DeviceInfo {
    fn from(dev: DiscoveredDevice) -> Self {
        DeviceInfo {
            device_id: dev.id,
            hostname: dev.hostname,
            label: dev.label,
            os: dev.os,
            ip_address: dev.ip_address.to_string(),
            bridge_port: dev.bridge_port,
        }
    }
}

/// Connection code with expiry information
#[derive(Debug, Clone)]
pub struct ConnectionCode {
    pub code: String,
    pub expires_at: Instant,
    pub remaining_seconds: u64,
}

impl ConnectionCode {
    pub fn new(code: String, expires_at: Instant) -> Self {
        let remaining_seconds = expires_at
            .saturating_duration_since(Instant::now())
            .as_secs();
        
        ConnectionCode {
            code,
            expires_at,
            remaining_seconds,
        }
    }
    
    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }
    
    pub fn update_remaining(&mut self) {
        self.remaining_seconds = self.expires_at
            .saturating_duration_since(Instant::now())
            .as_secs();
    }
}

/// Result of a connection attempt
#[derive(Debug, Clone)]
pub struct ConnectionResult {
    pub device: DiscoveredDevice,
    pub trust_established: bool,
    pub connection_type: ConnectionType,
}

/// Type of connection established
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionType {
    NewConnection,
    AlreadyTrusted,
    Reconnection,
}

/// Errors that can occur during connection
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionError {
    CodeNotFound,
    CodeExpired,
    NetworkError(String),
    AlreadyTrusted,
    RateLimited { remaining_secs: u64 },
    TrustFailed(String),
    InvalidCode,
}

impl ConnectionError {
    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            Self::CodeNotFound => "Code not found - please check and try again".to_string(),
            Self::CodeExpired => "Code expired - ask for a new code".to_string(),
            Self::NetworkError(_) => "Connection failed - check network and try again".to_string(),
            Self::AlreadyTrusted => "Device already trusted".to_string(),
            Self::RateLimited { remaining_secs } => {
                let minutes = remaining_secs / 60;
                if minutes > 0 {
                    format!("Too many failed attempts - try again in {} minutes", minutes)
                } else {
                    format!("Too many failed attempts - try again in {} seconds", remaining_secs)
                }
            }
            Self::TrustFailed(e) => format!("Trust establishment failed: {}", e),
            Self::InvalidCode => "Code must be 4 digits".to_string(),
        }
    }
}

impl std::fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.user_message())
    }
}

impl std::error::Error for ConnectionError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_connection_code_expiry() {
        let expires_at = Instant::now() + Duration::from_secs(600);
        let code = ConnectionCode::new("1234".to_string(), expires_at);
        
        assert_eq!(code.code, "1234");
        assert!(!code.is_expired());
        assert!(code.remaining_seconds > 0);
    }

    #[test]
    fn test_connection_code_expired() {
        let expires_at = Instant::now() - Duration::from_secs(1);
        let code = ConnectionCode::new("1234".to_string(), expires_at);
        
        assert!(code.is_expired());
        assert_eq!(code.remaining_seconds, 0);
    }

    #[test]
    fn test_connection_error_messages() {
        assert_eq!(
            ConnectionError::CodeNotFound.user_message(),
            "Code not found - please check and try again"
        );
        
        assert_eq!(
            ConnectionError::CodeExpired.user_message(),
            "Code expired - ask for a new code"
        );
        
        assert_eq!(
            ConnectionError::NetworkError("timeout".to_string()).user_message(),
            "Connection failed - check network and try again"
        );
        
        assert_eq!(
            ConnectionError::AlreadyTrusted.user_message(),
            "Device already trusted"
        );
        
        assert_eq!(
            ConnectionError::RateLimited { remaining_secs: 300 }.user_message(),
            "Too many failed attempts - try again in 5 minutes"
        );
        
        assert_eq!(
            ConnectionError::RateLimited { remaining_secs: 45 }.user_message(),
            "Too many failed attempts - try again in 45 seconds"
        );
        
        assert_eq!(
            ConnectionError::InvalidCode.user_message(),
            "Code must be 4 digits"
        );
    }

    #[test]
    fn test_device_info_from_registration() {
        let reg = DeviceRegistration {
            code: "1234".to_string(),
            device_id: "test_device".to_string(),
            ip_address: "192.168.1.10".to_string(),
            bridge_port: 45679,
            hostname: "TestHost".to_string(),
            label: "Test Device".to_string(),
            os: OperatingSystem::Linux,
            created_at: Instant::now(),
        };
        
        let info: DeviceInfo = reg.into();
        
        assert_eq!(info.device_id, "test_device");
        assert_eq!(info.hostname, "TestHost");
        assert_eq!(info.label, "Test Device");
        assert_eq!(info.os, OperatingSystem::Linux);
        assert_eq!(info.ip_address, "192.168.1.10");
        assert_eq!(info.bridge_port, 45679);
    }

    #[test]
    fn test_connection_type_equality() {
        assert_eq!(ConnectionType::NewConnection, ConnectionType::NewConnection);
        assert_eq!(ConnectionType::AlreadyTrusted, ConnectionType::AlreadyTrusted);
        assert_eq!(ConnectionType::Reconnection, ConnectionType::Reconnection);
        assert_ne!(ConnectionType::NewConnection, ConnectionType::AlreadyTrusted);
    }
}
