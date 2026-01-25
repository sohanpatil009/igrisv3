// src/file_share/trust.rs - Device Trust & Verification Manager

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use chrono::Utc;
use once_cell::sync::Lazy;

use super::config::{
    DeviceConfig, TrustedDevice, OperatingSystem,
    load_config, save_config,
};
use super::discovery::DiscoveredDevice;
use super::connection_types::DeviceInfo;

// Trust constants
const MAX_FAILED_ATTEMPTS: u32 = 3;
const BLOCK_DURATION_SECS: u64 = 300; // 5 minutes
const TRUST_EXPIRY_DAYS: i64 = 30;

/// Failed attempt tracking for rate limiting
#[derive(Debug, Clone)]
struct FailedAttempt {
    count: u32,
    last_attempt: Instant,
    blocked_until: Option<Instant>,
}

impl FailedAttempt {
    fn new() -> Self {
        FailedAttempt {
            count: 0,
            last_attempt: Instant::now(),
            blocked_until: None,
        }
    }
    
    fn is_blocked(&self) -> bool {
        if let Some(until) = self.blocked_until {
            Instant::now() < until
        } else {
            false
        }
    }
    
    fn block_remaining_secs(&self) -> u64 {
        if let Some(until) = self.blocked_until {
            if Instant::now() < until {
                return (until - Instant::now()).as_secs();
            }
        }
        0
    }
}

/// Trust verification result
#[derive(Debug, Clone)]
pub enum TrustResult {
    Success,
    InvalidCode,
    CodeExpired,
    Blocked { remaining_secs: u64 },
    AlreadyTrusted,
    DeviceNotFound,
    Error(String),
}

/// Trust Manager handles device trust relationships
pub struct TrustManager {
    /// Failed attempt tracking (device_id -> attempts)
    failed_attempts: HashMap<String, FailedAttempt>,
}

impl TrustManager {
    pub fn new() -> Self {
        TrustManager {
            failed_attempts: HashMap::new(),
        }
    }
    
    /// Establish trust with a device using DeviceInfo and certificate fingerprint
    /// This is the primary method for creating trust relationships
    pub fn establish_trust(&mut self, device_info: &DeviceInfo, cert_fingerprint: &str) -> Result<(), String> {
        let mut config = load_config()?;
        
        let trusted = TrustedDevice {
            id: device_info.device_id.clone(),
            label: device_info.label.clone(),
            os: device_info.os.clone(),
            cert_fingerprint: cert_fingerprint.to_string(),
            trusted_at: Utc::now(),
            last_connected: None,
        };
        
        config.add_trusted_device(trusted);
        save_config(&config)?;
        
        // Reset failed attempts on successful trust establishment
        self.failed_attempts.remove(&device_info.device_id);
        
        println!("[Trust] Established trust with device: {}", device_info.label);
        
        Ok(())
    }
    
    /// Check if a device is rate limited due to failed attempts
    /// Returns Ok(()) if not blocked, or Err with remaining seconds if blocked
    pub fn check_rate_limit(&self, device_id: &str) -> Result<(), u64> {
        if let Some(attempt) = self.failed_attempts.get(device_id) {
            if attempt.is_blocked() {
                return Err(attempt.block_remaining_secs());
            }
        }
        Ok(())
    }
    
    /// Record a failed verification attempt for rate limiting
    /// This should be called when a connection or verification fails
    pub fn record_failed_attempt(&mut self, device_id: &str) {
        let attempt = self.failed_attempts
            .entry(device_id.to_string())
            .or_insert_with(FailedAttempt::new);
        
        attempt.count += 1;
        attempt.last_attempt = Instant::now();
        
        if attempt.count >= MAX_FAILED_ATTEMPTS {
            attempt.blocked_until = Some(Instant::now() + Duration::from_secs(BLOCK_DURATION_SECS));
            println!("[Trust] Device {} blocked for {} seconds", &device_id[..8], BLOCK_DURATION_SECS);
        }
        
        println!("[Trust] Failed attempt {} for device {}", attempt.count, &device_id[..8]);
    }
    
    /// Add a device to trusted list (legacy method for DiscoveredDevice)
    pub fn add_trusted_device(&mut self, device: &DiscoveredDevice, cert_fingerprint: &str) -> Result<(), String> {
        let mut config = load_config()?;
        
        let trusted = TrustedDevice {
            id: device.id.clone(),
            label: device.label.clone(),
            os: device.os.clone(),
            cert_fingerprint: cert_fingerprint.to_string(),
            trusted_at: Utc::now(),
            last_connected: None,
        };
        
        config.add_trusted_device(trusted);
        save_config(&config)?;
        
        println!("[Trust] Added trusted device: {}", device.label);
        
        Ok(())
    }
    
    /// Remove a device from trusted list
    pub fn remove_trusted_device(&mut self, device_id: &str) -> Result<bool, String> {
        let mut config = load_config()?;
        let removed = config.remove_trusted_device(device_id);
        
        if removed {
            save_config(&config)?;
            println!("[Trust] Removed trusted device: {}", &device_id[..8]);
        }
        
        Ok(removed)
    }
    
    /// Rename a trusted device
    pub fn rename_device(&mut self, device_id: &str, new_label: &str) -> Result<bool, String> {
        let mut config = load_config()?;
        let renamed = config.rename_device(device_id, new_label);
        
        if renamed {
            save_config(&config)?;
            println!("[Trust] Renamed device {} to {}", &device_id[..8], new_label);
        }
        
        Ok(renamed)
    }
    
    /// Check if a device is trusted
    pub fn is_trusted(&self, device_id: &str) -> Result<bool, String> {
        let config = load_config()?;
        Ok(config.is_trusted(device_id))
    }
    
    /// Get all trusted devices
    pub fn get_trusted_devices(&self) -> Result<Vec<TrustedDevice>, String> {
        let config = load_config()?;
        Ok(config.trusted_devices)
    }
    
    /// Get expired trusted devices
    pub fn get_expired_devices(&self) -> Result<Vec<TrustedDevice>, String> {
        let config = load_config()?;
        Ok(config.trusted_devices.into_iter().filter(|d| d.is_expired()).collect())
    }
    
    /// Update last connected time
    pub fn update_last_connected(&mut self, device_id: &str) -> Result<(), String> {
        let mut config = load_config()?;
        config.update_last_connected(device_id);
        save_config(&config)?;
        Ok(())
    }
    
    /// Verify certificate fingerprint for a trusted device
    pub fn verify_certificate(&self, device_id: &str, cert_fingerprint: &str) -> Result<bool, String> {
        let config = load_config()?;
        
        if let Some(trusted) = config.get_trusted_device(device_id) {
            Ok(trusted.cert_fingerprint == cert_fingerprint)
        } else {
            Ok(false)
        }
    }
    
    /// Clean up expired blocks
    pub fn cleanup(&mut self) {
        // Reset old blocks (after block duration)
        let now = Instant::now();
        for attempt in self.failed_attempts.values_mut() {
            if let Some(until) = attempt.blocked_until {
                if now > until {
                    attempt.blocked_until = None;
                    attempt.count = 0;
                }
            }
        }
    }
}

// Global trust manager instance
static TRUST_MANAGER: Lazy<Arc<Mutex<TrustManager>>> = Lazy::new(|| {
    Arc::new(Mutex::new(TrustManager::new()))
});

/// Get the global trust manager
pub fn get_trust_manager() -> Arc<Mutex<TrustManager>> {
    TRUST_MANAGER.clone()
}

// Convenience functions

/// Establish trust with a device
pub fn establish_trust(device_info: &DeviceInfo, cert_fingerprint: &str) -> Result<(), String> {
    let manager = get_trust_manager();
    let mut manager = manager.lock().map_err(|e| format!("Lock error: {}", e))?;
    manager.establish_trust(device_info, cert_fingerprint)
}

/// Check if a device is rate limited
pub fn check_rate_limit(device_id: &str) -> Result<(), u64> {
    let manager = get_trust_manager();
    let manager = manager.lock().map_err(|_| 0u64)?;
    manager.check_rate_limit(device_id)
}

/// Record a failed attempt
pub fn record_failed_attempt(device_id: &str) -> Result<(), String> {
    let manager = get_trust_manager();
    let mut manager = manager.lock().map_err(|e| format!("Lock error: {}", e))?;
    manager.record_failed_attempt(device_id);
    Ok(())
}

/// Add a trusted device (legacy)
pub fn add_trusted(device: &DiscoveredDevice, cert_fingerprint: &str) -> Result<(), String> {
    let manager = get_trust_manager();
    let mut manager = manager.lock().map_err(|e| format!("Lock error: {}", e))?;
    manager.add_trusted_device(device, cert_fingerprint)
}

/// Remove a trusted device
pub fn remove_trusted(device_id: &str) -> Result<bool, String> {
    let manager = get_trust_manager();
    let mut manager = manager.lock().map_err(|e| format!("Lock error: {}", e))?;
    manager.remove_trusted_device(device_id)
}

/// Check if device is trusted
pub fn is_device_trusted(device_id: &str) -> Result<bool, String> {
    let manager = get_trust_manager();
    let manager = manager.lock().map_err(|e| format!("Lock error: {}", e))?;
    manager.is_trusted(device_id)
}

/// Get all trusted devices
pub fn get_all_trusted() -> Result<Vec<TrustedDevice>, String> {
    let manager = get_trust_manager();
    let manager = manager.lock().map_err(|e| format!("Lock error: {}", e))?;
    manager.get_trusted_devices()
}

/// Rename a trusted device
pub fn rename_trusted_device(device_id: &str, new_label: &str) -> Result<bool, String> {
    let manager = get_trust_manager();
    let mut manager = manager.lock().map_err(|e| format!("Lock error: {}", e))?;
    manager.rename_device(device_id, new_label)
}
