// src/file_share/quic_relay.rs - QUIC Relay for AP Isolation bypass

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use quinn::{Connection, RecvStream, SendStream};
use tokio::sync::mpsc;

/// Relay connection that forwards QUIC streams between two devices
pub struct QuicRelayConnection {
    device_a_id: String,
    device_b_id: String,
    device_a_conn: Option<Connection>,
    device_b_conn: Option<Connection>,
}

impl QuicRelayConnection {
    pub fn new(device_a_id: String, device_b_id: String) -> Self {
        QuicRelayConnection {
            device_a_id,
            device_b_id,
            device_a_conn: None,
            device_b_conn: None,
        }
    }
    
    /// Register device A's connection
    pub fn register_device_a(&mut self, conn: Connection) {
        println!("[QuicRelay] Device A ({}) connected", &self.device_a_id[..8]);
        self.device_a_conn = Some(conn);
    }
    
    /// Register device B's connection
    pub fn register_device_b(&mut self, conn: Connection) {
        println!("[QuicRelay] Device B ({}) connected", &self.device_b_id[..8]);
        self.device_b_conn = Some(conn);
    }
    
    /// Check if both devices are connected
    pub fn is_ready(&self) -> bool {
        self.device_a_conn.is_some() && self.device_b_conn.is_some()
    }
    
    /// Forward stream from A to B
    pub async fn forward_a_to_b(&self, mut recv: RecvStream, mut send: SendStream) -> Result<(), String> {
        let mut buffer = vec![0u8; 8192];
        loop {
            match recv.read(&mut buffer).await {
                Ok(Some(n)) => {
                    send.write_all(&buffer[..n]).await
                        .map_err(|e| format!("Write error: {}", e))?;
                }
                Ok(None) => break, // Stream closed
                Err(e) => return Err(format!("Read error: {}", e)),
            }
        }
        send.finish().map_err(|e| format!("Finish error: {}", e))?;
        Ok(())
    }
}

/// QUIC Relay Manager - manages relay connections between devices
pub struct QuicRelayManager {
    relay_connections: HashMap<String, Arc<Mutex<QuicRelayConnection>>>,
    relay_server_address: String,
}

impl QuicRelayManager {
    pub fn new(relay_server_address: String) -> Self {
        QuicRelayManager {
            relay_connections: HashMap::new(),
            relay_server_address,
        }
    }
    
    /// Create or get relay connection for two devices
    pub fn get_or_create_relay(&mut self, device_a: &str, device_b: &str) -> Arc<Mutex<QuicRelayConnection>> {
        // Create a consistent key regardless of order
        let key = if device_a < device_b {
            format!("{}:{}", device_a, device_b)
        } else {
            format!("{}:{}", device_b, device_a)
        };
        
        self.relay_connections.entry(key.clone())
            .or_insert_with(|| {
                println!("[QuicRelay] Creating relay for {} <-> {}", &device_a[..8], &device_b[..8]);
                Arc::new(Mutex::new(QuicRelayConnection::new(
                    device_a.to_string(),
                    device_b.to_string(),
                )))
            })
            .clone()
    }
}

/// Connect to relay server and register for relaying
pub async fn connect_via_relay(
    local_device_id: &str,
    remote_device_id: &str,
    relay_address: &str,
) -> Result<quinn::Connection, String> {
    println!("[QuicRelay] Connecting to relay server at {}", relay_address);
    
    // Parse relay address
    let addr = relay_address.parse()
        .map_err(|e| format!("Invalid relay address: {}", e))?;
    
    // Get QUIC client config
    let client_config = super::quic_crypto::QuicCertManager::client_config()?;
    
    // Create endpoint
    let mut endpoint = quinn::Endpoint::client("0.0.0.0:0".parse().unwrap())
        .map_err(|e| format!("Failed to create endpoint: {}", e))?;
    endpoint.set_default_client_config(client_config);
    
    // Connect to relay server
    let connection = endpoint.connect(addr, "relay")
        .map_err(|e| format!("Failed to connect: {}", e))?
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;
    
    println!("[QuicRelay] Connected to relay server");
    
    // Send registration message
    let (mut send, mut recv) = connection.open_bi().await
        .map_err(|e| format!("Failed to open stream: {}", e))?;
    
    let registration = serde_json::json!({
        "type": "register",
        "local_device_id": local_device_id,
        "remote_device_id": remote_device_id,
    });
    
    let reg_bytes = serde_json::to_vec(&registration)
        .map_err(|e| format!("Serialize error: {}", e))?;
    
    let len = reg_bytes.len() as u32;
    send.write_all(&len.to_be_bytes()).await
        .map_err(|e| format!("Failed to send length: {}", e))?;
    send.write_all(&reg_bytes).await
        .map_err(|e| format!("Failed to send registration: {}", e))?;
    send.finish()
        .map_err(|e| format!("Failed to finish: {}", e))?;
    
    // Wait for acknowledgment
    let mut len_buf = [0u8; 4];
    recv.read_exact(&mut len_buf).await
        .map_err(|e| format!("Failed to read ack length: {}", e))?;
    let len = u32::from_be_bytes(len_buf) as usize;
    
    let ack_bytes = recv.read_to_end(len).await
        .map_err(|e| format!("Failed to read ack: {}", e))?;
    
    let ack: serde_json::Value = serde_json::from_slice(&ack_bytes)
        .map_err(|e| format!("Deserialize error: {}", e))?;
    
    if ack["status"] != "ok" {
        return Err(format!("Relay registration failed: {:?}", ack));
    }
    
    println!("[QuicRelay] ✓ Registered with relay: {}", ack["message"].as_str().unwrap_or(""));
    
    Ok(connection)
}

/// Get default relay server address
pub fn get_default_relay_address() -> String {
    // You can change this to your relay server
    // For now, use localhost for testing
    "127.0.0.1:45680".to_string()
}

/// Check if we should use relay (detect AP isolation)
pub async fn should_use_relay(target_ip: &str) -> bool {
    // Try a quick UDP ping to detect AP isolation
    use tokio::net::UdpSocket;
    use tokio::time::{timeout, Duration};
    
    let socket = match UdpSocket::bind("0.0.0.0:0").await {
        Ok(s) => s,
        Err(_) => return true, // If we can't bind, assume we need relay
    };
    
    let test_msg = b"PING";
    let target = format!("{}:45679", target_ip);
    
    // Try to send a test packet
    match timeout(Duration::from_millis(500), socket.send_to(test_msg, &target)).await {
        Ok(Ok(_)) => {
            // Packet sent, but we can't reliably detect if it was received
            // due to AP isolation. Return false to try direct first.
            false
        }
        _ => {
            // Send failed or timed out, likely need relay
            println!("[QuicRelay] Direct connection test failed, will use relay");
            true
        }
    }
}
