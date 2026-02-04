// File receiver implementation

use super::{FileIntegrity, TransferProgress, TransferStatus};
use crate::file_share::protocol::{FileInfo, PrepareUploadRequest};
use anyhow::Result;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct FileReceiver {
    download_dir: PathBuf,
    progress: Arc<RwLock<HashMap<String, TransferProgress>>>,
    pending_sessions: Arc<RwLock<HashMap<String, PrepareUploadRequest>>>,
}

impl FileReceiver {
    pub fn new(download_dir: PathBuf) -> Self {
        Self {
            download_dir,
            progress: Arc::new(RwLock::new(HashMap::new())),
            pending_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Handle prepare upload request
    pub async fn handle_prepare_request(
        &self,
        session_id: String,
        request: PrepareUploadRequest,
    ) -> Result<HashMap<String, String>> {
        // Calculate total size
        let total_size: u64 = request.files.values().map(|f| f.size).sum();
        let files_count = request.files.len();

        // Create progress
        let progress = TransferProgress::new(session_id.clone(), total_size, files_count);
        self.progress.write().await.insert(session_id.clone(), progress);

        // Store pending session
        self.pending_sessions
            .write()
            .await
            .insert(session_id.clone(), request.clone());

        // Generate tokens for each file
        let mut tokens = HashMap::new();
        for file_id in request.files.keys() {
            let token = format!("token_{}", uuid::Uuid::new_v4());
            tokens.insert(file_id.clone(), token);
        }

        Ok(tokens)
    }

    /// Receive file data
    pub async fn receive_file(
        &self,
        session_id: &str,
        file_id: &str,
        data: Vec<u8>,
    ) -> Result<()> {
        // Get file info from pending session
        let file_info = {
            let sessions = self.pending_sessions.read().await;
            let session = sessions
                .get(session_id)
                .ok_or_else(|| anyhow::anyhow!("Session not found"))?;
            session
                .files
                .get(file_id)
                .ok_or_else(|| anyhow::anyhow!("File not found"))?
                .clone()
        };

        // Update progress
        {
            let mut progress_map = self.progress.write().await;
            if let Some(progress) = progress_map.get_mut(session_id) {
                progress.status = TransferStatus::Transferring;
                progress.current_file = Some(file_info.file_name.clone());
            }
        }

        // Save file
        let file_path = self.download_dir.join(&file_info.file_name);
        let mut file = File::create(&file_path)?;
        file.write_all(&data)?;

        // Verify checksum if provided
        if let Some(expected_hash) = &file_info.sha256 {
            let verified = FileIntegrity::verify_hash(&file_path, expected_hash)?;
            if !verified {
                std::fs::remove_file(&file_path)?;
                anyhow::bail!("Checksum verification failed");
            }
        }

        // Update progress
        {
            let mut progress_map = self.progress.write().await;
            if let Some(progress) = progress_map.get_mut(session_id) {
                progress.transferred_bytes += file_info.size;
                progress.files_completed += 1;

                // Check if all files completed
                if progress.files_completed == progress.files_total {
                    progress.status = TransferStatus::Completed;
                    progress.current_file = None;
                }
            }
        }

        Ok(())
    }

    /// Cancel a session
    pub async fn cancel_session(&self, session_id: &str) -> Result<()> {
        self.pending_sessions.write().await.remove(session_id);
        
        let mut progress_map = self.progress.write().await;
        if let Some(progress) = progress_map.get_mut(session_id) {
            progress.status = TransferStatus::Cancelled;
        }

        Ok(())
    }

    pub async fn get_progress(&self, session_id: &str) -> Option<TransferProgress> {
        self.progress.read().await.get(session_id).cloned()
    }
}
