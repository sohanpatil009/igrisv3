// src/file_share/device.rs
// Device information and identification

use serde::{Serialize, Deserialize};
use std::net::IpAddr;
use uuid::Uuid;

/// Device information for file sharing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub device_type: DeviceType,
    pub os: String,
    pub version: String,
    pub ip_address: IpAddr,
    pub port: u16,
    pub capabilities: Vec<String>,
    pub last_seen: u64,
    pub is_trusted: bool,
}

/// Device types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceType {
    Desktop,
    Laptop,
    Mobile,
    Tablet,
    Server,
    Unknown,
}

impl DeviceInfo {
    /// Get current device information
    pub async fn current() -> Result<Self, Box<dyn std::error::Error>> {
        let id = Self::get_or_create_device_id().await?;
        let name = Self::get_device_name();
        let device_type = Self::detect_device_type();
        let os = Self::get_os_info();
        let version = env!("CARGO_PKG_VERSION").to_string();
        let ip_address = Self::get_local_ip().await?;
        let port = 45679; // Default transfer port
        
        Ok(Self {
            id,
            name,
            device_type,
            os,
            version,
            ip_address,
            port,
            capabilities: vec![
                "file_transfer".to_string(),
                "voice_commands".to_string(),
                "encryption".to_string(),
            ],
            last_seen: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            is_trusted: false,
        })
    }

    /// Get or create persistent device ID
    async fn get_or_create_device_id() -> Result<String, Box<dyn std::error::Error>> {
        let config_dir = Self::get_config_dir()?;
        let id_file = config_dir.join("device_id.txt");

        if id_file.exists() {
            match tokio::fs::read_to_string(&id_file).await {
                Ok(id) => {
                    let id = id.trim();
                    if !id.is_empty() {
                        return Ok(id.to_string());
                    }
                }
                Err(_) => {}
            }
        }

        // Create new ID
        let new_id = Uuid::new_v4().to_string();
        
        // Ensure directory exists
        tokio::fs::create_dir_all(&config_dir).await?;
        
        // Save ID
        tokio::fs::write(&id_file, &new_id).await?;
        
        Ok(new_id)
    }

    /// Get device name
    fn get_device_name() -> String {
        #[cfg(target_os = "windows")]
        {
            std::env::var("COMPUTERNAME")
                .unwrap_or_else(|_| "Windows PC".to_string())
        }
        
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("scutil")
                .args(&["--get", "ComputerName"])
                .output()
                .ok()
                .and_then(|output| String::from_utf8(output.stdout).ok())
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| "Mac".to_string())
        }
        
        #[cfg(target_os = "linux")]
        {
            std::env::var("HOSTNAME")
                .or_else(|_| std::fs::read_to_string("/etc/hostname"))
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|_| "Linux PC".to_string())
        }
        
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            "Unknown Device".to_string()
        }
    }

    /// Detect device type
    fn detect_device_type() -> DeviceType {
        #[cfg(target_os = "windows")]
        {
            // Check if it's a laptop by looking for battery
            if std::path::Path::new("C:\\Windows\\System32\\wbem\\wmic.exe").exists() {
                if let Ok(output) = std::process::Command::new("wmic")
                    .args(&["path", "win32_battery", "get", "name"])
                    .output()
                {
                    if String::from_utf8_lossy(&output.stdout).contains("Battery") {
                        return DeviceType::Laptop;
                    }
                }
            }
            DeviceType::Desktop
        }
        
        #[cfg(target_os = "macos")]
        {
            // Check if it's a MacBook
            if let Ok(output) = std::process::Command::new("system_profiler")
                .args(&["SPHardwareDataType"])
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if output_str.contains("MacBook") {
                    return DeviceType::Laptop;
                }
            }
            DeviceType::Desktop
        }
        
        #[cfg(target_os = "linux")]
        {
            // Check for laptop indicators
            if std::path::Path::new("/sys/class/power_supply/BAT0").exists() ||
               std::path::Path::new("/sys/class/power_supply/BAT1").exists() {
                return DeviceType::Laptop;
            }
            DeviceType::Desktop
        }
        
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            DeviceType::Unknown
        }
    }

    /// Get OS information
    fn get_os_info() -> String {
        #[cfg(target_os = "windows")]
        {
            format!("Windows {}", std::env::consts::ARCH)
        }
        
        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = std::process::Command::new("sw_vers")
                .args(&["-productVersion"])
                .output()
            {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                format!("macOS {} ({})", version, std::env::consts::ARCH)
            } else {
                format!("macOS ({})", std::env::consts::ARCH)
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
                for line in content.lines() {
                    if line.starts_with("PRETTY_NAME=") {
                        let name = line.split('=').nth(1)
                            .unwrap_or("Linux")
                            .trim_matches('"');
                        return format!("{} ({})", name, std::env::consts::ARCH);
                    }
                }
            }
            format!("Linux ({})", std::env::consts::ARCH)
        }
        
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            format!("Unknown OS ({})", std::env::consts::ARCH)
        }
    }

    /// Get local IP address
    async fn get_local_ip() -> Result<IpAddr, Box<dyn std::error::Error>> {
        // Try to connect to a remote address to determine local IP
        let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect("8.8.8.8:80").await?;
        let local_addr = socket.local_addr()?;
        Ok(local_addr.ip())
    }

    /// Get configuration directory
    fn get_config_dir() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        let config_dir = if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("IGRIS")
        } else {
            std::path::PathBuf::from(".").join("IGRIS")
        };
        
        Ok(config_dir)
    }

    /// Update last seen timestamp
    pub fn update_last_seen(&mut self) {
        self.last_seen = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// Check if device is online (seen within last 30 seconds)
    pub fn is_online(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        now - self.last_seen < 30
    }

    /// Get device icon based on type and OS
    pub fn get_icon(&self) -> &'static str {
        match self.device_type {
            DeviceType::Desktop => {
                if self.os.contains("Windows") {
                    "🖥️"
                } else if self.os.contains("macOS") {
                    "🖥️"
                } else {
                    "🖥️"
                }
            }
            DeviceType::Laptop => {
                if self.os.contains("Windows") {
                    "💻"
                } else if self.os.contains("macOS") {
                    "💻"
                } else {
                    "💻"
                }
            }
            DeviceType::Mobile => "📱",
            DeviceType::Tablet => "📱",
            DeviceType::Server => "🖥️",
            DeviceType::Unknown => "❓",
        }
    }

    /// Get display name with icon
    pub fn display_name(&self) -> String {
        format!("{} {}", self.get_icon(), self.name)
    }
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::Desktop => write!(f, "Desktop"),
            DeviceType::Laptop => write!(f, "Laptop"),
            DeviceType::Mobile => write!(f, "Mobile"),
            DeviceType::Tablet => write!(f, "Tablet"),
            DeviceType::Server => write!(f, "Server"),
            DeviceType::Unknown => write!(f, "Unknown"),
        }
    }
}