// src/file_share/config.rs - Device Identity & Trust Configuration

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use std::fs;
use sha2::{Sha256, Digest};
use rand::Rng;

/// Operating system type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperatingSystem {
    Windows,
    MacOS,
    Linux,
    Unknown,
}

impl OperatingSystem {
    pub fn current() -> Self {
        #[cfg(target_os = "windows")]
        return OperatingSystem::Windows;
        
        #[cfg(target_os = "macos")]
        return OperatingSystem::MacOS;
        
        #[cfg(target_os = "linux")]
        return OperatingSystem::Linux;
        
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        return OperatingSystem::Linux; // Default fallback
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            OperatingSystem::Windows => "Windows",
            OperatingSystem::MacOS => "macOS",
            OperatingSystem::Linux => "Linux",
            OperatingSystem::Unknown => "Unknown",
        }
    }
}

/// Device identity - unique per IGRIS installation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceIdentity {
    /// Unique device ID (SHA-256 fingerprint)
    pub id: String,
    /// User-defined label for this device
    pub label: String,
    /// Operating system
    pub os: OperatingSystem,
    /// Hostname
    pub hostname: String,
    /// Random salt used for fingerprint generation
    pub salt: String,
    /// When this identity was created
    pub created_at: DateTime<Utc>,
}

impl DeviceIdentity {
    /// Generate a new device identity
    pub fn generate() -> Self {
        let hostname = get_hostname();
        let salt = generate_random_salt();
        let id = generate_fingerprint(&hostname, &salt);
        
        DeviceIdentity {
            id,
            label: hostname.clone(),
            os: OperatingSystem::current(),
            hostname,
            salt,
            created_at: Utc::now(),
        }
    }
}

/// A trusted device that can connect automatically
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedDevice {
    /// Device fingerprint ID
    pub id: String,
    /// User-defined label
    pub label: String,
    /// Operating system
    pub os: OperatingSystem,
    /// TLS certificate fingerprint for verification
    pub cert_fingerprint: String,
    /// When trust was established
    pub trusted_at: DateTime<Utc>,
    /// Last successful connection
    pub last_connected: Option<DateTime<Utc>>,
}

impl TrustedDevice {
    /// Check if trust has expired (30 days without connection)
    pub fn is_expired(&self) -> bool {
        match self.last_connected {
            Some(last) => {
                let days_since = (Utc::now() - last).num_days();
                days_since > 30
            }
            None => {
                // If never connected, check from trust establishment
                let days_since = (Utc::now() - self.trusted_at).num_days();
                days_since > 30
            }
        }
    }
}

/// Main configuration for file sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    /// This device's identity
    pub identity: DeviceIdentity,
    /// List of trusted devices
    pub trusted_devices: Vec<TrustedDevice>,
    /// Bridge port for secure connections
    pub bridge_port: u16,
    /// Discovery port for UDP multicast
    pub discovery_port: u16,
}

impl Default for DeviceConfig {
    fn default() -> Self {
        DeviceConfig {
            identity: DeviceIdentity::generate(),
            trusted_devices: Vec::new(),
            bridge_port: 45679,
            discovery_port: 45678,
        }
    }
}

impl DeviceConfig {
    /// Add a trusted device
    pub fn add_trusted_device(&mut self, device: TrustedDevice) {
        // Remove if already exists (update)
        self.trusted_devices.retain(|d| d.id != device.id);
        self.trusted_devices.push(device);
    }
    
    /// Remove a trusted device
    pub fn remove_trusted_device(&mut self, device_id: &str) -> bool {
        let len_before = self.trusted_devices.len();
        self.trusted_devices.retain(|d| d.id != device_id);
        self.trusted_devices.len() < len_before
    }
    
    /// Check if a device is trusted
    pub fn is_trusted(&self, device_id: &str) -> bool {
        self.trusted_devices.iter().any(|d| d.id == device_id && !d.is_expired())
    }
    
    /// Get a trusted device by ID
    pub fn get_trusted_device(&self, device_id: &str) -> Option<&TrustedDevice> {
        self.trusted_devices.iter().find(|d| d.id == device_id)
    }
    
    /// Get mutable trusted device by ID
    pub fn get_trusted_device_mut(&mut self, device_id: &str) -> Option<&mut TrustedDevice> {
        self.trusted_devices.iter_mut().find(|d| d.id == device_id)
    }
    
    /// Update last connected time for a device
    pub fn update_last_connected(&mut self, device_id: &str) {
        if let Some(device) = self.get_trusted_device_mut(device_id) {
            device.last_connected = Some(Utc::now());
        }
    }
    
    /// Rename a trusted device
    pub fn rename_device(&mut self, device_id: &str, new_label: &str) -> bool {
        if let Some(device) = self.get_trusted_device_mut(device_id) {
            device.label = new_label.to_string();
            true
        } else {
            false
        }
    }
    
    /// Get all expired devices
    pub fn get_expired_devices(&self) -> Vec<&TrustedDevice> {
        self.trusted_devices.iter().filter(|d| d.is_expired()).collect()
    }
}

/// Get platform-specific config directory path
pub fn get_config_path() -> PathBuf {
    let base = if cfg!(target_os = "windows") {
        dirs::config_dir().unwrap_or_else(|| PathBuf::from("."))
    } else if cfg!(target_os = "macos") {
        dirs::config_dir().unwrap_or_else(|| PathBuf::from("."))
    } else {
        dirs::config_dir().unwrap_or_else(|| PathBuf::from("."))
    };
    
    base.join("IGRIS")
}

/// Get the full path to the config file
pub fn get_config_file_path() -> PathBuf {
    get_config_path().join("file_share.json")
}

/// Load configuration from disk
pub fn load_config() -> Result<DeviceConfig, String> {
    let path = get_config_file_path();
    
    if !path.exists() {
        // First time - create and save config
        let config = DeviceConfig::default();
        save_config(&config)?;
        println!("[FileShare] Created new device identity: {}", &config.identity.id[..8]);
        return Ok(config);
    }
    
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read config: {}", e))?;
    
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config: {}", e))
}

/// Save configuration to disk
pub fn save_config(config: &DeviceConfig) -> Result<(), String> {
    let path = get_config_file_path();
    
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }
    
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    fs::write(&path, content)
        .map_err(|e| format!("Failed to write config: {}", e))?;
    
    Ok(())
}

/// Get or create device identity
pub fn get_or_create_device_identity() -> Result<DeviceIdentity, String> {
    let config = load_config()?;
    Ok(config.identity)
}

fn generate_random_salt() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 16] = rng.gen();
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes)
}

fn generate_fingerprint(hostname: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(hostname.as_bytes());
    hasher.update(salt.as_bytes());
    
    if let Ok(machine_id) = get_machine_id() {
        hasher.update(machine_id.as_bytes());
    }
    
    let result = hasher.finalize();
    format!("{:x}", result)
}

fn get_hostname() -> String {
    hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "IGRIS-Device".to_string())
}

fn get_machine_id() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let output = Command::new("wmic")
            .args(["csproduct", "get", "UUID"])
            .output()
            .map_err(|e| e.to_string())?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let uuid = stdout.lines()
            .nth(1)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        
        Ok(uuid)
    }
    
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let output = Command::new("ioreg")
            .args(["-rd1", "-c", "IOPlatformExpertDevice"])
            .output()
            .map_err(|e| e.to_string())?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("IOPlatformUUID") {
                if let Some(uuid) = line.split('"').nth(3) {
                    return Ok(uuid.to_string());
                }
            }
        }
        Ok(String::new())
    }
    
    #[cfg(target_os = "linux")]
    {
        if let Ok(id) = fs::read_to_string("/etc/machine-id") {
            return Ok(id.trim().to_string());
        }
        if let Ok(id) = fs::read_to_string("/var/lib/dbus/machine-id") {
            return Ok(id.trim().to_string());
        }
        Ok(String::new())
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Ok(String::new())
    }
}

pub fn generate_device_fingerprint() -> String {
    let hostname = get_hostname();
    let salt = generate_random_salt();
    generate_fingerprint(&hostname, &salt)
}
