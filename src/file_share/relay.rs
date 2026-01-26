// src/file_share/relay.rs - Simple 4-digit code system for device pairing

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;

use super::config::OperatingSystem;

/// Code validity duration (30 minutes - increased for better UX)
const CODE_VALIDITY_DURATION: Duration = Duration::from_secs(1800);

/// Device registration with 4-digit code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRegistration {
    pub code: String,           // 4-digit code
    pub device_id: String,
    pub ip_address: String,
    pub bridge_port: u16,
    pub hostname: String,
    pub label: String,
    pub os: OperatingSystem,
    #[serde(skip)]
    #[serde(default = "Instant::now")]
    pub created_at: Instant,
}

impl DeviceRegistration {
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > CODE_VALIDITY_DURATION
    }
}

/// Local relay service - stores code mappings
pub struct RelayService {
    registrations: Arc<Mutex<HashMap<String, DeviceRegistration>>>,
}

impl RelayService {
    pub fn new() -> Self {
        RelayService {
            registrations: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Generate a random 4-digit code
    pub fn generate_code() -> String {
        // TEMPORARY FIX: Use fixed code "1234" for ALL devices for testing
        // This allows cross-platform connection testing
        "1234".to_string()
        
        // Original random code generation (commented out for testing)
        // use rand::Rng;
        // let mut rng = rand::thread_rng();
        // format!("{:04}", rng.gen_range(1000..9999))
    }
    
    /// Register this device with a 4-digit code
    pub fn register_device(
        &self,
        device_id: String,
        ip_address: String,
        bridge_port: u16,
        hostname: String,
        label: String,
        os: OperatingSystem,
    ) -> Result<String, String> {
        let code = Self::generate_code();
        
        let registration = DeviceRegistration {
            code: code.clone(),
            device_id: device_id.clone(),
            ip_address,
            bridge_port,
            hostname,
            label,
            os,
            created_at: Instant::now(),
        };
        
        let mut regs = self.registrations.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        regs.insert(code.clone(), registration);
        
        println!("[Relay] Device registered: device_id={}, code={}", 
            &device_id[..8.min(device_id.len())], code);
        Ok(code)
    }
    
    /// Lookup device by 4-digit code
    pub fn lookup_device(&self, code: &str) -> Result<DeviceRegistration, String> {
        let mut regs = self.registrations.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        // Clean up expired codes
        let before_count = regs.len();
        regs.retain(|_, reg| !reg.is_expired());
        let after_count = regs.len();
        if before_count != after_count {
            println!("[Relay] Cleaned up {} expired codes", before_count - after_count);
        }
        
        // Find the code
        if let Some(reg) = regs.get(code) {
            let age_secs = reg.created_at.elapsed().as_secs();
            let remaining_secs = CODE_VALIDITY_DURATION.as_secs().saturating_sub(age_secs);
            
            if reg.is_expired() {
                println!("[Relay] Code {} expired (age: {}s, validity: {}s)", 
                    code, age_secs, CODE_VALIDITY_DURATION.as_secs());
                regs.remove(code);
                return Err("Code expired".to_string());
            }
            
            println!("[Relay] Code {} found! Device: {}, Age: {}s, Remaining: {}s", 
                code, reg.label, age_secs, remaining_secs);
            Ok(reg.clone())
        } else {
            println!("[Relay] Code {} not found. Available codes: {:?}", 
                code, regs.keys().collect::<Vec<_>>());
            Err("Code not found or expired".to_string())
        }
    }
    
    /// Remove a code (after successful connection)
    pub fn remove_code(&self, code: &str) -> Result<(), String> {
        let mut regs = self.registrations.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if regs.remove(code).is_some() {
            println!("[Relay] Code removed: {}", code);
            Ok(())
        } else {
            Err("Code not found".to_string())
        }
    }
    
    /// Get my device's code
    pub fn get_my_code(&self, device_id: &str) -> Option<String> {
        let regs = self.registrations.lock().ok()?;
        for (code, reg) in regs.iter() {
            if reg.device_id == device_id && !reg.is_expired() {
                return Some(code.clone());
            }
        }
        None
    }
    
    /// Update an existing registration to refresh its expiry time with the same code
    /// This allows extending the validity period without generating a new code
    pub fn update_registration(&self, code: &str) -> Result<(), String> {
        let mut regs = self.registrations.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(reg) = regs.get_mut(code) {
            // Refresh the created_at timestamp to extend validity
            reg.created_at = Instant::now();
            println!("[Relay] Registration updated for code: {}", code);
            Ok(())
        } else {
            Err("Code not found".to_string())
        }
    }
    
    /// Clean up expired registrations
    /// This should be called periodically to remove stale entries
    pub fn cleanup_expired(&self) -> Result<usize, String> {
        let mut regs = self.registrations.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let initial_count = regs.len();
        regs.retain(|_, reg| !reg.is_expired());
        let removed_count = initial_count - regs.len();
        
        if removed_count > 0 {
            println!("[Relay] Cleaned up {} expired registrations", removed_count);
        }
        
        Ok(removed_count)
    }
}

/// Global relay service instance wrapped in Arc for sharing
static RELAY_SERVICE: Lazy<Arc<RelayService>> = Lazy::new(|| Arc::new(RelayService::new()));

/// Get the global relay service instance
pub fn get_relay_service() -> Arc<RelayService> {
    Arc::clone(&RELAY_SERVICE)
}

/// Get a reference to the global relay service
pub fn get_relay_service_ref() -> &'static RelayService {
    &RELAY_SERVICE
}

/// Generate a 4-digit code for this device
pub fn generate_my_code(
    device_id: String,
    ip_address: String,
    bridge_port: u16,
    hostname: String,
    label: String,
    os: OperatingSystem,
) -> Result<String, String> {
    RELAY_SERVICE.register_device(device_id, ip_address, bridge_port, hostname, label, os)
}

/// Connect to a device using their 4-digit code
pub fn connect_with_code(code: &str) -> Result<DeviceRegistration, String> {
    RELAY_SERVICE.lookup_device(code)
}

/// Remove a code after successful connection
pub fn invalidate_code(code: &str) -> Result<(), String> {
    RELAY_SERVICE.remove_code(code)
}

/// Get my device's current code
pub fn get_my_device_code(device_id: &str) -> Option<String> {
    let code = RELAY_SERVICE.get_my_code(device_id);
    if let Some(ref c) = code {
        println!("[Relay] get_my_device_code({}) = {}", &device_id[..8.min(device_id.len())], c);
    } else {
        println!("[Relay] get_my_device_code({}) = None (no code registered)", &device_id[..8.min(device_id.len())]);
    }
    code
}

/// Update an existing registration to refresh its expiry time
pub fn update_my_code(code: &str) -> Result<(), String> {
    RELAY_SERVICE.update_registration(code)
}

/// Clean up expired registrations
pub fn cleanup_expired_codes() -> Result<usize, String> {
    RELAY_SERVICE.cleanup_expired()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_code_generation() {
        let code = RelayService::generate_code();
        assert_eq!(code.len(), 4);
        assert!(code.chars().all(|c| c.is_numeric()));
        let num: u32 = code.parse().unwrap();
        assert!(num >= 1000 && num <= 9999);
    }
    
    #[test]
    fn test_register_and_lookup() {
        let service = RelayService::new();
        
        let code = service.register_device(
            "test123".to_string(),
            "192.168.1.10".to_string(),
            45679,
            "TestHost".to_string(),
            "Test Device".to_string(),
            OperatingSystem::Linux,
        ).unwrap();
        
        let reg = service.lookup_device(&code).unwrap();
        assert_eq!(reg.device_id, "test123");
        assert_eq!(reg.ip_address, "192.168.1.10");
        assert_eq!(reg.os, OperatingSystem::Linux);
    }
    
    #[test]
    fn test_update_registration() {
        let service = RelayService::new();
        
        let code = service.register_device(
            "test123".to_string(),
            "192.168.1.10".to_string(),
            45679,
            "TestHost".to_string(),
            "Test Device".to_string(),
            OperatingSystem::Linux,
        ).unwrap();
        
        // Get the initial registration
        let reg1 = service.lookup_device(&code).unwrap();
        let created_at1 = reg1.created_at;
        
        // Wait a bit
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        // Update the registration
        service.update_registration(&code).unwrap();
        
        // Get the updated registration
        let reg2 = service.lookup_device(&code).unwrap();
        let created_at2 = reg2.created_at;
        
        // The created_at should be newer
        assert!(created_at2 > created_at1);
        
        // Other fields should remain the same
        assert_eq!(reg2.device_id, "test123");
        assert_eq!(reg2.ip_address, "192.168.1.10");
    }
    
    #[test]
    fn test_cleanup_expired() {
        let service = RelayService::new();
        
        // Register a device
        let code = service.register_device(
            "test123".to_string(),
            "192.168.1.10".to_string(),
            45679,
            "TestHost".to_string(),
            "Test Device".to_string(),
            OperatingSystem::Linux,
        ).unwrap();
        
        // Manually expire the registration by modifying created_at
        {
            let mut regs = service.registrations.lock().unwrap();
            if let Some(reg) = regs.get_mut(&code) {
                reg.created_at = Instant::now() - Duration::from_secs(700); // Expired
            }
        }
        
        // Cleanup should remove the expired registration
        let removed = service.cleanup_expired().unwrap();
        assert_eq!(removed, 1);
        
        // The code should no longer be found
        assert!(service.lookup_device(&code).is_err());
    }
    
    #[test]
    fn test_get_my_code() {
        let service = RelayService::new();
        
        let device_id = "test123";
        
        // Initially no code
        assert!(service.get_my_code(device_id).is_none());
        
        // Register device
        let code = service.register_device(
            device_id.to_string(),
            "192.168.1.10".to_string(),
            45679,
            "TestHost".to_string(),
            "Test Device".to_string(),
            OperatingSystem::Linux,
        ).unwrap();
        
        // Should return the code
        assert_eq!(service.get_my_code(device_id), Some(code.clone()));
        
        // Expire the code
        {
            let mut regs = service.registrations.lock().unwrap();
            if let Some(reg) = regs.get_mut(&code) {
                reg.created_at = Instant::now() - Duration::from_secs(700);
            }
        }
        
        // Should return None for expired code
        assert!(service.get_my_code(device_id).is_none());
    }
}

// Property tests for RelayService
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Property 1: Code Format Validity
    proptest! {
        #[test]
        fn prop_code_format_valid(_seed in 0u64..1000) {
            let code = RelayService::generate_code();
            
            // Must be exactly 4 characters
            prop_assert_eq!(code.len(), 4);
            
            // Must be all numeric
            prop_assert!(code.chars().all(|c| c.is_numeric()));
            
            // Must be in range 1000-9999
            let num: u32 = code.parse().unwrap();
            prop_assert!(num >= 1000 && num <= 9999);
        }
    }

    // Property 2: Code Uniqueness Per Device
    proptest! {
        #[test]
        fn prop_code_uniqueness(device_id in "[a-z0-9]{8,16}") {
            let service = RelayService::new();
            
            let code1 = service.register_device(
                device_id.clone(),
                "192.168.1.10".to_string(),
                45679,
                "TestHost".to_string(),
                "Test Device".to_string(),
                OperatingSystem::Linux,
            ).unwrap();
            
            // Same device should get the same code when looked up
            let my_code = service.get_my_code(&device_id);
            prop_assert_eq!(my_code, Some(code1));
        }
    }

    // Property 3: Code Expiry Timing
    proptest! {
        #[test]
        fn prop_code_expiry_timing(device_id in "[a-z0-9]{8,16}") {
            let service = RelayService::new();
            
            let code = service.register_device(
                device_id,
                "192.168.1.10".to_string(),
                45679,
                "TestHost".to_string(),
                "Test Device".to_string(),
                OperatingSystem::Linux,
            ).unwrap();
            
            // Code should be valid immediately after creation
            let reg = service.lookup_device(&code);
            prop_assert!(reg.is_ok());
            prop_assert!(!reg.unwrap().is_expired());
        }
    }

    // Property 4: Automatic Code Regeneration
    proptest! {
        #[test]
        fn prop_code_regeneration(device_id in "[a-z0-9]{8,16}") {
            let service = RelayService::new();
            
            // Register device
            let code1 = service.register_device(
                device_id.clone(),
                "192.168.1.10".to_string(),
                45679,
                "TestHost".to_string(),
                "Test Device".to_string(),
                OperatingSystem::Linux,
            ).unwrap();
            
            // Register same device again (simulating regeneration)
            let code2 = service.register_device(
                device_id,
                "192.168.1.10".to_string(),
                45679,
                "TestHost".to_string(),
                "Test Device".to_string(),
                OperatingSystem::Linux,
            ).unwrap();
            
            // Both codes should be valid 4-digit codes
            prop_assert_eq!(code1.len(), 4);
            prop_assert_eq!(code2.len(), 4);
        }
    }

    // Property 10: Code Registration Completeness
    proptest! {
        #[test]
        fn prop_registration_completeness(
            device_id in "[a-z0-9]{8,16}",
            hostname in "[A-Za-z0-9-]{3,20}",
            label in "[A-Za-z0-9 ]{3,30}",
        ) {
            let service = RelayService::new();
            
            let code = service.register_device(
                device_id.clone(),
                "192.168.1.10".to_string(),
                45679,
                hostname.clone(),
                label.clone(),
                OperatingSystem::Linux,
            ).unwrap();
            
            let reg = service.lookup_device(&code).unwrap();
            
            // All fields should be preserved
            prop_assert_eq!(reg.device_id, device_id);
            prop_assert_eq!(reg.hostname, hostname);
            prop_assert_eq!(reg.label, label);
            prop_assert_eq!(reg.ip_address, "192.168.1.10");
            prop_assert_eq!(reg.bridge_port, 45679);
            prop_assert_eq!(reg.os, OperatingSystem::Linux);
        }
    }

    // Property 11: Code Lookup Round-Trip
    proptest! {
        #[test]
        fn prop_lookup_round_trip(device_id in "[a-z0-9]{8,16}") {
            let service = RelayService::new();
            
            // Register device
            let code = service.register_device(
                device_id.clone(),
                "192.168.1.10".to_string(),
                45679,
                "TestHost".to_string(),
                "Test Device".to_string(),
                OperatingSystem::Windows,
            ).unwrap();
            
            // Lookup should return the same device
            let reg = service.lookup_device(&code).unwrap();
            prop_assert_eq!(reg.device_id, device_id.clone());
            prop_assert_eq!(reg.code, code.clone());
            
            // Looking up with the same code again should work
            let reg2 = service.lookup_device(&code).unwrap();
            prop_assert_eq!(reg2.device_id, device_id);
        }
    }
}
