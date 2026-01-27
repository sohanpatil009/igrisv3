// src/file_share/connection.rs - Connection Coordinator for unified connection flow

use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::relay::RelayService;
use super::trust::TrustManager;
use super::discovery::DiscoveryService;
use super::connection_types::{
    ConnectionCode, ConnectionResult, ConnectionType, ConnectionError, DeviceInfo,
};
use super::config::{load_config, OperatingSystem};

/// Connection Coordinator orchestrates the connection flow between relay, trust, and discovery
pub struct ConnectionCoordinator {
    relay: Arc<RelayService>,
    trust: Arc<Mutex<TrustManager>>,
    discovery: Arc<Mutex<Option<DiscoveryService>>>,
}

impl ConnectionCoordinator {
    /// Create a new ConnectionCoordinator
    pub fn new(
        relay: Arc<RelayService>,
        trust: Arc<Mutex<TrustManager>>,
        discovery: Arc<Mutex<Option<DiscoveryService>>>,
    ) -> Self {
        ConnectionCoordinator {
            relay,
            trust,
            discovery,
        }
    }
    
    /// Generate and register this device's connection code
    /// Returns a ConnectionCode with the 4-digit code and expiry information
    pub fn generate_my_code(&self) -> Result<ConnectionCode, ConnectionError> {
        // Load device config to get identity and network info
        let config = load_config()
            .map_err(|e| ConnectionError::NetworkError(format!("Failed to load config: {}", e)))?;
        
        // Get local IP address (use first non-loopback IPv4 address)
        let ip_address = self.get_local_ip_address()
            .unwrap_or_else(|| "127.0.0.1".to_string());
        
        // Check if we already have a valid code for this device
        if let Some(existing_code) = self.relay.get_my_code(&config.identity.id) {
            // Return the existing code with updated remaining time
            let registration = self.relay.lookup_device(&existing_code)
                .map_err(|_| ConnectionError::CodeExpired)?;
            
            let expires_at = registration.created_at + std::time::Duration::from_secs(600);
            return Ok(ConnectionCode::new(existing_code, expires_at));
        }
        
        // Register device with relay service to generate a new code
        let code = self.relay.register_device(
            config.identity.id.clone(),
            ip_address,
            config.bridge_port,
            config.identity.hostname.clone(),
            config.identity.label.clone(),
            config.identity.os.clone(),
        ).map_err(|e| ConnectionError::NetworkError(format!("Failed to register device: {}", e)))?;
        
        // Calculate expiry time (10 minutes from now)
        let expires_at = Instant::now() + std::time::Duration::from_secs(600);
        
        println!("[ConnectionCoordinator] Generated code: {}", code);
        
        Ok(ConnectionCode::new(code, expires_at))
    }
    
    /// Get the local IP address for this device
    fn get_local_ip_address(&self) -> Option<String> {
        use std::net::UdpSocket;
        
        // Try to get local IP by connecting to a public DNS server
        // This doesn't actually send data, just determines which interface would be used
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
            if socket.connect("8.8.8.8:80").is_ok() {
                if let Ok(addr) = socket.local_addr() {
                    return Some(addr.ip().to_string());
                }
            }
        }
        
        // Fallback: try to get from network interfaces
        if let Ok(interfaces) = get_if_addrs::get_if_addrs() {
            for iface in interfaces {
                if let get_if_addrs::IfAddr::V4(ref addr) = iface.addr {
                    if !addr.ip.is_loopback() && !addr.ip.is_link_local() {
                        return Some(addr.ip.to_string());
                    }
                }
            }
        }
        
        None
    }
    
    /// Connect to a device directly using IP address (bypassing code system)
    /// This method performs direct connection without requiring a 4-digit code
    pub async fn connect_direct(&self, ip_address: &str, bridge_port: u16, device_label: &str) -> Result<ConnectionResult, ConnectionError> {
        println!("[ConnectionCoordinator] Direct connection to {} at {}:{}", device_label, ip_address, bridge_port);
        
        // Get local device info for handshake
        let config = load_config()
            .map_err(|e| ConnectionError::NetworkError(format!("Failed to load config: {}", e)))?;
        
        // Get local certificate fingerprint from QUIC crypto
        let cert_manager_lock = super::quic_crypto::get_quic_cert_manager()
            .map_err(|e| ConnectionError::NetworkError(format!("Failed to get cert manager: {}", e)))?;
        let cert_manager = cert_manager_lock.lock()
            .map_err(|e| ConnectionError::NetworkError(format!("Lock error: {}", e)))?;
        let cert_mgr = cert_manager.as_ref()
            .ok_or_else(|| ConnectionError::NetworkError("Certificate manager not initialized".to_string()))?;
        let cert_fingerprint = cert_mgr.fingerprint().to_string();
        
        // Get local IP address
        let local_ip = self.get_local_ip_address()
            .unwrap_or_else(|| "127.0.0.1".to_string());
        
        // Create handshake message with local device info
        let handshake_msg = super::handshake::HandshakeMessage::initiator_hello(
            config.identity.id.clone(),
            config.identity.hostname.clone(),
            config.identity.label.clone(),
            config.identity.os.clone(),
            local_ip,
            config.bridge_port,
            cert_fingerprint.clone(),
        );
        
        // Establish QUIC connection to remote device and perform handshake
        let response = self.establish_quic_connection_with_handshake(
            ip_address,
            bridge_port,
            handshake_msg,
        ).await?;
        
        // Parse the response to get the real device_id
        let (real_device_id, remote_cert_fingerprint) = match &response {
            super::handshake::HandshakeMessage::ResponderAck {
                device_id,
                cert_fingerprint,
                ..
            } => (device_id.clone(), cert_fingerprint.clone()),
            _ => {
                return Err(ConnectionError::TrustFailed("Invalid handshake response".to_string()));
            }
        };
        
        println!("[ConnectionCoordinator] Received device_id from remote: {}", &real_device_id[..8]);
        
        // Create registration with the real device_id
        let registration = super::relay::DeviceRegistration {
            code: "DIRECT".to_string(),
            device_id: real_device_id.clone(),
            ip_address: ip_address.to_string(),
            bridge_port,
            hostname: device_label.to_string(),
            label: device_label.to_string(),
            os: super::config::OperatingSystem::Unknown,
            created_at: std::time::Instant::now(),
        };
        
        // Add device to discovery cache
        let discovered_device = self.add_to_discovery_cache(&registration).await?;
        
        // Continue to trust establishment
        self.connect_with_code_part3(registration, discovered_device, response, cert_fingerprint).await
    }
    /// Connect to a device using their 4-digit code
    /// This method performs the complete connection flow:
    /// 1. Validate code format and check rate limiting
    /// 2. Lookup device in relay service
    /// 3. Establish TLS connection and perform handshake
    /// 4. Establish bidirectional trust
    /// 2. Lookup device in relay service
    /// 3. Establish TLS connection and perform handshake
    /// 4. Establish bidirectional trust
    pub async fn connect_with_code(&self, code: &str) -> Result<ConnectionResult, ConnectionError> {
        // Part 1: Lookup
        
        // Validate code format (4 digits, numeric)
        if code.len() != 4 {
            return Err(ConnectionError::InvalidCode);
        }
        
        if !code.chars().all(|c| c.is_numeric()) {
            return Err(ConnectionError::InvalidCode);
        }
        
        // Lookup device in relay service
        let registration = self.relay.lookup_device(code)
            .map_err(|e| {
                if e.contains("expired") {
                    ConnectionError::CodeExpired
                } else {
                    ConnectionError::CodeNotFound
                }
            })?;
        
        // Check if device is already trusted
        let trust_manager = self.trust.lock()
            .map_err(|e| ConnectionError::NetworkError(format!("Lock error: {}", e)))?;
        
        if trust_manager.is_trusted(&registration.device_id)
            .map_err(|e| ConnectionError::NetworkError(format!("Trust check failed: {}", e)))? {
            // Device is already trusted - we can still proceed to reconnect
            // but we'll return AlreadyTrusted connection type
            drop(trust_manager); // Release lock before continuing
            
            // Add to discovery cache and return
            return self.handle_already_trusted_device(registration).await;
        }
        
        // Check rate limiting for this device
        trust_manager.check_rate_limit(&registration.device_id)
            .map_err(|remaining_secs| ConnectionError::RateLimited { remaining_secs })?;
        
        drop(trust_manager); // Release lock before async operations
        
        println!("[ConnectionCoordinator] Code lookup successful: {} ({})", 
            registration.label, &registration.device_id[..8]);
        
        // Continue to Part 2: Connection
        self.connect_with_code_part2(registration).await
    }
    
    /// Handle connection to an already trusted device
    async fn handle_already_trusted_device(
        &self,
        registration: super::relay::DeviceRegistration,
    ) -> Result<ConnectionResult, ConnectionError> {
        // Add device to discovery cache
        let discovered_device = self.add_to_discovery_cache(&registration).await?;
        
        Ok(ConnectionResult {
            device: discovered_device,
            trust_established: true,
            connection_type: ConnectionType::AlreadyTrusted,
        })
    }
    
    /// Add a device to the discovery cache
    async fn add_to_discovery_cache(
        &self,
        registration: &super::relay::DeviceRegistration,
    ) -> Result<super::discovery::DiscoveredDevice, ConnectionError> {
        let discovery_lock = self.discovery.lock()
            .map_err(|e| ConnectionError::NetworkError(format!("Lock error: {}", e)))?;
        
        if let Some(ref discovery_service) = *discovery_lock {
            // Add device manually to discovery cache
            let discovered = discovery_service.add_manual_device(
                &registration.ip_address,
                registration.bridge_port,
            ).await.map_err(|e| ConnectionError::NetworkError(format!("Failed to add to discovery: {}", e)))?;
            
            Ok(discovered)
        } else {
            Err(ConnectionError::NetworkError("Discovery service not initialized".to_string()))
        }
    }
    
    /// Part 2 of connect_with_code - establish TLS connection and send handshake
    async fn connect_with_code_part2(
        &self,
        registration: super::relay::DeviceRegistration,
    ) -> Result<ConnectionResult, ConnectionError> {
        println!("[ConnectionCoordinator] Connecting to {} at {}:{}", 
            registration.label, registration.ip_address, registration.bridge_port);
        
        // Add device to discovery cache first
        let discovered_device = self.add_to_discovery_cache(&registration).await?;
        
        // Get local device info for handshake
        let config = load_config()
            .map_err(|e| ConnectionError::NetworkError(format!("Failed to load config: {}", e)))?;
        
        // Get local certificate fingerprint from QUIC crypto
        let cert_manager_lock = super::quic_crypto::get_quic_cert_manager()
            .map_err(|e| ConnectionError::NetworkError(format!("Failed to get cert manager: {}", e)))?;
        let cert_manager = cert_manager_lock.lock()
            .map_err(|e| ConnectionError::NetworkError(format!("Lock error: {}", e)))?;
        let cert_mgr = cert_manager.as_ref()
            .ok_or_else(|| ConnectionError::NetworkError("Certificate manager not initialized".to_string()))?;
        let cert_fingerprint = cert_mgr.fingerprint().to_string();
        
        // Get local IP address
        let local_ip = self.get_local_ip_address()
            .unwrap_or_else(|| "127.0.0.1".to_string());
        
        // Create handshake message with local device info
        let handshake_msg = super::handshake::HandshakeMessage::initiator_hello(
            config.identity.id.clone(),
            config.identity.hostname.clone(),
            config.identity.label.clone(),
            config.identity.os.clone(),
            local_ip,
            config.bridge_port,
            cert_fingerprint.clone(),
        );
        
        // Establish QUIC connection to remote device and perform handshake
        let response = self.establish_quic_connection_with_handshake(
            &registration.ip_address,
            registration.bridge_port,
            handshake_msg,
        ).await?;
        
        // Continue to Part 3: Trust establishment
        self.connect_with_code_part3(registration, discovered_device, response, cert_fingerprint).await
    }
    
    /// Establish QUIC connection and perform handshake
    async fn establish_quic_connection_with_handshake(
        &self,
        ip_address: &str,
        port: u16,
        handshake_msg: super::handshake::HandshakeMessage,
    ) -> Result<super::handshake::HandshakeMessage, ConnectionError> {
        use super::quic_bridge::get_quic_bridge_manager;
        
        println!("[ConnectionCoordinator] establish_quic_connection_with_handshake() called for {}:{}", ip_address, port);
        
        let manager_lock = get_quic_bridge_manager()
            .map_err(|e| {
                println!("[ConnectionCoordinator] Failed to get QUIC bridge manager: {}", e);
                ConnectionError::NetworkError(e)
            })?;
        
        let mut manager = manager_lock.lock()
            .map_err(|e| {
                println!("[ConnectionCoordinator] Failed to lock manager: {}", e);
                ConnectionError::NetworkError(format!("Lock error: {}", e))
            })?;
        
        let mgr = manager.as_mut()
            .ok_or_else(|| {
                println!("[ConnectionCoordinator] QUIC bridge not initialized");
                ConnectionError::NetworkError("QUIC bridge not initialized".to_string())
            })?;
        
        // Create temporary device for connection
        let temp_device = super::discovery::DiscoveredDevice {
            id: format!("temp_{}", ip_address.replace(".", "_")),
            hostname: ip_address.to_string(),
            label: ip_address.to_string(),
            os: super::config::OperatingSystem::Unknown,
            ip_address: ip_address.parse()
                .map_err(|e| {
                    println!("[ConnectionCoordinator] Invalid IP address: {}", e);
                    ConnectionError::NetworkError(format!("Invalid IP: {}", e))
                })?,
            bridge_port: port,
            last_seen: std::time::Instant::now(),
            is_trusted: false,
            code: None,
        };
        
        println!("[ConnectionCoordinator] Created temp device: {} at {}:{}", temp_device.label, temp_device.ip_address, temp_device.bridge_port);
        
        // Connect via QUIC (TLS 1.3 automatic!)
        println!("[ConnectionCoordinator] Calling mgr.connect()...");
        mgr.connect(&temp_device).await
            .map_err(|e| {
                println!("[ConnectionCoordinator] QUIC connect failed: {}", e);
                ConnectionError::NetworkError(e)
            })?;
        
        println!("[ConnectionCoordinator] QUIC connection established, getting connection from map...");
        
        // Send handshake message over QUIC stream
        let handshake_bytes = serde_json::to_vec(&handshake_msg)
            .map_err(|e| {
                println!("[ConnectionCoordinator] Failed to serialize handshake: {}", e);
                ConnectionError::NetworkError(format!("Serialize error: {}", e))
            })?;
        
        let conn = mgr.connections.get(&temp_device.id)
            .ok_or_else(|| {
                println!("[ConnectionCoordinator] Connection not found in map after connect");
                ConnectionError::NetworkError("Connection not found".to_string())
            })?;
        
        println!("[ConnectionCoordinator] Opening bidirectional stream...");
        let (mut send, mut recv) = conn.connection.open_bi().await
            .map_err(|e| {
                println!("[ConnectionCoordinator] Failed to open stream: {}", e);
                ConnectionError::NetworkError(format!("Failed to open stream: {}", e))
            })?;
        
        // Send handshake
        println!("[ConnectionCoordinator] Sending handshake ({} bytes)...", handshake_bytes.len());
        let len = handshake_bytes.len() as u32;
        send.write_all(&len.to_be_bytes()).await
            .map_err(|e| {
                println!("[ConnectionCoordinator] Failed to send length: {}", e);
                ConnectionError::NetworkError(format!("Failed to send length: {}", e))
            })?;
        send.write_all(&handshake_bytes).await
            .map_err(|e| {
                println!("[ConnectionCoordinator] Failed to send handshake: {}", e);
                ConnectionError::NetworkError(format!("Failed to send: {}", e))
            })?;
        send.finish()
            .map_err(|e| {
                println!("[ConnectionCoordinator] Failed to finish stream: {}", e);
                ConnectionError::NetworkError(format!("Failed to finish: {}", e))
            })?;
        
        println!("[ConnectionCoordinator] Handshake sent, waiting for response...");
        
        // Receive response
        let mut len_buf = [0u8; 4];
        recv.read_exact(&mut len_buf).await
            .map_err(|e| {
                println!("[ConnectionCoordinator] Failed to read response length: {}", e);
                ConnectionError::NetworkError(format!("Failed to read length: {}", e))
            })?;
        let len = u32::from_be_bytes(len_buf) as usize;
        
        println!("[ConnectionCoordinator] Reading response ({} bytes)...", len);
        let response_bytes = recv.read_to_end(len).await
            .map_err(|e| {
                println!("[ConnectionCoordinator] Failed to read response: {}", e);
                ConnectionError::NetworkError(format!("Failed to receive: {}", e))
            })?;
        
        let response: super::handshake::HandshakeMessage = serde_json::from_slice(&response_bytes)
            .map_err(|e| {
                println!("[ConnectionCoordinator] Failed to deserialize response: {}", e);
                ConnectionError::NetworkError(format!("Deserialize error: {}", e))
            })?;
        
        println!("[ConnectionCoordinator] Handshake response received successfully");
        
        Ok(response)
    }
    
    /// Part 3 of connect_with_code - establish trust and return result
    async fn connect_with_code_part3(
        &self,
        mut registration: super::relay::DeviceRegistration,
        discovered_device: super::discovery::DiscoveredDevice,
        handshake_response: super::handshake::HandshakeMessage,
        local_cert_fingerprint: String,
    ) -> Result<ConnectionResult, ConnectionError> {
        // Parse handshake response
        match handshake_response {
            super::handshake::HandshakeMessage::ResponderAck {
                device_id,
                cert_fingerprint,
                trust_established,
            } => {
                println!("[ConnectionCoordinator] Received ResponderAck from {}", &device_id[..8]);
                
                // For direct connections, update the registration with the real device_id
                if registration.code == "DIRECT" {
                    registration.device_id = device_id.clone();
                    println!("[ConnectionCoordinator] Updated direct connection device_id to {}", &device_id[..8]);
                }
                
                // Verify device_id matches (should match now after update)
                if device_id != registration.device_id {
                    return Err(ConnectionError::TrustFailed(
                        "Device ID mismatch in handshake response".to_string()
                    ));
                }
                
                // Establish trust locally with the remote device's certificate
                let device_info = DeviceInfo::from(registration.clone());
                
                let mut trust_manager = self.trust.lock()
                    .map_err(|e| ConnectionError::NetworkError(format!("Lock error: {}", e)))?;
                
                trust_manager.establish_trust(&device_info, &cert_fingerprint)
                    .map_err(|e| ConnectionError::TrustFailed(format!("Failed to establish trust: {}", e)))?;
                
                println!("[ConnectionCoordinator] Trust established with {}", device_info.label);
                
                // Update last connected time
                trust_manager.update_last_connected(&device_id)
                    .map_err(|e| ConnectionError::NetworkError(format!("Failed to update last connected: {}", e)))?;
                
                drop(trust_manager); // Release lock
                
                // CRITICAL FIX: Add device to QUIC BridgeManager so UI shows as connected
                println!("[ConnectionCoordinator] Adding device to QUIC BridgeManager...");
                if let Err(e) = super::quic_bridge::connect_to_device_quic(&discovered_device).await {
                    println!("[ConnectionCoordinator] Warning: Failed to add to QUIC BridgeManager: {}", e);
                    // Don't fail the connection - trust is already established
                }
                
                // Return successful connection result
                Ok(ConnectionResult {
                    device: discovered_device,
                    trust_established: true,
                    connection_type: ConnectionType::NewConnection,
                })
            }
            super::handshake::HandshakeMessage::Error { message } => {
                Err(ConnectionError::TrustFailed(format!("Remote error: {}", message)))
            }
            _ => {
                Err(ConnectionError::TrustFailed("Unexpected handshake response".to_string()))
            }
        }
    }
    
    /// Handle incoming connection from another device (responder side)
    /// This is called when we receive an InitiatorHello handshake
    pub async fn handle_incoming_connection(
        &self,
        initiator_hello: super::handshake::HandshakeMessage,
    ) -> Result<super::handshake::HandshakeMessage, ConnectionError> {
        // Extract device info from InitiatorHello
        let (device_id, hostname, label, os, ip_address, bridge_port, cert_fingerprint) = 
            match initiator_hello {
                super::handshake::HandshakeMessage::InitiatorHello {
                    device_id,
                    hostname,
                    label,
                    os,
                    ip_address,
                    bridge_port,
                    cert_fingerprint,
                } => (device_id, hostname, label, os, ip_address, bridge_port, cert_fingerprint),
                _ => {
                    return Ok(super::handshake::HandshakeMessage::error(
                        "Expected InitiatorHello message".to_string()
                    ));
                }
            };
        
        println!("[ConnectionCoordinator] Received connection from {} ({})", label, &device_id[..8]);
        
        // Check rate limiting
        {
            let trust_manager = self.trust.lock()
                .map_err(|e| ConnectionError::NetworkError(format!("Lock error: {}", e)))?;
            
            if let Err(remaining_secs) = trust_manager.check_rate_limit(&device_id) {
                println!("[ConnectionCoordinator] Device {} is rate limited", &device_id[..8]);
                return Ok(super::handshake::HandshakeMessage::error(
                    format!("Too many failed attempts - try again in {} seconds", remaining_secs)
                ));
            }
        }
        
        // Create DeviceInfo for the initiator
        let device_info = DeviceInfo {
            device_id: device_id.clone(),
            hostname,
            label: label.clone(),
            os,
            ip_address: ip_address.clone(),
            bridge_port,
        };
        
        // Establish trust for the initiator
        {
            let mut trust_manager = self.trust.lock()
                .map_err(|e| ConnectionError::NetworkError(format!("Lock error: {}", e)))?;
            
            trust_manager.establish_trust(&device_info, &cert_fingerprint)
                .map_err(|e| {
                    println!("[ConnectionCoordinator] Failed to establish trust: {}", e);
                    ConnectionError::TrustFailed(format!("Failed to establish trust: {}", e))
                })?;
            
            println!("[ConnectionCoordinator] Trust established with initiator {}", label);
        }
        
        // Add initiator to discovery cache
        let discovered_device = {
            let discovery_lock = self.discovery.lock()
                .map_err(|e| ConnectionError::NetworkError(format!("Lock error: {}", e)))?;
            
            if let Some(ref discovery_service) = *discovery_lock {
                let device = discovery_service.add_manual_device(&ip_address, bridge_port).await
                    .map_err(|e| {
                        println!("[ConnectionCoordinator] Failed to add to discovery: {}", e);
                        ConnectionError::NetworkError(format!("Failed to add to discovery: {}", e))
                    })?;
                
                println!("[ConnectionCoordinator] Added initiator to discovery cache");
                Some(device)
            } else {
                None
            }
        };
        
        // CRITICAL FIX: Add device to QUIC BridgeManager so Windows UI shows as connected
        if let Some(ref device) = discovered_device {
            println!("[ConnectionCoordinator] Adding initiator to QUIC BridgeManager...");
            if let Err(e) = super::quic_bridge::connect_to_device_quic(device).await {
                println!("[ConnectionCoordinator] Warning: Failed to add to QUIC BridgeManager: {}", e);
                // Don't fail the connection - trust is already established
            }
        }
        
        // Get our certificate fingerprint for the response from QUIC crypto
        let cert_manager_lock = super::quic_crypto::get_quic_cert_manager()
            .map_err(|e| ConnectionError::NetworkError(format!("Failed to get cert manager: {}", e)))?;
        let cert_manager = cert_manager_lock.lock()
            .map_err(|e| ConnectionError::NetworkError(format!("Lock error: {}", e)))?;
        let cert_mgr = cert_manager.as_ref()
            .ok_or_else(|| ConnectionError::NetworkError("Certificate manager not initialized".to_string()))?;
        let our_cert_fingerprint = cert_mgr.fingerprint().to_string();
        
        // Get our device ID
        let config = load_config()
            .map_err(|e| ConnectionError::NetworkError(format!("Failed to load config: {}", e)))?;
        
        // Send ResponderAck
        let response = super::handshake::HandshakeMessage::responder_ack(
            config.identity.id.clone(),
            our_cert_fingerprint,
            true,
        );
        
        println!("[ConnectionCoordinator] Sending ResponderAck to {}", label);
        
        Ok(response)
    }
}


// ═══════════════════════════════════════════════════════
// GLOBAL CONNECTION COORDINATOR
// ═══════════════════════════════════════════════════════

use once_cell::sync::Lazy;

/// Global connection coordinator instance
static GLOBAL_COORDINATOR: Lazy<Arc<Mutex<Option<Arc<ConnectionCoordinator>>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(None)));

/// Get or create the global connection coordinator
pub fn get_connection_coordinator() -> Result<Arc<ConnectionCoordinator>, String> {
    let mut coord_lock = GLOBAL_COORDINATOR.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    if let Some(ref coordinator) = *coord_lock {
        return Ok(Arc::clone(coordinator));
    }
    
    // Create new coordinator with global services
    let relay = super::relay::get_relay_service();
    let trust = Arc::new(Mutex::new(super::trust::TrustManager::new()));
    
    // Get the GLOBAL discovery service (not None!)
    let discovery = super::discovery::get_discovery_service()
        .map_err(|e| format!("Failed to get discovery service: {}", e))?;
    
    let coordinator = Arc::new(ConnectionCoordinator::new(relay, trust, discovery));
    *coord_lock = Some(Arc::clone(&coordinator));
    
    println!("[ConnectionCoordinator] Global coordinator initialized");
    
    Ok(coordinator)
}
