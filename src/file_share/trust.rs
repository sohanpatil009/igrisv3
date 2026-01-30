// src/file_share/trust.rs
// Trust and pairing management for devices

use super::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

/// Trust manager for handling device pairing
pub struct TrustManager {
    trusted_devices: Arc<RwLock<HashMap<String, TrustedDevice>>>,
    config: FileShareConfig,
}

/// Trusted device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedDevice {
    pub device_info: DeviceInfo,
    pub paired_at: u64,
    pub expires_at: u64,
    pub verification_code: String,
}

impl TrustManager {
    /// Create new trust manager
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            trusted_devices: Arc::new(RwLock::new(HashMap::new())),
            config: FileShareConfig::default(),
        })
    }

    /// Check if device is trusted
    pub async fn is_trusted(&self, device_id: &str) -> bool {
        if let Some(trusted) = self.trusted_devices.read().await.get(device_id) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            
            now < trusted.expires_at
        } else {
            false
        }
    }

    /// Add trusted device
    pub async fn add_trusted_device(&self, device_info: DeviceInfo, verification_code: String) -> Result<(), Box<dyn std::error::Error>> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        
        let expires_at = now + (self.config.trust_duration_days as u64 * 24 * 60 * 60);
        
        let trusted = TrustedDevice {
            device_info: device_info.clone(),
            paired_at: now,
            expires_at,
            verification_code,
        };
        
        self.trusted_devices.write().await.insert(device_info.id.clone(), trusted);
        Ok(())
    }

    /// Remove trusted device
    pub async fn remove_trusted_device(&self, device_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.trusted_devices.write().await.remove(device_id);
        Ok(())
    }

    /// Get all trusted devices
    pub async fn get_trusted_devices(&self) -> Vec<DeviceInfo> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        self.trusted_devices.read().await.values()
            .filter(|t| now < t.expires_at)
            .map(|t| t.device_info.clone())
            .collect()
    }

    /// Generate verification code for pairing
    pub fn generate_verification_code() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        format!("{:06}", rng.gen_range(100000..=999999))
    }

    /// Verify pairing code
    pub async fn verify_pairing_code(&self, device_id: &str, code: &str) -> bool {
        if let Some(trusted) = self.trusted_devices.read().await.get(device_id) {
            trusted.verification_code == code
        } else {
            false
        }
    }
}
