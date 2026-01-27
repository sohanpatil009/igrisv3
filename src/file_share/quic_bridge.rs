// src/file_share/quic_bridge.rs - QUIC Connection Manager

use quinn::{Endpoint, Connection};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;

use super::discovery::DiscoveredDevice;
use super::quic_crypto::{get_quic_cert_manager, QuicCertManager};

/// Message types for QUIC communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuicMessage {
    Heartbeat,
    FileTransferRequest { 
        filename: String, 
        size: u64, 
        checksum: String,
        transfer_id: String,
    },
    FileTransferAccept { transfer_id: String },
    FileTransferReject { transfer_id: String, reason: String },
    FileChunk { 
        transfer_id: String,
        sequence: u32, 
        data: Vec<u8>,
        is_last: bool,
    },
    FileTransferComplete { transfer_id: String, checksum: String },
    Disconnect { reason: String },
}

impl QuicMessage {
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| format!("Serialize error: {}", e))
    }
    
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(data).map_err(|e| format!("Deserialize error: {}", e))
    }
}

/// Connection state
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Connecting,
    Connected,
    Disconnected,
    Error(String),
}

/// QUIC connection wrapper
pub struct QuicBridgeConnection {
    pub device_id: String,
    pub device_label: String,
    pub connection: Connection,
    pub state: ConnectionState,
    pub connected_at: std::time::Instant,
}

impl QuicBridgeConnection {
    fn new(device_id: String, device_label: String, connection: Connection) -> Self {
        QuicBridgeConnection {
            device_id,
            device_label,
            connection,
            state: ConnectionState::Connected,
            connected_at: std::time::Instant::now(),
        }
    }
    
    /// Send a message over a new bidirectional stream
    pub async fn send_message(&self, message: &QuicMessage) -> Result<(), String> {
        let (mut send, _recv) = self.connection.open_bi().await
            .map_err(|e| format!("Failed to open stream: {}", e))?;
        
        let bytes = message.to_bytes()?;
        
        // Write length prefix (4 bytes)
        let len = bytes.len() as u32;
        send.write_all(&len.to_be_bytes()).await
            .map_err(|e| format!("Failed to send length: {}", e))?;
        
        // Write message
        send.write_all(&bytes).await
            .map_err(|e| format!("Failed to send: {}", e))?;
        
        send.finish()
            .map_err(|e| format!("Failed to finish stream: {}", e))?;
        
        Ok(())
    }
    
    /// Receive messages from incoming streams
    pub async fn receive_message(&self) -> Result<QuicMessage, String> {
        let (_send, mut recv) = self.connection.accept_bi().await
            .map_err(|e| format!("Failed to accept stream: {}", e))?;
        
        // Read length prefix
        let mut len_buf = [0u8; 4];
        recv.read_exact(&mut len_buf).await
            .map_err(|e| format!("Failed to read length: {}", e))?;
        let len = u32::from_be_bytes(len_buf) as usize;
        
        if len > 10 * 1024 * 1024 {
            return Err("Message too large".to_string());
        }
        
        // Read message
        let data = recv.read_to_end(len).await
            .map_err(|e| format!("Failed to read: {}", e))?;
        
        QuicMessage::from_bytes(&data)
    }
    
    /// Check if connection is healthy
    pub fn is_healthy(&self) -> bool {
        self.state == ConnectionState::Connected && 
        self.connection.close_reason().is_none()
    }
    
    /// Disconnect gracefully
    pub fn disconnect(&self, reason: &str) {
        self.connection.close(0u32.into(), reason.as_bytes());
    }
}

/// Events from the QUIC bridge
#[derive(Debug, Clone)]
pub enum QuicBridgeEvent {
    Connected { device_id: String },
    Disconnected { device_id: String, reason: String },
    MessageReceived { device_id: String, message: QuicMessage },
    Error { device_id: String, error: String },
}

/// QUIC Bridge Manager
pub struct QuicBridgeManager {
    pub endpoint: Option<Endpoint>,
    pub connections: HashMap<String, QuicBridgeConnection>,
    event_sender: broadcast::Sender<QuicBridgeEvent>,
    my_device_id: String,
}

impl QuicBridgeManager {
    pub fn new() -> Result<Self, String> {
        let config = super::config::load_config()?;
        let (event_sender, _) = broadcast::channel(100);
        
        Ok(QuicBridgeManager {
            endpoint: None,
            connections: HashMap::new(),
            event_sender,
            my_device_id: config.identity.id,
        })
    }
    
    /// Initialize QUIC endpoint (server + client)
    pub async fn initialize(&mut self, port: u16) -> Result<(), String> {
        let cert_manager_lock = get_quic_cert_manager()?;
        let cert_manager = cert_manager_lock.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let cert_mgr = cert_manager.as_ref()
            .ok_or("Certificate manager not initialized")?;
        
        let server_config = cert_mgr.server_config()?;
        let client_config = QuicCertManager::client_config()?;
        
        let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()
            .map_err(|e| format!("Invalid address: {}", e))?;
        
        let mut endpoint = Endpoint::server(server_config, addr)
            .map_err(|e| format!("Failed to create endpoint: {}", e))?;
        
        endpoint.set_default_client_config(client_config);
        
        println!("[QuicBridge] Endpoint initialized on UDP port {}", port);
        
        self.endpoint = Some(endpoint);
        Ok(())
    }
    
    /// Connect to a discovered device
    pub async fn connect(&mut self, device: &DiscoveredDevice) -> Result<(), String> {
        println!("[QuicBridge] connect() called for device: {} ({})", device.label, &device.id[..8]);
        
        if self.connections.contains_key(&device.id) {
            println!("[QuicBridge] Already connected to {}", device.label);
            return Err("Already connected to this device".to_string());
        }
        
        let endpoint = self.endpoint.as_ref()
            .ok_or("Endpoint not initialized")?;
        
        println!("[QuicBridge] Connecting to {} at {}:{}", 
            device.label, device.ip_address, device.bridge_port);
        
        let addr: SocketAddr = format!("{}:{}", device.ip_address, device.bridge_port).parse()
            .map_err(|e| {
                println!("[QuicBridge] Invalid address format: {}", e);
                format!("Invalid address: {}", e)
            })?;
        
        println!("[QuicBridge] Parsed address: {}", addr);
        
        // Connect with QUIC (TLS handshake automatic!)
        println!("[QuicBridge] Initiating QUIC connection...");
        let connecting = endpoint.connect(addr, "localhost")
            .map_err(|e| {
                println!("[QuicBridge] Failed to initiate connection: {}", e);
                format!("Failed to initiate connection: {}", e)
            })?;
        
        println!("[QuicBridge] Waiting for connection to establish...");
        let connection = connecting.await
            .map_err(|e| {
                println!("[QuicBridge] Connection failed: {}", e);
                format!("Connection failed: {}", e)
            })?;
        
        println!("[QuicBridge] QUIC connection established to {}", device.label);
        
        let quic_conn = QuicBridgeConnection::new(
            device.id.clone(),
            device.label.clone(),
            connection,
        );
        
        self.connections.insert(device.id.clone(), quic_conn);
        
        let _ = self.event_sender.send(QuicBridgeEvent::Connected { 
            device_id: device.id.clone() 
        });
        
        println!("[QuicBridge] Device {} added to connections map", device.label);
        
        Ok(())
    }
    
    /// Accept incoming connections (run in background task)
    pub async fn accept_incoming(&mut self) -> Result<(), String> {
        let endpoint = self.endpoint.as_ref()
            .ok_or("Endpoint not initialized")?;
        
        if let Some(incoming) = endpoint.accept().await {
            let connection = incoming.await
                .map_err(|e| format!("Failed to accept connection: {}", e))?;
            
            let remote_addr = connection.remote_address();
            println!("[QuicBridge] Accepted connection from {}", remote_addr);
            
            // TODO: Get device info from handshake message
            // For now, use temporary ID
            let device_id = format!("incoming_{}", remote_addr.ip());
            let device_label = format!("Device at {}", remote_addr.ip());
            
            let quic_conn = QuicBridgeConnection::new(
                device_id.clone(),
                device_label,
                connection,
            );
            
            self.connections.insert(device_id.clone(), quic_conn);
            
            let _ = self.event_sender.send(QuicBridgeEvent::Connected { 
                device_id 
            });
        }
        
        Ok(())
    }
    
    /// Disconnect from a device
    pub fn disconnect(&mut self, device_id: &str, reason: &str) -> Result<(), String> {
        if let Some(conn) = self.connections.remove(device_id) {
            conn.disconnect(reason);
            
            let _ = self.event_sender.send(QuicBridgeEvent::Disconnected { 
                device_id: device_id.to_string(),
                reason: reason.to_string(),
            });
            
            println!("[QuicBridge] Disconnected from {}", conn.device_label);
        }
        Ok(())
    }
    
    /// Send a message to a connected device
    pub async fn send_message(&self, device_id: &str, message: QuicMessage) -> Result<(), String> {
        let conn = self.connections.get(device_id)
            .ok_or("Device not connected")?;
        
        conn.send_message(&message).await
    }
    
    /// Check if connected to a device
    pub fn is_connected(&self, device_id: &str) -> bool {
        self.connections.get(device_id)
            .map(|c| c.is_healthy())
            .unwrap_or(false)
    }
    
    /// Get all connected device IDs
    pub fn get_connected_devices(&self) -> Vec<String> {
        self.connections.keys().cloned().collect()
    }
    
    /// Subscribe to bridge events
    pub fn subscribe(&self) -> broadcast::Receiver<QuicBridgeEvent> {
        self.event_sender.subscribe()
    }
}

// Global QUIC bridge manager
static QUIC_BRIDGE_MANAGER: Lazy<Arc<Mutex<Option<QuicBridgeManager>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(None)));

pub fn get_quic_bridge_manager() -> Result<Arc<Mutex<Option<QuicBridgeManager>>>, String> {
    let mut manager = QUIC_BRIDGE_MANAGER.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    if manager.is_none() {
        *manager = Some(QuicBridgeManager::new()?);
    }
    
    Ok(QUIC_BRIDGE_MANAGER.clone())
}

/// Initialize QUIC bridge
pub async fn initialize_quic_bridge(port: u16) -> Result<(), String> {
    let manager_lock = get_quic_bridge_manager()?;
    let mut manager = manager_lock.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    if let Some(ref mut mgr) = *manager {
        mgr.initialize(port).await?;
    }
    
    Ok(())
}

/// Connect to a device via QUIC
pub async fn connect_to_device_quic(device: &DiscoveredDevice) -> Result<(), String> {
    let manager_lock = get_quic_bridge_manager()?;
    let mut manager = manager_lock.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    if let Some(ref mut mgr) = *manager {
        mgr.connect(device).await
    } else {
        Err("QUIC bridge not initialized".to_string())
    }
}

/// Send message to device via QUIC
pub async fn send_to_device_quic(device_id: &str, message: QuicMessage) -> Result<(), String> {
    let manager_lock = get_quic_bridge_manager()?;
    let manager = manager_lock.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    if let Some(ref mgr) = *manager {
        mgr.send_message(device_id, message).await
    } else {
        Err("QUIC bridge not initialized".to_string())
    }
}

/// Check if connected to device via QUIC
pub fn is_connected_to_quic(device_id: &str) -> Result<bool, String> {
    let manager_lock = get_quic_bridge_manager()?;
    let manager = manager_lock.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    if let Some(ref mgr) = *manager {
        Ok(mgr.is_connected(device_id))
    } else {
        Ok(false)
    }
}
