// LocalSend Protocol v2.1 Types

pub mod errors;
pub mod framing;
pub mod handshake;
pub mod messages;

pub use errors::ProtocolError;
pub use messages::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Protocol version
pub const PROTOCOL_VERSION: &str = "2.1";

/// Default port for LocalSend
pub const DEFAULT_PORT: u16 = 53317;

/// Default multicast address
pub const MULTICAST_ADDR: &str = "224.0.0.167";

/// Device types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    Mobile,
    Desktop,
    Web,
    Headless,
    Server,
}

/// Protocol type (HTTP or HTTPS)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Http,
    Https,
}

/// Device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub alias: String,
    pub version: String,
    #[serde(rename = "deviceModel")]
    pub device_model: Option<String>,
    #[serde(rename = "deviceType")]
    pub device_type: Option<DeviceType>,
    pub fingerprint: String,
    pub port: u16,
    pub protocol: Protocol,
    #[serde(default)]
    pub download: bool,
}

/// Announcement message (UDP multicast)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnouncementMessage {
    pub alias: String,
    pub version: String,
    #[serde(rename = "deviceModel")]
    pub device_model: Option<String>,
    #[serde(rename = "deviceType")]
    pub device_type: Option<DeviceType>,
    pub fingerprint: String,
    pub port: u16,
    pub protocol: Protocol,
    #[serde(default)]
    pub download: bool,
    pub announce: bool,
}

/// Register request/response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterMessage {
    pub alias: String,
    pub version: String,
    #[serde(rename = "deviceModel")]
    pub device_model: Option<String>,
    #[serde(rename = "deviceType")]
    pub device_type: Option<DeviceType>,
    pub fingerprint: String,
    pub port: u16,
    pub protocol: Protocol,
    #[serde(default)]
    pub download: bool,
}

/// File metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub id: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
    pub size: u64,
    #[serde(rename = "fileType")]
    pub file_type: String,
    pub sha256: Option<String>,
    pub preview: Option<String>,
    pub metadata: Option<FileMetadata>,
}

/// File metadata (timestamps)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub modified: Option<String>,
    pub accessed: Option<String>,
}

/// Prepare upload request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrepareUploadRequest {
    pub info: DeviceInfo,
    pub files: HashMap<String, FileInfo>,
}

/// Prepare upload response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrepareUploadResponse {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub files: HashMap<String, String>, // fileId -> token
}

/// Prepare download response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrepareDownloadResponse {
    pub info: DeviceInfo,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub files: HashMap<String, FileInfo>,
}

/// Session information
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub session_id: String,
    pub device: DeviceInfo,
    pub files: HashMap<String, FileInfo>,
    pub tokens: HashMap<String, String>,
}

impl DeviceInfo {
    pub fn new(alias: String, fingerprint: String, port: u16) -> Self {
        Self {
            alias,
            version: PROTOCOL_VERSION.to_string(),
            device_model: Some(get_device_model()),
            device_type: Some(DeviceType::Desktop),
            fingerprint,
            port,
            protocol: Protocol::Https,
            download: true,
        }
    }
}

fn get_device_model() -> String {
    #[cfg(target_os = "windows")]
    return "Windows".to_string();
    #[cfg(target_os = "macos")]
    return "macOS".to_string();
    #[cfg(target_os = "linux")]
    return "Linux".to_string();
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    return "Unknown".to_string();
}
