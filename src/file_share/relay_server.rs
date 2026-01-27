// src/file_share/relay_server.rs - QUIC Relay Server for AP Isolation bypass

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use quinn::{Connection, Endpoint, ServerConfig, RecvStream, SendStream};
use serde::{Deserialize, Serialize};

/// Relay registration message
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RelayRegistration {
    #[serde(rename = "type")]
    msg_type: String,
    local_device_id: String,
    remote_device_id: String,
}

/// Relay acknowledgment
#[derive(Debug, Serialize, Deserialize)]
struct RelayAck {
    status: String,
    message: String,
}

/// Active relay session between two devices
struct RelaySession {
    device_a_id: String,
    device_b_id: String,
    device_a_conn: Option<Connection>,
    device_b_conn: Option<Connection>,
}

impl RelaySession {
    fn new(device_a_id: String, device_b_id: String) -> Self {
        RelaySession {
            device_a_id,
            device_b_id,
            device_a_conn: None,
            device_b_conn: None,
        }
    }
    
    fn is_complete(&self) -> bool {
        self.device_a_conn.is_some() && self.device_b_conn.is_some()
    }
    
    fn add_connection(&mut self, device_id: &str, conn: Connection) -> bool {
        if device_id == self.device_a_id && self.device_a_conn.is_none() {
            self.device_a_conn = Some(conn);
            true
        } else if device_id == self.device_b_id && self.device_b_conn.is_none() {
            self.device_b_conn = Some(conn);
            true
        } else {
            false
        }
    }
}

/// QUIC Relay Server
pub struct QuicRelayServer {
    sessions: Arc<Mutex<HashMap<String, Arc<Mutex<RelaySession>>>>>,
    endpoint: Option<Endpoint>,
}

impl QuicRelayServer {
    pub fn new() -> Self {
        QuicRelayServer {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            endpoint: None,
        }
    }
    
    /// Start relay server on specified port
    pub async fn start(&mut self, port: u16) -> Result<(), String> {
        println!("[RelayServer] Starting QUIC relay server on port {}", port);
        
        // Get server config
        let cert_manager = super::quic_crypto::QuicCertManager::new()?;
        let server_config = cert_manager.server_config()?;
        
        // Bind endpoint
        let addr = format!("0.0.0.0:{}", port).parse()
            .map_err(|e| format!("Invalid address: {}", e))?;
        
        let endpoint = Endpoint::server(server_config, addr)
            .map_err(|e| format!("Failed to bind: {}", e))?;
        
        println!("[RelayServer] ✓ Listening on {}", addr);
        
        self.endpoint = Some(endpoint.clone());
        
        // Accept connections
        let sessions = self.sessions.clone();
        tokio::spawn(async move {
            while let Some(incoming) = endpoint.accept().await {
                let sessions = sessions.clone();
                tokio::spawn(async move {
                    if let Err(e) = Self::handle_connection(incoming, sessions).await {
                        eprintln!("[RelayServer] Connection error: {}", e);
                    }
                });
            }
        });
        
        Ok(())
    }
    
    /// Handle incoming connection
    async fn handle_connection(
        incoming: quinn::Incoming,
        sessions: Arc<Mutex<HashMap<String, Arc<Mutex<RelaySession>>>>>,
    ) -> Result<(), String> {
        let connection = incoming.await
            .map_err(|e| format!("Connection failed: {}", e))?;
        
        let remote_addr = connection.remote_address();
        println!("[RelayServer] New connection from {}", remote_addr);
        
        // Accept first stream for registration
        let (mut send, mut recv) = connection.accept_bi().await
            .map_err(|e| format!("Failed to accept stream: {}", e))?;
        
        // Read registration message
        let mut len_buf = [0u8; 4];
        recv.read_exact(&mut len_buf).await
            .map_err(|e| format!("Failed to read length: {}", e))?;
        let len = u32::from_be_bytes(len_buf) as usize;
        
        let reg_bytes = recv.read_to_end(len).await
            .map_err(|e| format!("Failed to read registration: {}", e))?;
        
        let registration: RelayRegistration = serde_json::from_slice(&reg_bytes)
            .map_err(|e| format!("Invalid registration: {}", e))?;
        
        println!("[RelayServer] Registration: {} wants to connect to {}", 
            &registration.local_device_id[..8], &registration.remote_device_id[..8]);
        
        // Create session key (consistent ordering)
        let session_key = if registration.local_device_id < registration.remote_device_id {
            format!("{}:{}", registration.local_device_id, registration.remote_device_id)
        } else {
            format!("{}:{}", registration.remote_device_id, registration.local_device_id)
        };
        
        // Get or create session
        let session = {
            let mut sessions_lock = sessions.lock().await;
            sessions_lock.entry(session_key.clone())
                .or_insert_with(|| {
                    println!("[RelayServer] Creating new session: {}", session_key);
                    Arc::new(Mutex::new(RelaySession::new(
                        registration.local_device_id.clone(),
                        registration.remote_device_id.clone(),
                    )))
                })
                .clone()
        };
        
        // Add connection to session
        let is_complete = {
            let mut session_lock = session.lock().await;
            session_lock.add_connection(&registration.local_device_id, connection.clone());
            session_lock.is_complete()
        };
        
        // Send acknowledgment
        let ack = RelayAck {
            status: "ok".to_string(),
            message: if is_complete {
                "Both devices connected, relay active".to_string()
            } else {
                "Waiting for other device".to_string()
            },
        };
        
        let ack_bytes = serde_json::to_vec(&ack)
            .map_err(|e| format!("Serialize error: {}", e))?;
        
        let len = ack_bytes.len() as u32;
        send.write_all(&len.to_be_bytes()).await
            .map_err(|e| format!("Failed to send ack length: {}", e))?;
        send.write_all(&ack_bytes).await
            .map_err(|e| format!("Failed to send ack: {}", e))?;
        send.finish()
            .map_err(|e| format!("Failed to finish: {}", e))?;
        
        println!("[RelayServer] Sent ack: {}", ack.message);
        
        // If both devices connected, start relaying
        if is_complete {
            println!("[RelayServer] ✓ Both devices connected, starting relay");
            Self::start_relaying(session, registration.local_device_id).await?;
        }
        
        Ok(())
    }
    
    /// Start relaying streams between two devices
    async fn start_relaying(
        session: Arc<Mutex<RelaySession>>,
        initiator_id: String,
    ) -> Result<(), String> {
        let session_lock = session.lock().await;
        
        let conn_a = session_lock.device_a_conn.as_ref()
            .ok_or("Device A not connected")?;
        let conn_b = session_lock.device_b_conn.as_ref()
            .ok_or("Device B not connected")?;
        
        let conn_a = conn_a.clone();
        let conn_b = conn_b.clone();
        
        drop(session_lock); // Release lock
        
        // Determine which is initiator and which is responder
        let (initiator_conn, responder_conn) = if initiator_id == session.lock().await.device_a_id {
            (conn_a, conn_b)
        } else {
            (conn_b, conn_a)
        };
        
        println!("[RelayServer] Starting bidirectional relay");
        
        // Spawn relay tasks
        tokio::spawn(async move {
            loop {
                // Accept stream from initiator
                match initiator_conn.accept_bi().await {
                    Ok((send_init, recv_init)) => {
                        // Open stream to responder
                        match responder_conn.open_bi().await {
                            Ok((send_resp, recv_resp)) => {
                                // Forward initiator → responder
                                let init_to_resp = tokio::spawn(Self::forward_stream(recv_init, send_resp));
                                // Forward responder → initiator
                                let resp_to_init = tokio::spawn(Self::forward_stream(recv_resp, send_init));
                                
                                // Wait for both directions
                                let _ = tokio::join!(init_to_resp, resp_to_init);
                            }
                            Err(e) => {
                                eprintln!("[RelayServer] Failed to open responder stream: {}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[RelayServer] Failed to accept initiator stream: {}", e);
                        break;
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Forward data from recv to send stream
    async fn forward_stream(mut recv: RecvStream, mut send: SendStream) -> Result<(), String> {
        let mut buffer = vec![0u8; 8192];
        let mut total_bytes = 0u64;
        
        loop {
            match recv.read(&mut buffer).await {
                Ok(Some(n)) => {
                    send.write_all(&buffer[..n]).await
                        .map_err(|e| format!("Write error: {}", e))?;
                    total_bytes += n as u64;
                }
                Ok(None) => {
                    // Stream closed
                    send.finish().map_err(|e| format!("Finish error: {}", e))?;
                    println!("[RelayServer] Stream closed, forwarded {} bytes", total_bytes);
                    break;
                }
                Err(e) => {
                    return Err(format!("Read error: {}", e));
                }
            }
        }
        
        Ok(())
    }
}

/// Start relay server (for standalone relay server binary)
pub async fn run_relay_server(port: u16) -> Result<(), String> {
    let mut server = QuicRelayServer::new();
    server.start(port).await?;
    
    println!("[RelayServer] ✓ Relay server running on port {}", port);
    println!("[RelayServer] Press Ctrl+C to stop");
    
    // Keep running
    tokio::signal::ctrl_c().await
        .map_err(|e| format!("Signal error: {}", e))?;
    
    println!("[RelayServer] Shutting down...");
    Ok(())
}
