// Protocol message types

use serde::{Deserialize, Serialize};

/// Info endpoint response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoResponse {
    pub alias: String,
    pub version: String,
    #[serde(rename = "deviceModel")]
    pub device_model: Option<String>,
    #[serde(rename = "deviceType")]
    pub device_type: Option<String>,
    pub fingerprint: String,
    #[serde(default)]
    pub download: bool,
}

/// Error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub message: String,
}

impl ErrorResponse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}
