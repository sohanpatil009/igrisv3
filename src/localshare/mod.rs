// LocalShare integration module for IGRIS
// Based on localshare-desktop implementation

pub mod models;
pub mod network;

pub use models::*;
pub use network::*;

use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;

/// LocalShare manager for IGRIS integration
pub struct LocalShareManager {
    discovery: Arc<RwLock<DiscoveryService>>,
    server_handle: Option<tokio::task::JoinHandle<()>>,
    port: u16,
}

impl LocalShareManager {
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
                eprintln!("[LocalShare] Server error: {}", e);
            }
        });
        
        self.server_handle = Some(handle);
        
        tracing::info!("[LocalShare] Started on port {}", self.port);
        Ok(())
    }
    
    pub async fn stop(&mut self) {
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
        }
        tracing::info!("[LocalShare] Stopped");
    }
    
    pub async fn get_devices(&self) -> Vec<Device> {
        self.discovery.read().await.get_devices().await
    }
    
    pub fn get_discovery_service(&self) -> Arc<RwLock<DiscoveryService>> {
        Arc::clone(&self.discovery)
    }
}

impl Drop for LocalShareManager {
    fn drop(&mut self) {
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
        }
    }
}
