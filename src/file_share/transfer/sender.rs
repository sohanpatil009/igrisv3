// File sender implementation

use super::{FileIntegrity, TransferProgress, TransferStatus};
use crate::file_share::discovery::Device;
use crate::file_share::protocol::{DeviceInfo, FileInfo, PrepareUploadRequest, PrepareUploadResponse};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

const CHUNK_SIZE: usize = 65536; // 64KB chunks

pub struct FileSender {
    device_info: DeviceInfo,
    progress: Arc<RwLock<HashMap<String, TransferProgress>>>,
}

impl FileSender {
    pub fn new(device_info: DeviceInfo) -> Self {
        Self {
            device_info,
            progress: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Send files to a device
    pub async fn send_files(
        &self,
        target_device: Device,
        file_paths: Vec<String>,
    ) -> Result<String> {
        // Prepare file metadata
        let mut files_map = HashMap::new();
        let mut total_size = 0u64;

        for (idx, path_str) in file_paths.iter().enumerate() {
            let path = Path::new(path_str);
            if !path.exists() {
                anyhow::bail!("File not found: {}", path_str);
            }

            let file_id = format!("file_{}", idx);
            let metadata = std::fs::metadata(path)?;
            let size = metadata.len();
            total_size += size;

            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let file_type = mime_guess::from_path(path)
                .first_or_octet_stream()
                .to_string();

            // Calculate SHA-256 hash
            let sha256 = FileIntegrity::calculate_hash(path).ok();

            let file_info = FileInfo {
                id: file_id.clone(),
                file_name,
                size,
                file_type,
                sha256,
                preview: None,
                metadata: None,
            };

            files_map.insert(file_id, file_info);
        }

        // Create prepare request
        let prepare_request = PrepareUploadRequest {
            info: self.device_info.clone(),
            files: files_map.clone(),
        };

        // Send prepare request
        let session_id = self.send_prepare_request(&target_device, &prepare_request).await?;

        // Initialize progress
        let progress = TransferProgress::new(session_id.clone(), total_size, file_paths.len());
        self.progress.write().await.insert(session_id.clone(), progress);

        // Send files
        self.transfer_files(&target_device, &session_id, file_paths, files_map)
            .await?;

        Ok(session_id)
    }

    async fn send_prepare_request(
        &self,
        device: &Device,
        request: &PrepareUploadRequest,
    ) -> Result<String> {
        let url = device.api_url("/prepare-upload");
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true) // For self-signed certs
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        let response = client
            .post(&url)
            .json(request)
            .send()
            .await
            .context("Failed to send prepare request")?;

        if !response.status().is_success() {
            anyhow::bail!("Prepare request failed: {}", response.status());
        }

        let prepare_response: PrepareUploadResponse = response
            .json::<PrepareUploadResponse>()
            .await
            .context("Failed to parse prepare response")?;

        Ok(prepare_response.session_id)
    }

    async fn transfer_files(
        &self,
        device: &Device,
        session_id: &str,
        file_paths: Vec<String>,
        files_map: HashMap<String, FileInfo>,
    ) -> Result<()> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(300))
            .build()?;

        for (idx, path_str) in file_paths.iter().enumerate() {
            let file_id = format!("file_{}", idx);
            let file_info = files_map.get(&file_id).unwrap();

            // Update progress
            {
                let mut progress_map = self.progress.write().await;
                if let Some(progress) = progress_map.get_mut(session_id) {
                    progress.status = TransferStatus::Transferring;
                    progress.current_file = Some(file_info.file_name.clone());
                }
            }

            // Send file
            self.send_file(&client, device, session_id, &file_id, path_str)
                .await?;

            // Update progress
            {
                let mut progress_map = self.progress.write().await;
                if let Some(progress) = progress_map.get_mut(session_id) {
                    progress.transferred_bytes += file_info.size;
                    progress.files_completed += 1;
                }
            }
        }

        // Mark as completed
        {
            let mut progress_map = self.progress.write().await;
            if let Some(progress) = progress_map.get_mut(session_id) {
                progress.status = TransferStatus::Completed;
                progress.current_file = None;
            }
        }

        Ok(())
    }

    async fn send_file(
        &self,
        client: &reqwest::Client,
        device: &Device,
        session_id: &str,
        file_id: &str,
        file_path: &str,
    ) -> Result<()> {
        let url = format!(
            "{}/upload?sessionId={}&fileId={}&token={}",
            device.api_url(""),
            session_id,
            file_id,
            "dummy_token" // In real implementation, use token from prepare response
        );

        let mut file = File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let response = client.post(&url).body(buffer).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("File upload failed: {}", response.status());
        }

        Ok(())
    }

    pub async fn get_progress(&self, session_id: &str) -> Option<TransferProgress> {
        self.progress.read().await.get(session_id).cloned()
    }

    pub async fn cancel_transfer(&self, session_id: &str) -> Result<()> {
        let mut progress_map = self.progress.write().await;
        if let Some(progress) = progress_map.get_mut(session_id) {
            progress.status = TransferStatus::Cancelled;
        }
        Ok(())
    }
}
