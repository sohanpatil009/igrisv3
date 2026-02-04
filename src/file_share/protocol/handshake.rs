// Handshake and connection establishment

use super::{DeviceInfo, RegisterMessage};
use anyhow::Result;

/// Handshake manager for establishing connections
pub struct HandshakeManager {
    device_info: DeviceInfo,
}

impl HandshakeManager {
    pub fn new(device_info: DeviceInfo) -> Self {
        Self { device_info }
    }

    /// Create a register message for discovery
    pub fn create_register_message(&self) -> RegisterMessage {
        RegisterMessage {
            alias: self.device_info.alias.clone(),
            version: self.device_info.version.clone(),
            device_model: self.device_info.device_model.clone(),
            device_type: self.device_info.device_type.clone(),
            fingerprint: self.device_info.fingerprint.clone(),
            port: self.device_info.port,
            protocol: self.device_info.protocol.clone(),
            download: self.device_info.download,
        }
    }

    /// Validate incoming register message
    pub fn validate_register(&self, msg: &RegisterMessage) -> Result<bool> {
        // Don't register self
        if msg.fingerprint == self.device_info.fingerprint {
            return Ok(false);
        }

        // Check protocol version compatibility
        let their_version = msg.version.split('.').next().unwrap_or("0");
        let our_version = self.device_info.version.split('.').next().unwrap_or("0");
        
        if their_version != our_version {
            return Ok(false);
        }

        Ok(true)
    }
}
