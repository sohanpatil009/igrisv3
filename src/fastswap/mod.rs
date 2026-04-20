// FastSwap integration module for IGRIS
// Based on localshare-desktop implementation

pub mod models;
pub mod network;

pub use models::*;
pub use network::*;

use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;

// Global progress tracker for UI access
static GLOBAL_PROGRESS_TRACKER: once_cell::sync::Lazy<ProgressTracker> =
    once_cell::sync::Lazy::new(|| models::progress::create_progress_tracker());

// Global pending transfers (for receiver approval)
#[derive(Clone, Debug)]
pub struct PendingTransfer {
    pub session_id: String,
    pub sender_name: String,
    pub sender_device: String,
    pub file_count: usize,
    pub total_size: u64,
    pub files: Vec<String>,
}

static PENDING_TRANSFERS: once_cell::sync::Lazy<Arc<RwLock<Vec<PendingTransfer>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(Vec::new())));

static APPROVED_SESSIONS: once_cell::sync::Lazy<Arc<RwLock<Vec<String>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(Vec::new())));

/// FastSwap manager for IGRIS integration
pub struct FastSwapManager {
    discovery: Arc<RwLock<DiscoveryService>>,
    server_handle: Option<tokio::task::JoinHandle<()>>,
    port: u16,
}

impl FastSwapManager {
    pub fn new(port: u16) -> Self {
        let discovery = Arc::new(RwLock::new(DiscoveryService::new()));
        
        Self {
            discovery,
            server_handle: None,
            port,
        }
    }
    
    pub async fn start(&mut self, local_device: Device) -> Result<()> {
        // Start HTTP server
        let port = self.port;
        let handle = tokio::spawn(async move {
            if let Err(e) = network::start_server(port, local_device).await {
                eprintln!("[FastSwap] Server error: {}", e);
            }
        });
        
        self.server_handle = Some(handle);
        
        tracing::info!("[FastSwap] Started on port {}", self.port);
        Ok(())
    }
    
    pub async fn stop(&mut self) {
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
        }
        tracing::info!("[FastSwap] Stopped");
    }
    
    pub async fn get_devices(&self) -> Vec<Device> {
        self.discovery.read().await.get_devices().await
    }
    
    pub fn get_discovery_service(&self) -> Arc<RwLock<DiscoveryService>> {
        Arc::clone(&self.discovery)
    }
}

/// Get global progress tracker for UI access
pub fn get_progress_tracker() -> ProgressTracker {
    Arc::clone(&GLOBAL_PROGRESS_TRACKER)
}

/// Add pending transfer for approval
pub async fn add_pending_transfer(transfer: PendingTransfer) {
    let mut pending = PENDING_TRANSFERS.write().await;
    pending.push(transfer);
}

/// Get all pending transfers
pub async fn get_pending_transfers() -> Vec<PendingTransfer> {
    PENDING_TRANSFERS.read().await.clone()
}

/// Approve a transfer
pub async fn approve_transfer(session_id: &str) {
    let mut approved = APPROVED_SESSIONS.write().await;
    approved.push(session_id.to_string());
    
    // Remove from pending
    let mut pending = PENDING_TRANSFERS.write().await;
    pending.retain(|t| t.session_id != session_id);
}

/// Deny a transfer
pub async fn deny_transfer(session_id: &str) {
    // Just remove from pending
    let mut pending = PENDING_TRANSFERS.write().await;
    pending.retain(|t| t.session_id != session_id);
}

/// Check if transfer is approved
pub async fn is_transfer_approved(session_id: &str) -> bool {
    let approved = APPROVED_SESSIONS.read().await;
    approved.contains(&session_id.to_string())
}

impl Drop for FastSwapManager {
    fn drop(&mut self) {
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
        }
    }
}
