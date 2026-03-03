// Thin Rust client for Go file share backend
// Communicates via HTTP/WebSocket

use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub alias: String,
    pub ip: String,
    pub port: i32,
    pub fingerprint: String,
    pub device_type: String,
    pub last_seen: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    pub session_id: String,
    pub status: String,
    pub bytes_sent: i64,
    pub total_bytes: i64,
    pub from_device: String,
}

impl Transfer {
    pub fn progress(&self) -> f64 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        (self.bytes_sent as f64 / self.total_bytes as f64) * 100.0
    }
}

pub struct FileShareClient {
    base_url: String,
    client: reqwest::Client,
}

impl FileShareClient {
    pub fn new(port: u16) -> Self {
        Self {
            base_url: format!("http://localhost:{}", port),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
        }
    }

    /// Check if Go backend is running
    pub async fn is_running(&self) -> bool {
        self.client
            .get(format!("{}/health", self.base_url))
            .send()
            .await
            .is_ok()
    }

    /// Get list of discovered devices
    pub async fn get_devices(&self) -> Result<Vec<Device>, String> {
        let response = self
            .client
            .get(format!("{}/api/igris/devices", self.base_url))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("JSON parse failed: {}", e))?;

        let devices: Vec<Device> = serde_json::from_value(json["devices"].clone())
            .map_err(|e| format!("Device parse failed: {}", e))?;

        Ok(devices)
    }

    /// Get all transfers
    pub async fn get_transfers(&self) -> Result<Vec<Transfer>, String> {
        let response = self
            .client
            .get(format!("{}/api/igris/transfers", self.base_url))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("JSON parse failed: {}", e))?;

        let transfers: Vec<Transfer> = serde_json::from_value(json["transfers"].clone())
            .map_err(|e| format!("Transfer parse failed: {}", e))?;

        Ok(transfers)
    }

    /// Get specific transfer
    pub async fn get_transfer(&self, session_id: &str) -> Result<Transfer, String> {
        let response = self
            .client
            .get(format!("{}/api/igris/transfer/{}", self.base_url, session_id))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let transfer: Transfer = response
            .json()
            .await
            .map_err(|e| format!("JSON parse failed: {}", e))?;

        Ok(transfer)
    }

    /// Cancel transfer
    pub async fn cancel_transfer(&self, session_id: &str) -> Result<(), String> {
        self.client
            .delete(format!("{}/api/igris/transfer/{}", self.base_url, session_id))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        Ok(())
    }

    /// Send file to device
    pub async fn send_file(&self, device_id: &str, file_path: &str) -> Result<String, String> {
        use std::path::Path;
        
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", file_path));
        }

        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| "Invalid file name".to_string())?;

        let file_size = std::fs::metadata(path)
            .map_err(|e| format!("Failed to get file size: {}", e))?
            .len();

        // Prepare upload
        let prepare_body = serde_json::json!({
            "info": {
                "alias": "IGRIS",
                "deviceType": "desktop"
            },
            "files": {
                file_name: {
                    "id": uuid::Uuid::new_v4().to_string(),
                    "fileName": file_name,
                    "size": file_size,
                    "fileType": "file"
                }
            }
        });

        let response = self.client
            .post(format!("{}/api/localsend/v2/prepare-upload", self.base_url))
            .json(&prepare_body)
            .send()
            .await
            .map_err(|e| format!("Prepare upload failed: {}", e))?;

        let prepare_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse prepare response: {}", e))?;

        let session_id = prepare_response["sessionId"]
            .as_str()
            .ok_or_else(|| "No session ID in response".to_string())?
            .to_string();

        // Upload file
        let file_bytes = std::fs::read(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let file_part = reqwest::multipart::Part::bytes(file_bytes)
            .file_name(file_name.to_string());

        let form = reqwest::multipart::Form::new()
            .part("file", file_part);

        self.client
            .post(format!("{}/api/localsend/v2/upload?sessionId={}&fileId={}", 
                self.base_url, session_id, file_name))
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("Upload failed: {}", e))?;

        Ok(session_id)
    }

    /// Send multiple files to device
    pub async fn send_files(&self, device_id: &str, file_paths: Vec<String>) -> Result<Vec<String>, String> {
        let mut session_ids = Vec::new();
        
        for file_path in file_paths {
            match self.send_file(device_id, &file_path).await {
                Ok(session_id) => session_ids.push(session_id),
                Err(e) => eprintln!("Failed to send {}: {}", file_path, e),
            }
        }

        if session_ids.is_empty() {
            Err("All file transfers failed".to_string())
        } else {
            Ok(session_ids)
        }
    }
}
