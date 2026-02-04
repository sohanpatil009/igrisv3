// Transfer orchestrator - manages both sending and receiving

use super::{FileReceiver, FileSender, TransferProgress};
use crate::file_share::discovery::Device;
use crate::file_share::protocol::{DeviceInfo, PrepareUploadRequest};
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct TransferOrchestrator {
    sender: Arc<FileSender>,
    receiver: Arc<FileReceiver>,
    sessions: Arc<RwLock<HashMap<String, SessionType>>>,
}

#[derive(Debug, Clone)]
enum SessionType {
    Sending,
    Receiving,
}

impl TransferOrchestrator {
    pub fn new() -> Self {
        // These will be properly initialized when FileShareManager is created
        let device_info = DeviceInfo::new(
            "IGRIS".to_string(),
            "temp".to_string(),
            53317,
        );
        
        let sender = Arc::new(FileSender::new(device_info));
        let receiver = Arc::new(FileReceiver::new(PathBuf::from("./downloads")));

        Self {
            sender,
            receiver,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_config(device_info: DeviceInfo, download_dir: PathBuf) -> Self {
        let sender = Arc::new(FileSender::new(device_info));
        let receiver = Arc::new(FileReceiver::new(download_dir));

        Self {
            sender,
            receiver,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Send files to a device
    pub async fn send_files(&self, device: Device, file_paths: Vec<String>) -> Result<String> {
        let session_id = self.sender.send_files(device, file_paths).await?;
        self.sessions
            .write()
            .await
            .insert(session_id.clone(), SessionType::Sending);
        Ok(session_id)
    }

    /// Handle incoming prepare request
    pub async fn handle_prepare_request(
        &self,
        session_id: String,
        request: PrepareUploadRequest,
    ) -> Result<HashMap<String, String>> {
        let tokens = self.receiver.handle_prepare_request(session_id.clone(), request).await?;
        self.sessions
            .write()
            .await
            .insert(session_id, SessionType::Receiving);
        Ok(tokens)
    }

    /// Receive file data
    pub async fn receive_file(
        &self,
        session_id: &str,
        file_id: &str,
        data: Vec<u8>,
    ) -> Result<()> {
        self.receiver.receive_file(session_id, file_id, data).await
    }

    /// Get transfer progress
    pub fn get_progress(&self, session_id: &str) -> Option<TransferProgress> {
        // Try both sender and receiver
        if let Some(progress) = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.sender.get_progress(session_id))
        }) {
            return Some(progress);
        }

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.receiver.get_progress(session_id))
        })
    }

    /// Cancel transfer
    pub async fn cancel_transfer(&self, session_id: &str) -> Result<()> {
        let session_type = self.sessions.read().await.get(session_id).cloned();

        match session_type {
            Some(SessionType::Sending) => self.sender.cancel_transfer(session_id).await,
            Some(SessionType::Receiving) => self.receiver.cancel_session(session_id).await,
            None => Ok(()),
        }
    }

    /// Get all active sessions
    pub async fn get_active_sessions(&self) -> Vec<String> {
        self.sessions.read().await.keys().cloned().collect()
    }
}

impl Default for TransferOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}
