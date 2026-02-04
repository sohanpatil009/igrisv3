// Device representation

use crate::file_share::protocol::{DeviceInfo, DeviceType, Protocol};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::time::{SystemTime, UNIX_EPOCH};

/// Discovered device
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Device {
    pub id: String,
    pub alias: String,
    pub ip: IpAddr,
    pub port: u16,
    pub device_type: Option<DeviceType>,
    pub device_model: Option<String>,
    pub fingerprint: String,
    pub protocol: Protocol,
    pub download: bool,
    pub last_seen: u64,
}

impl Device {
    pub fn from_device_info(info: DeviceInfo, ip: IpAddr) -> Self {
        let id = format!("{}:{}", ip, info.port);
        Self {
            id,
            alias: info.alias,
            ip,
            port: info.port,
            device_type: info.device_type,
            device_model: info.device_model,
            fingerprint: info.fingerprint,
            protocol: info.protocol,
            download: info.download,
            last_seen: current_timestamp(),
        }
    }

    pub fn update_last_seen(&mut self) {
        self.last_seen = current_timestamp();
    }

    pub fn is_stale(&self, timeout_secs: u64) -> bool {
        let now = current_timestamp();
        now - self.last_seen > timeout_secs
    }

    pub fn base_url(&self) -> String {
        let protocol = match self.protocol {
            Protocol::Http => "http",
            Protocol::Https => "https",
        };
        format!("{}://{}:{}", protocol, self.ip, self.port)
    }

    pub fn api_url(&self, path: &str) -> String {
        format!("{}/api/localsend/v2{}", self.base_url(), path)
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
