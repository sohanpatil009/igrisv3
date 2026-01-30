// src/file_share/mod.rs
// Cross-platform file sharing ecosystem

pub mod discovery;
pub mod transfer;
pub mod crypto;
pub mod trust;
pub mod bridge;
pub mod device;
pub mod protocol;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use serde::{Serialize, Deserialize};

pub use discovery::*;
pub use transfer::*;
pub use crypto::*;
pub use trust::*;
pub use bridge::*;
pub use device::*;
pub use protocol::*;

/// Main file sharing manager
#[derive(Clone)]
pub struct FileShareManager {
    pub discovery: Arc<DiscoveryService>,
    pub transfer: Arc<TransferManager>,
    pub trust: Arc<TrustManager>,
    pub bridge: Arc<BridgeService>,
    pub device_info: DeviceInfo,
    pub event_tx: mpsc::UnboundedSender<FileShareEvent>,
}

/// File sharing events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileShareEvent {
    DeviceDiscovered(DeviceInfo),
    DeviceLost(String),
    ConnectionRequest(String, SocketAddr),
    TransferStarted(String, String),
    TransferProgress(String, u64, u64),
    TransferCompleted(String),
    TransferFailed(String, String),
    TrustRequest(String, String),
    TrustEstablished(String),
}

impl FileShareManager {
    /// Create new file share manager
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (event_tx, _) = mpsc::unbounded_channel();
        
        let device_info = DeviceInfo::current().await?;
        let trust = Arc::new(TrustManager::new().await?);
        let crypto = Arc::new(CryptoManager::new().await?);
        let discovery = Arc::new(DiscoveryService::new(device_info.clone(), event_tx.clone()).await?);
        let transfer = Arc::new(TransferManager::new(crypto.clone(), trust.clone(), event_tx.clone()).await?);
        let bridge = Arc::new(BridgeService::new(device_info.clone(), event_tx.clone()).await?);

        Ok(Self {
            discovery,
            transfer,
            trust,
            bridge,
            device_info,
            event_tx,
        })
    }

    /// Start file sharing services
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Start discovery service
        self.discovery.start().await?;
        
        // Start transfer service
        self.transfer.start().await?;
        
        // Start bridge service for cross-network connections
        self.bridge.start().await?;
        
        println!("🚀 File sharing services started");
        println!("📱 Device: {} ({})", self.device_info.name, self.device_info.id);
        println!("🔗 Bridge Code: {}", self.bridge.get_code().await);
        
        Ok(())
    }

    /// Stop file sharing services
    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.discovery.stop().await?;
        self.transfer.stop().await?;
        self.bridge.stop().await?;
        
        println!("🛑 File sharing services stopped");
        Ok(())
    }

    /// Get discovered devices
    pub async fn get_devices(&self) -> Vec<DeviceInfo> {
        self.discovery.get_devices().await
    }

    /// Connect to device by ID
    pub async fn connect_device(&self, device_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(device) = self.discovery.get_device(device_id).await {
            self.transfer.connect_device(device).await?;
            Ok(())
        } else {
            Err(format!("Device {} not found", device_id).into())
        }
    }

    /// Connect using bridge code
    pub async fn connect_by_code(&self, code: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.bridge.connect_by_code(code).await
    }

    /// Send file to device
    pub async fn send_file(&self, device_id: &str, file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.transfer.send_file(device_id, file_path).await
    }

    /// Get transfer progress
    pub async fn get_transfer_progress(&self, transfer_id: &str) -> Option<TransferProgress> {
        self.transfer.get_progress(transfer_id).await
    }

    /// Get trusted devices
    pub async fn get_trusted_devices(&self) -> Vec<DeviceInfo> {
        self.trust.get_trusted_devices().await
    }

    /// Get current bridge code
    pub async fn get_bridge_code(&self) -> String {
        self.bridge.get_code().await
    }

    /// Subscribe to events
    pub fn subscribe_events(&self) -> mpsc::UnboundedReceiver<FileShareEvent> {
        let (tx, rx) = mpsc::unbounded_channel();
        // In a real implementation, you'd register this receiver
        // For now, return empty receiver
        rx
    }
}

/// Configuration for file sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileShareConfig {
    pub discovery_port: u16,
    pub transfer_port: u16,
    pub bridge_port: u16,
    pub max_file_size: u64,
    pub chunk_size: usize,
    pub timeout_seconds: u64,
    pub trust_duration_days: u32,
}

impl Default for FileShareConfig {
    fn default() -> Self {
        Self {
            discovery_port: 45678,
            transfer_port: 45679,
            bridge_port: 45680,
            max_file_size: 10 * 1024 * 1024 * 1024, // 10GB
            chunk_size: 64 * 1024, // 64KB
            timeout_seconds: 300, // 5 minutes
            trust_duration_days: 30,
        }
    }
}