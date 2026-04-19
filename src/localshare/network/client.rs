use crate::localshare::models::*;
use anyhow::Result;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use uuid::Uuid;

pub struct TransferClient {
    client: reqwest::Client,
    progress_tracker: ProgressTracker,
}

impl TransferClient {
    pub fn new(progress_tracker: ProgressTracker) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(300))
                .build()
                .unwrap(),
            progress_tracker,
        }
    }

    pub async fn send_files(
        &self,
        target: &Device,
        files: Vec<PathBuf>,
        local_device: &Device,
    ) -> Result<String> {
        tracing::info!("Sending {} files to {}", files.len(), target.alias);

        let local_session_id = Uuid::new_v4().to_string();

        // Prepare file info
        let mut file_infos = Vec::new();
        let mut file_progresses = Vec::new();
        
        for path in &files {
            let metadata = tokio::fs::metadata(path).await?;
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            
            let file_id = Uuid::new_v4().to_string();

            file_infos.push(FileInfo {
                id: file_id.clone(),
                file_name: file_name.clone(),
                size: metadata.len(),
                file_type: mime_guess::from_path(path)
                    .first_or_octet_stream()
                    .to_string(),
                preview: None,
            });

            file_progresses.push(FileProgress::new(
                file_id,
                file_name,
                metadata.len(),
            ));
        }

        // Initialize progress tracking with local session ID
        let transfer_progress = TransferProgress::new(local_session_id.clone(), file_progresses);
        self.progress_tracker.write().await.insert(local_session_id.clone(), transfer_progress);

        // Prepare upload request
        let prepare_request = PrepareUploadRequest {
            info: DeviceInfo {
                alias: local_device.alias.clone(),
                version: "2.0".to_string(),
                device_model: local_device.device_model.clone(),
                device_type: format!("{:?}", local_device.device_type).to_lowercase(),
                fingerprint: local_device.id.clone(),
            },
            files: file_infos.clone(),
        };

        let url = format!("http://{}:{}/api/localsend/v2/prepare-upload", target.ip, target.port);
        
        tracing::info!("Preparing upload to {}", url);
        let response = self.client
            .post(&url)
            .json(&prepare_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            
            // Mark as failed
            let mut tracker = self.progress_tracker.write().await;
            if let Some(progress) = tracker.get_mut(&local_session_id) {
                for file in &mut progress.files {
                    file.status = ProgressStatus::Failed(format!("Prepare failed: {}", status));
                }
            }
            
            anyhow::bail!("Failed to prepare upload: {} - {}", status, body);
        }

        let prepare_response: PrepareUploadResponse = response.json().await?;
        let server_session_id = prepare_response.session_id.clone();
        tracing::info!("Upload prepared, server session: {}", server_session_id);

        // Step 3: Send confirmation (ACK) - Complete the three-way handshake
        let confirm_url = format!("http://{}:{}/api/localsend/v2/confirm-upload", target.ip, target.port);
        let confirm_request = ConfirmUploadRequest {
            session_id: server_session_id.clone(),
        };
        
        tracing::info!("Sending confirmation handshake...");
        let confirm_response = self.client
            .post(&confirm_url)
            .json(&confirm_request)
            .send()
            .await?;
        
        if !confirm_response.status().is_success() {
            let status = confirm_response.status();
            let body = confirm_response.text().await.unwrap_or_default();
            
            let mut tracker = self.progress_tracker.write().await;
            if let Some(progress) = tracker.get_mut(&local_session_id) {
                for file in &mut progress.files {
                    file.status = ProgressStatus::Failed(format!("Handshake failed: {}", status));
                }
            }
            
            anyhow::bail!("Failed to confirm upload: {} - {}", status, body);
        }
        
        let _confirm_resp: ConfirmUploadResponse = confirm_response.json().await?;
        tracing::info!("✅ Three-way handshake complete, starting transfer");

        // Upload each file
        for (file_path, file_info) in files.iter().zip(file_infos.iter()) {
            // Check if cancelled
            {
                let tracker = self.progress_tracker.read().await;
                if let Some(progress) = tracker.get(&local_session_id) {
                    if progress.is_cancelled {
                        tracing::info!("Transfer cancelled by user");
                        return Ok(local_session_id);
                    }
                }
            }

            let file_response = prepare_response.files.iter()
                .find(|f| f.id == file_info.id)
                .ok_or_else(|| anyhow::anyhow!("File response not found"))?;

            match self.upload_file(
                target,
                file_path,
                &server_session_id,  // Use server's session ID for upload
                &file_info.id,
                &file_response.token,
                file_info.size,
                &file_info.file_type,
                &local_session_id,  // Pass local session ID for progress tracking
            ).await {
                Ok(_) => {
                    let mut tracker = self.progress_tracker.write().await;
                    if let Some(progress) = tracker.get_mut(&local_session_id) {
                        progress.mark_file_completed(&file_info.id);
                    }
                }
                Err(e) => {
                    let mut tracker = self.progress_tracker.write().await;
                    if let Some(progress) = tracker.get_mut(&local_session_id) {
                        progress.mark_file_failed(&file_info.id, e.to_string());
                    }
                    return Err(e);
                }
            }
        }

        tracing::info!("All files sent successfully");
        Ok(local_session_id)
    }

    async fn upload_file(
        &self,
        target: &Device,
        file_path: &PathBuf,
        server_session_id: &str,
        file_id: &str,
        token: &str,
        file_size: u64,
        content_type: &str,
        local_session_id: &str,
    ) -> Result<()> {
        tracing::info!("Uploading file: {:?} ({} bytes / {:.2} MB)", 
            file_path, file_size, file_size as f64 / 1_048_576.0);

        // Update status to transferring
        {
            let mut tracker = self.progress_tracker.write().await;
            if let Some(progress) = tracker.get_mut(local_session_id) {
                if let Some(file) = progress.files.iter_mut().find(|f| f.file_id == file_id) {
                    file.status = ProgressStatus::Transferring;
                }
            }
        }

        let url = format!(
            "http://{}:{}/api/localsend/v2/upload?sessionId={}&fileId={}&token={}",
            target.ip, target.port, server_session_id, file_id, token
        );

        tracing::info!("Uploading to URL: {}", url);
        tracing::info!("Server session: {}, File: {}, Token: {}", server_session_id, file_id, token);

        // Open file for streaming
        let file = File::open(file_path).await?;
        let mut reader = tokio::io::BufReader::new(file);
        
        // Create progress tracking channel
        let (tx, rx) = tokio::sync::mpsc::channel::<Result<bytes::Bytes, std::io::Error>>(100);
        
        // Clone values for the streaming task
        let tracker = self.progress_tracker.clone();
        let local_session_id = local_session_id.to_string();
        let file_id = file_id.to_string();
        let chunk_size = 64 * 1024; // 64KB chunks for better performance
        
        // Spawn task to read file and send chunks
        tokio::spawn(async move {
            let mut bytes_sent = 0u64;
            let mut buffer = vec![0u8; chunk_size];
            
            loop {
                match reader.read(&mut buffer).await {
                    Ok(0) => {
                        // EOF reached
                        break;
                    }
                    Ok(n) => {
                        bytes_sent += n as u64;
                        
                        // Update progress
                        {
                            let mut guard = tracker.write().await;
                            if let Some(progress) = guard.get_mut(&local_session_id) {
                                progress.update_file_progress(&file_id, bytes_sent);
                            }
                        }
                        
                        // Send chunk
                        let chunk = bytes::Bytes::copy_from_slice(&buffer[..n]);
                        if tx.send(Ok(chunk)).await.is_err() {
                            break;
                        }
                        
                        // Small delay for UI updates (every 10 chunks = ~640KB)
                        if bytes_sent % (chunk_size as u64 * 10) == 0 {
                            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e)).await;
                        break;
                    }
                }
            }
        });
        
        // Create streaming body
        let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        let body = reqwest::Body::wrap_stream(stream);

        let response = self.client
            .post(&url)
            .header("Content-Type", content_type)
            .header("Content-Length", file_size.to_string())
            .body(body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to upload file: {} - {}", status, body);
        }

        tracing::info!("✅ File uploaded successfully: {:?}", file_path);
        Ok(())
    }

    pub async fn cancel_transfer(&self, session_id: &str) {
        let mut tracker = self.progress_tracker.write().await;
        if let Some(progress) = tracker.get_mut(session_id) {
            progress.cancel();
            tracing::info!("Transfer {} cancelled", session_id);
        }
    }
}
