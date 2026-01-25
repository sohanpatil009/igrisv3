// src/file_share/bridge.rs - Secure TLS Bridge for Device Communication

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;
use rustls::{ClientConfig, ServerConfig, ClientConnection, ServerConnection};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};
use std::sync::Arc as StdArc;

use super::config::load_config;
use super::crypto::{initialize_certificate, get_certificate_manager};
use super::trust::{get_trust_manager, is_device_trusted};
use super::discovery::DiscoveredDevice;

// Bridge constants
const BRIDGE_PORT: u16 = 45679;
const HEARTBEAT_INTERVAL_SECS: u64 = 5;
const RECONNECT_TIMEOUT_SECS: u64 = 30;
const READ_TIMEOUT_MS: u64 = 5000;

/// Message types for bridge communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeMessage {
    /// Heartbeat to keep connection alive
    Heartbeat,
    /// Verification request with code
    VerificationRequest { code: String, device_id: String },
    /// Verification response
    VerificationResponse { accepted: bool, device_id: String },
    /// File transfer request
    FileTransferRequest { 
        filename: String, 
        size: u64, 
        checksum: String,
        transfer_id: String,
    },
    /// Accept file transfer
    FileTransferAccept { transfer_id: String },
    /// Reject file transfer
    FileTransferReject { transfer_id: String, reason: String },
    /// File data chunk
    FileChunk { 
        transfer_id: String,
        sequence: u32, 
        data: Vec<u8>,
        is_last: bool,
    },
    /// File transfer complete
    FileTransferComplete { transfer_id: String, checksum: String },
    /// Disconnect gracefully
    Disconnect { reason: String },
    /// Custom message for extensibility
    Custom { msg_type: String, payload: String },
}

impl BridgeMessage {
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        let json = serde_json::to_vec(self)
            .map_err(|e| format!("Serialize error: {}", e))?;
        
        // Prepend length as 4 bytes
        let len = json.len() as u32;
        let mut bytes = len.to_be_bytes().to_vec();
        bytes.extend(json);
        
        Ok(bytes)
    }
    
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(data)
            .map_err(|e| format!("Deserialize error: {}", e))
    }
}

/// Connection state
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Connecting,
    Connected,
    Verified,
    Disconnected,
    Error(String),
}

/// A bridge connection to another device
pub struct BridgeConnection {
    pub device_id: String,
    pub device_label: String,
    pub remote_addr: SocketAddr,
    pub state: ConnectionState,
    pub connected_at: Instant,
    pub last_heartbeat: Instant,
    stream: Option<TcpStream>,
    is_outgoing: bool,
}

impl BridgeConnection {
    fn new(device_id: String, device_label: String, remote_addr: SocketAddr, is_outgoing: bool) -> Self {
        let now = Instant::now();
        BridgeConnection {
            device_id,
            device_label,
            remote_addr,
            state: ConnectionState::Connecting,
            connected_at: now,
            last_heartbeat: now,
            stream: None,
            is_outgoing,
        }
    }
    
    /// Send a message over the connection
    pub fn send(&mut self, message: &BridgeMessage) -> Result<(), String> {
        let stream = self.stream.as_mut()
            .ok_or("Connection not established")?;
        
        let bytes = message.to_bytes()?;
        stream.write_all(&bytes)
            .map_err(|e| format!("Send error: {}", e))?;
        stream.flush()
            .map_err(|e| format!("Flush error: {}", e))?;
        
        Ok(())
    }
    
    /// Receive a message from the connection
    pub fn receive(&mut self) -> Result<Option<BridgeMessage>, String> {
        let stream = self.stream.as_mut()
            .ok_or("Connection not established")?;
        
        // Read length prefix (4 bytes)
        let mut len_buf = [0u8; 4];
        match stream.read_exact(&mut len_buf) {
            Ok(_) => {}
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                return Ok(None);
            }
            Err(e) => return Err(format!("Read length error: {}", e)),
        }
        
        let len = u32::from_be_bytes(len_buf) as usize;
        if len > 10 * 1024 * 1024 { // 10MB max
            return Err("Message too large".to_string());
        }
        
        // Read message body
        let mut body = vec![0u8; len];
        stream.read_exact(&mut body)
            .map_err(|e| format!("Read body error: {}", e))?;
        
        let message = BridgeMessage::from_bytes(&body)?;
        self.last_heartbeat = Instant::now();
        
        Ok(Some(message))
    }
    
    /// Send heartbeat
    pub fn send_heartbeat(&mut self) -> Result<(), String> {
        self.send(&BridgeMessage::Heartbeat)
    }
    
    /// Check if connection is healthy
    pub fn is_healthy(&self) -> bool {
        self.state == ConnectionState::Connected || self.state == ConnectionState::Verified
    }
    
    /// Check if heartbeat is overdue
    pub fn is_heartbeat_overdue(&self) -> bool {
        self.last_heartbeat.elapsed().as_secs() > HEARTBEAT_INTERVAL_SECS * 3
    }
    
    /// Disconnect gracefully
    pub fn disconnect(&mut self, reason: &str) {
        // Send disconnect message first
        let disconnect_msg = BridgeMessage::Disconnect { 
            reason: reason.to_string() 
        };
        if let Ok(bytes) = disconnect_msg.to_bytes() {
            if let Some(ref mut stream) = self.stream {
                let _ = stream.write_all(&bytes);
                let _ = stream.flush();
                let _ = stream.shutdown(std::net::Shutdown::Both);
            }
        }
        self.stream = None;
        self.state = ConnectionState::Disconnected;
    }
}

/// Events from the bridge
#[derive(Debug, Clone)]
pub enum BridgeEvent {
    Connected { device_id: String },
    Disconnected { device_id: String, reason: String },
    VerificationRequest { device_id: String, code: String },
    VerificationResult { device_id: String, accepted: bool },
    MessageReceived { device_id: String, message: BridgeMessage },
    Error { device_id: String, error: String },
}

/// Bridge manager handles all connections
pub struct BridgeManager {
    connections: HashMap<String, BridgeConnection>,
    running: bool,
    event_sender: broadcast::Sender<BridgeEvent>,
    my_device_id: String,
}

impl BridgeManager {
    pub fn new() -> Result<Self, String> {
        let config = load_config()?;
        let (event_sender, _) = broadcast::channel(100);
        
        Ok(BridgeManager {
            connections: HashMap::new(),
            running: false,
            event_sender,
            my_device_id: config.identity.id,
        })
    }
    
    /// Subscribe to bridge events
    pub fn subscribe(&self) -> broadcast::Receiver<BridgeEvent> {
        self.event_sender.subscribe()
    }
    
    /// Connect to a discovered device
    pub fn connect(&mut self, device: &DiscoveredDevice) -> Result<(), String> {
        if self.connections.contains_key(&device.id) {
            return Err("Already connected to this device".to_string());
        }
        
        println!("[Bridge] Connecting to {} at {}:{}", 
            device.label, device.ip_address, device.bridge_port);
        
        let addr = SocketAddr::new(device.ip_address, device.bridge_port);
        
        // Create TCP connection
        let stream = TcpStream::connect_timeout(&addr, Duration::from_secs(10))
            .map_err(|e| format!("Connection failed: {}", e))?;
        
        stream.set_read_timeout(Some(Duration::from_millis(READ_TIMEOUT_MS)))
            .map_err(|e| format!("Set timeout failed: {}", e))?;
        
        stream.set_nodelay(true)
            .map_err(|e| format!("Set nodelay failed: {}", e))?;
        
        let mut conn = BridgeConnection::new(
            device.id.clone(),
            device.label.clone(),
            addr,
            true,
        );
        conn.stream = Some(stream);
        conn.state = ConnectionState::Connected;
        
        self.connections.insert(device.id.clone(), conn);
        
        let _ = self.event_sender.send(BridgeEvent::Connected { 
            device_id: device.id.clone() 
        });
        
        println!("[Bridge] Connected to {}", device.label);
        
        Ok(())
    }
    
    /// Disconnect from a device
    pub fn disconnect(&mut self, device_id: &str) -> Result<(), String> {
        if let Some(mut conn) = self.connections.remove(device_id) {
            conn.disconnect("User requested disconnect");
            
            let _ = self.event_sender.send(BridgeEvent::Disconnected { 
                device_id: device_id.to_string(),
                reason: "User requested".to_string(),
            });
            
            println!("[Bridge] Disconnected from {}", conn.device_label);
        }
        Ok(())
    }
    
    /// Send a message to a connected device
    pub fn send_message(&mut self, device_id: &str, message: BridgeMessage) -> Result<(), String> {
        let conn = self.connections.get_mut(device_id)
            .ok_or("Device not connected")?;
        
        conn.send(&message)
    }
    
    /// Send verification request
    pub fn send_verification(&mut self, device_id: &str, code: &str) -> Result<(), String> {
        let message = BridgeMessage::VerificationRequest {
            code: code.to_string(),
            device_id: self.my_device_id.clone(),
        };
        self.send_message(device_id, message)
    }
    
    /// Send verification response
    pub fn send_verification_response(&mut self, device_id: &str, accepted: bool) -> Result<(), String> {
        let message = BridgeMessage::VerificationResponse {
            accepted,
            device_id: self.my_device_id.clone(),
        };
        self.send_message(device_id, message)
    }
    
    /// Get connection state for a device
    pub fn get_connection_state(&self, device_id: &str) -> Option<ConnectionState> {
        self.connections.get(device_id).map(|c| c.state.clone())
    }
    
    /// Get all connected device IDs
    pub fn get_connected_devices(&self) -> Vec<String> {
        self.connections.keys().cloned().collect()
    }
    
    /// Check if connected to a device
    pub fn is_connected(&self, device_id: &str) -> bool {
        self.connections.get(device_id)
            .map(|c| c.is_healthy())
            .unwrap_or(false)
    }
    
    /// Process incoming messages for all connections
    pub fn poll_messages(&mut self) -> Vec<(String, BridgeMessage)> {
        let mut messages = Vec::new();
        let mut disconnected = Vec::new();
        
        for (device_id, conn) in self.connections.iter_mut() {
            // Check heartbeat
            if conn.is_heartbeat_overdue() {
                disconnected.push(device_id.clone());
                continue;
            }
            
            // Try to receive message
            match conn.receive() {
                Ok(Some(msg)) => {
                    messages.push((device_id.clone(), msg));
                }
                Ok(None) => {
                    // No message available
                }
                Err(e) => {
                    println!("[Bridge] Receive error from {}: {}", conn.device_label, e);
                    disconnected.push(device_id.clone());
                }
            }
        }
        
        // Remove disconnected
        for device_id in disconnected {
            if let Some(mut conn) = self.connections.remove(&device_id) {
                conn.state = ConnectionState::Disconnected;
                let _ = self.event_sender.send(BridgeEvent::Disconnected {
                    device_id: device_id.clone(),
                    reason: "Connection lost".to_string(),
                });
            }
        }
        
        messages
    }
    
    /// Send heartbeats to all connections
    pub fn send_heartbeats(&mut self) {
        for conn in self.connections.values_mut() {
            if conn.is_healthy() {
                let _ = conn.send_heartbeat();
            }
        }
    }
}

// Global bridge manager
static BRIDGE_MANAGER: Lazy<Arc<Mutex<Option<BridgeManager>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(None))
});

/// Get the bridge manager
pub fn get_bridge_manager() -> Result<Arc<Mutex<Option<BridgeManager>>>, String> {
    let mut manager = BRIDGE_MANAGER.lock().map_err(|e| format!("Lock error: {}", e))?;
    if manager.is_none() {
        *manager = Some(BridgeManager::new()?);
    }
    Ok(BRIDGE_MANAGER.clone())
}

/// Bridge server for accepting incoming connections
pub struct BridgeServer {
    port: u16,
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl BridgeServer {
    pub fn new(port: u16) -> Self {
        BridgeServer {
            port,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
    
    /// Start the bridge server to accept incoming connections
    pub async fn start(
        &self,
        connection_coordinator: Arc<super::connection::ConnectionCoordinator>,
    ) -> Result<(), String> {
        use tokio::net::TcpListener;
        use tokio_rustls::TlsAcceptor;
        use rustls::ServerConfig;
        use std::sync::Arc as StdArc;
        
        // Get certificate for TLS
        let cert_manager = get_certificate_manager();
        let (cert_pem, key_pem) = {
            let cert_manager_lock = cert_manager.lock()
                .map_err(|e| format!("Lock error: {}", e))?;
            
            let cert = cert_manager_lock.get_certificate()
                .ok_or("Certificate not initialized")?;
            
            (cert.cert_pem.clone(), cert.key_pem.clone())
        };
        
        // Parse certificate and key
        let cert_der = rustls_pemfile::certs(&mut cert_pem.as_bytes())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to parse certificate: {}", e))?;
        
        let key_der = rustls_pemfile::private_key(&mut key_pem.as_bytes())
            .map_err(|e| format!("Failed to parse private key: {}", e))?
            .ok_or("No private key found")?;
        
        // Create TLS server config
        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_der, key_der)
            .map_err(|e| format!("TLS config error: {}", e))?;
        
        let acceptor = TlsAcceptor::from(StdArc::new(config));
        
        // Bind to port
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr).await
            .map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;
        
        println!("[BridgeServer] Listening on {}", addr);
        
        self.running.store(true, std::sync::atomic::Ordering::SeqCst);
        let running = self.running.clone();
        
        // Accept connections loop
        while running.load(std::sync::atomic::Ordering::SeqCst) {
            match listener.accept().await {
                Ok((stream, peer_addr)) => {
                    println!("[BridgeServer] Incoming connection from {}", peer_addr);
                    
                    let acceptor = acceptor.clone();
                    let coordinator = connection_coordinator.clone();
                    
                    // Spawn task to handle connection
                    // We need to use spawn_blocking because ConnectionCoordinator uses std::sync::Mutex
                    tokio::task::spawn(async move {
                        // Run the handler which internally manages its own blocking operations
                        if let Err(e) = handle_incoming_connection_wrapper(stream, acceptor, coordinator).await {
                            eprintln!("[BridgeServer] Connection error from {}: {}", peer_addr, e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("[BridgeServer] Accept error: {}", e);
                }
            }
        }
        
        println!("[BridgeServer] Server stopped");
        Ok(())
    }
    
    /// Stop the bridge server
    pub fn stop(&self) {
        self.running.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

/// Wrapper for handle_incoming_connection that ensures Send safety
async fn handle_incoming_connection_wrapper(
    stream: tokio::net::TcpStream,
    acceptor: tokio_rustls::TlsAcceptor,
    connection_coordinator: Arc<super::connection::ConnectionCoordinator>,
) -> Result<(), String> {
    // Perform TLS handshake
    let mut tls_stream = acceptor.accept(stream).await
        .map_err(|e| format!("TLS handshake failed: {}", e))?;
    
    println!("[BridgeServer] TLS handshake complete");
    
    // Receive handshake message
    let handshake_msg = super::handshake::receive_handshake_server(&mut tls_stream).await
        .map_err(|e| format!("Failed to receive handshake: {}", e))?;
    
    println!("[BridgeServer] Received handshake message");
    
    // Verify certificate fingerprint for trusted devices
    let verification_result = verify_trusted_device_certificate(&handshake_msg);
    
    if let Err(e) = verification_result {
        let error_response = super::handshake::HandshakeMessage::error(e.clone());
        super::handshake::send_handshake_server(&mut tls_stream, &error_response).await
            .map_err(|e| format!("Failed to send error response: {}", e))?;
        return Err(e);
    }
    
    // Handle connection in a blocking context since it uses std::sync::Mutex
    let response = tokio::task::spawn_blocking(move || {
        // Create a new tokio runtime for the async operations inside
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async move {
            connection_coordinator.handle_incoming_connection(handshake_msg).await
        })
    }).await
    .map_err(|e| format!("Task join error: {}", e))?
    .unwrap_or_else(|e| {
        eprintln!("[BridgeServer] Error handling connection: {}", e);
        super::handshake::HandshakeMessage::error(e.to_string())
    });
    
    // Send response
    super::handshake::send_handshake_server(&mut tls_stream, &response).await
        .map_err(|e| format!("Failed to send response: {}", e))?;
    
    println!("[BridgeServer] Handshake complete, response sent");
    
    Ok(())
}

/// Verify certificate for trusted devices (synchronous, no async)
fn verify_trusted_device_certificate(
    handshake_msg: &super::handshake::HandshakeMessage,
) -> Result<(), String> {
    if let super::handshake::HandshakeMessage::InitiatorHello { 
        ref device_id, 
        ref cert_fingerprint,
        .. 
    } = handshake_msg {
        // Check if device is already trusted
        let trust_manager = super::trust::get_trust_manager();
        let trust_manager_lock = trust_manager.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Ok(true) = trust_manager_lock.is_trusted(device_id) {
            // Device is trusted - verify certificate fingerprint
            match trust_manager_lock.verify_certificate(device_id, cert_fingerprint) {
                Ok(true) => {
                    println!("[BridgeServer] Certificate verified for trusted device {}", &device_id[..8]);
                    Ok(())
                }
                Ok(false) => {
                    Err("Certificate verification failed: fingerprint mismatch".to_string())
                }
                Err(e) => {
                    Err(format!("Certificate verification error: {}", e))
                }
            }
        } else {
            // Device not trusted yet - verification will happen during trust establishment
            Ok(())
        }
    } else {
        Ok(())
    }
}

// Global bridge server instance
static BRIDGE_SERVER: Lazy<Arc<Mutex<Option<BridgeServer>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(None))
});

/// Start the bridge server
pub async fn start_bridge_server(
    connection_coordinator: Arc<super::connection::ConnectionCoordinator>,
) -> Result<(), String> {
    let config = load_config()?;
    let port = config.bridge_port;
    
    // Check if server is already running
    {
        let server_lock = BRIDGE_SERVER.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if server_lock.is_some() {
            return Err("Bridge server already running".to_string());
        }
    }
    
    // Create and start server
    let server = BridgeServer::new(port);
    
    // Store server instance
    {
        let mut server_lock = BRIDGE_SERVER.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        *server_lock = Some(server);
    }
    
    // Get server reference for the async task
    let server_arc = BRIDGE_SERVER.clone();
    
    // Start server in background task
    tokio::spawn(async move {
        // Get server from the global state
        let server_opt = {
            let server_lock = server_arc.lock().unwrap();
            server_lock.as_ref().map(|s| (s.port, s.running.clone()))
        };
        
        if let Some((port, running)) = server_opt {
            // Create a new server instance for this task
            let server = BridgeServer {
                port,
                running,
            };
            
            if let Err(e) = server.start(connection_coordinator).await {
                eprintln!("[BridgeServer] Server error: {}", e);
            }
        }
    });
    
    println!("[BridgeServer] Started on port {}", port);
    Ok(())
}

/// Stop the bridge server
pub fn stop_bridge_server() -> Result<(), String> {
    let server_lock = BRIDGE_SERVER.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    if let Some(ref server) = *server_lock {
        server.stop();
        println!("[BridgeServer] Stopped");
    }
    
    Ok(())
}

// Convenience functions

/// Connect to a device
pub fn connect_to_device(device: &DiscoveredDevice) -> Result<(), String> {
    let manager_lock = get_bridge_manager()?;
    let mut manager = manager_lock.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(ref mut mgr) = *manager {
        mgr.connect(device)
    } else {
        Err("Bridge manager not initialized".to_string())
    }
}

/// Disconnect from a device
pub fn disconnect_from_device(device_id: &str) -> Result<(), String> {
    let manager_lock = get_bridge_manager()?;
    let mut manager = manager_lock.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(ref mut mgr) = *manager {
        mgr.disconnect(device_id)
    } else {
        Err("Bridge manager not initialized".to_string())
    }
}

/// Send a message to a device
pub fn send_to_device(device_id: &str, message: BridgeMessage) -> Result<(), String> {
    let manager_lock = get_bridge_manager()?;
    let mut manager = manager_lock.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(ref mut mgr) = *manager {
        mgr.send_message(device_id, message)
    } else {
        Err("Bridge manager not initialized".to_string())
    }
}

/// Check if connected to a device
pub fn is_connected_to(device_id: &str) -> Result<bool, String> {
    let manager_lock = get_bridge_manager()?;
    let manager = manager_lock.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(ref mgr) = *manager {
        Ok(mgr.is_connected(device_id))
    } else {
        Ok(false)
    }
}

/// Get all connected device IDs
pub fn get_connected_device_ids() -> Result<Vec<String>, String> {
    let manager_lock = get_bridge_manager()?;
    let manager = manager_lock.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(ref mgr) = *manager {
        Ok(mgr.get_connected_devices())
    } else {
        Ok(Vec::new())
    }
}
