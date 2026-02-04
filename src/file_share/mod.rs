// File Share Module - LocalSend Protocol Implementation
// Based on LocalSend Protocol v2.1

pub mod api;
pub mod connection;
pub mod crypto;
pub mod discovery;
pub mod protocol;
pub mod transfer;
pub mod trust;
pub mod firewall;

use std::sync::Arc;
use tokio::sync::RwLock;

pub use api::{FileShareApi, FileShareCommand, FileShareEvent};
pub use discovery::{Device, DeviceRegistry, MdnsDiscovery};
pub use protocol::{FileInfo, PrepareUploadRequest, PrepareUploadResponse, SessionInfo};
pub use transfer::{TransferOrchestrator, TransferProgress, TransferStatus};

/// Main File Share Manager
pub struct FileShareManager {
    discovery: Arc<RwLock<MdnsDiscovery>>,
    registry: Arc<RwLock<DeviceRegistry>>,
    orchestrator: Arc<TransferOrchestrator>,
    api: Arc<RwLock<FileShareApi>>,
}

impl FileShareManager {
    pub async fn new(device_name: String, port: u16) -> anyhow::Result<Self> {
        let registry = Arc::new(RwLock::new(DeviceRegistry::new()));
        let discovery = Arc::new(RwLock::new(MdnsDiscovery::new(device_name.clone(), port, registry.clone()).await?));
        let orchestrator = Arc::new(TransferOrchestrator::new());
        let api = Arc::new(RwLock::new(FileShareApi::new(port, orchestrator.clone()).await?));

        Ok(Self {
            discovery,
            registry,
            orchestrator,
            api,
        })
    }

    /// Start discovery and HTTP server
    pub async fn start(&self) -> anyhow::Result<()> {
        self.discovery.write().await.start_broadcasting().await?;
        self.api.write().await.start_server().await?;
        Ok(())
    }

    /// Stop all services
    pub async fn stop(&self) -> anyhow::Result<()> {
        self.discovery.write().await.stop_broadcasting().await?;
        self.api.write().await.stop_server().await?;
        Ok(())
    }

    /// Get list of discovered devices
    pub async fn get_devices(&self) -> Vec<Device> {
        self.registry.read().await.get_all_devices()
    }

    /// Send files to a device
    pub async fn send_files(&self, device_id: &str, file_paths: Vec<String>) -> anyhow::Result<String> {
        let device = self.registry.read().await.get_device(device_id).ok_or_else(|| {
            anyhow::anyhow!("Device not found")
        })?;

        self.orchestrator.send_files(device, file_paths).await
    }

    /// Get transfer progress
    pub fn get_progress(&self, session_id: &str) -> Option<TransferProgress> {
        self.orchestrator.get_progress(session_id)
    }

    /// Cancel transfer
    pub async fn cancel_transfer(&self, session_id: &str) -> anyhow::Result<()> {
        self.orchestrator.cancel_transfer(session_id).await
    }
}
