// src/file_share/bridge.rs
// Bridge service for cross-network connections using 4-digit codes

use super::*;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{RwLock, mpsc};
use tokio::time::{interval, Duration};
use serde::{Serialize, Deserialize};
use rand::Rng;

/// Bridge service for connecting devices across different networks
pub struct BridgeService {
    device_info: DeviceInfo,
    current_code: Arc<RwLock<Option<BridgeCode>>>,
    pending_connections: Arc<RwLock<HashMap<String, PendingConnection>>>,
    active_bridges: Arc<RwLock<HashMap<String, BridgeConnection>>>,
    event_tx: mpsc::UnboundedSender<FileShareEvent>,
    running: Arc<RwLock<bool>>,
    config: FileShareConfig,
}

/// Bridge code for device connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeCode {
    pub code: String,
    pub device_info: DeviceInfo,
    pub created_at: u64, // Unix timestamp instead of Instant
    pub expires_at: u64, // Unix timestamp instead of Instant
}

/// Pending connection waiting for verification
#[derive(Debug, Clone)]
pub struct PendingConnection {
    pub device_info: DeviceInfo,
    pub socket_addr: SocketAddr,
    pub created_at: u64, // Unix timestamp
}

/// Active bridge connection
#[derive(Debug, Clone)]
pub struct BridgeConnection {
    pub device_info: DeviceInfo,
    pub socket_addr: SocketAddr,
    pub established_at: u64, // Unix timestamp
}

/// Bridge protocol messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeMessage {
    CodeRequest,
    CodeResponse(BridgeCode),
    ConnectRequest(String, DeviceInfo), // code, device_info
    ConnectResponse(bool, Option<DeviceInfo>), // success, device_info
    Ping,
    Pong,
}

impl BridgeService {
    /// Create new bridge service
    pub async fn new(
        device_info: DeviceInfo,
        event_tx: mpsc::UnboundedSender<FileShareEvent>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            device_info,
            current_code: Arc::new(RwLock::new(None)),
            pending_connections: Arc::new(RwLock::new(HashMap::new())),
            active_bridges: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            running: Arc::new(RwLock::new(false)),
            config: FileShareConfig::default(),
        })
    }

    /// Start bridge service
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }

        // Generate initial code
        self.generate_new_code().await;

        // Start TCP listener for incoming connections
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.config.bridge_port)).await?;
        
        let accept_pending = self.pending_connections.clone();
        let accept_active = self.active_bridges.clone();
        let accept_event_tx = self.event_tx.clone();
        let accept_running = self.running.clone();
        let accept_device_info = self.device_info.clone();
        
        tokio::spawn(async move {
            Self::accept_connections(
                listener,
                accept_pending,
                accept_active,
                accept_event_tx,
                accept_running,
                accept_device_info,
            ).await;
        });

        // Start code rotation task
        let code_rotation_current = self.current_code.clone();
        let code_rotation_device = self.device_info.clone();
        let code_rotation_running = self.running.clone();
        
        tokio::spawn(async move {
            Self::code_rotation_task(
                code_rotation_current,
                code_rotation_device,
                code_rotation_running,
            ).await;
        });

        // Start cleanup task
        let cleanup_pending = self.pending_connections.clone();
        let cleanup_active = self.active_bridges.clone();
        let cleanup_running = self.running.clone();
        
        tokio::spawn(async move {
            Self::cleanup_task(cleanup_pending, cleanup_active, cleanup_running).await;
        });

        *running = true;
        println!("🌉 Bridge service started on port {}", self.config.bridge_port);
        
        Ok(())
    }

    /// Stop bridge service
    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut running = self.running.write().await;
        if !*running {
            return Ok(());
        }

        *running = false;
        
        // Clear all connections
        self.pending_connections.write().await.clear();
        self.active_bridges.write().await.clear();
        *self.current_code.write().await = None;
        
        println!("🛑 Bridge service stopped");
        Ok(())
    }

    /// Get current bridge code
    pub async fn get_code(&self) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        if let Some(code) = self.current_code.read().await.as_ref() {
            if code.expires_at > now {
                return code.code.clone();
            }
        }
        
        // Generate new code if expired or missing
        self.generate_new_code().await;
        
        if let Some(code) = self.current_code.read().await.as_ref() {
            code.code.clone()
        } else {
            "0000".to_string() // Fallback
        }
    }

    /// Connect to device using bridge code
    pub async fn connect_by_code(&self, code: &str) -> Result<(), Box<dyn std::error::Error>> {
        // This would typically involve a discovery mechanism to find the device with this code
        // For now, we'll implement a simple broadcast approach
        
        println!("🔗 Attempting to connect using code: {}", code);
        
        // Try to connect to common local network ranges
        let ranges = vec![
            "192.168.1",
            "192.168.0", 
            "10.0.0",
            "172.16.0",
        ];
        
        for range in ranges {
            for i in 1..255 {
                let ip = format!("{}.{}", range, i);
                if let Ok(addr) = format!("{}:{}", ip, self.config.bridge_port).parse::<SocketAddr>() {
                    if let Ok(stream) = tokio::time::timeout(
                        Duration::from_millis(100),
                        TcpStream::connect(addr)
                    ).await {
                        if let Ok(mut stream) = stream {
                            // Send connect request
                            let request = BridgeMessage::ConnectRequest(
                                code.to_string(),
                                self.device_info.clone()
                            );
                            
                            if let Ok(data) = serde_json::to_vec(&request) {
                                if tokio::io::AsyncWriteExt::write_all(&mut stream, &data).await.is_ok() {
                                    // Wait for response
                                    let mut buffer = [0u8; 4096];
                                    if let Ok(len) = tokio::io::AsyncReadExt::read(&mut stream, &mut buffer).await {
                                        if let Ok(response) = serde_json::from_slice::<BridgeMessage>(&buffer[..len]) {
                                            if let BridgeMessage::ConnectResponse(true, Some(device_info)) = response {
                                                // Connection successful
                                                let connection = BridgeConnection {
                                                    device_info: device_info.clone(),
                                                    socket_addr: addr,
                                                    established_at: std::time::SystemTime::now()
                                                        .duration_since(std::time::UNIX_EPOCH)
                                                        .unwrap_or_default()
                                                        .as_secs(),
                                                };
                                                
                                                self.active_bridges.write().await.insert(
                                                    device_info.id.clone(),
                                                    connection
                                                );
                                                
                                                let _ = self.event_tx.send(
                                                    FileShareEvent::DeviceDiscovered(device_info)
                                                );
                                                
                                                println!("✅ Connected to device via bridge code");
                                                return Ok(());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Err("Failed to connect using bridge code".into())
    }

    /// Generate new bridge code
    async fn generate_new_code(&self) {
        let mut rng = rand::thread_rng();
        let code = format!("{:04}", rng.gen_range(1000..=9999));
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let bridge_code = BridgeCode {
            code: code.clone(),
            device_info: self.device_info.clone(),
            created_at: now,
            expires_at: now + 600, // 10 minutes
        };
        
        *self.current_code.write().await = Some(bridge_code);
        println!("🔑 New bridge code generated: {}", code);
    }

    /// Accept incoming connections
    async fn accept_connections(
        listener: TcpListener,
        pending: Arc<RwLock<HashMap<String, PendingConnection>>>,
        active: Arc<RwLock<HashMap<String, BridgeConnection>>>,
        event_tx: mpsc::UnboundedSender<FileShareEvent>,
        running: Arc<RwLock<bool>>,
        device_info: DeviceInfo,
    ) {
        while *running.read().await {
            match listener.accept().await {
                Ok((mut stream, addr)) => {
                    let pending_clone = pending.clone();
                    let active_clone = active.clone();
                    let event_tx_clone = event_tx.clone();
                    let device_info_clone = device_info.clone();
                    
                    tokio::spawn(async move {
                        let mut buffer = [0u8; 4096];
                        
                        if let Ok(len) = tokio::io::AsyncReadExt::read(&mut stream, &mut buffer).await {
                            if let Ok(message) = serde_json::from_slice::<BridgeMessage>(&buffer[..len]) {
                                match message {
                                    BridgeMessage::ConnectRequest(code, remote_device) => {
                                        // Validate code (in real implementation, check against current code)
                                        let success = code.len() == 4 && code.chars().all(|c| c.is_ascii_digit());
                                        
                                        let response = if success {
                                            // Add to active connections
                                            let connection = BridgeConnection {
                                                device_info: remote_device.clone(),
                                                socket_addr: addr,
                                                established_at: std::time::SystemTime::now()
                                                    .duration_since(std::time::UNIX_EPOCH)
                                                    .unwrap_or_default()
                                                    .as_secs(),
                                            };
                                            
                                            active_clone.write().await.insert(
                                                remote_device.id.clone(),
                                                connection
                                            );
                                            
                                            let _ = event_tx_clone.send(
                                                FileShareEvent::DeviceDiscovered(remote_device)
                                            );
                                            
                                            BridgeMessage::ConnectResponse(true, Some(device_info_clone))
                                        } else {
                                            BridgeMessage::ConnectResponse(false, None)
                                        };
                                        
                                        if let Ok(response_data) = serde_json::to_vec(&response) {
                                            let _ = tokio::io::AsyncWriteExt::write_all(&mut stream, &response_data).await;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Bridge accept error: {}", e);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }

    /// Code rotation task
    async fn code_rotation_task(
        current_code: Arc<RwLock<Option<BridgeCode>>>,
        device_info: DeviceInfo,
        running: Arc<RwLock<bool>>,
    ) {
        let mut interval = interval(Duration::from_secs(60)); // Check every minute

        while *running.read().await {
            interval.tick().await;

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let should_rotate = {
                if let Some(code) = current_code.read().await.as_ref() {
                    code.expires_at <= now
                } else {
                    true
                }
            };

            if should_rotate {
                // Generate random number BEFORE the async block to avoid Send issues
                let random_code = {
                    let mut rng = rand::thread_rng();
                    format!("{:04}", rng.gen_range(1000..=9999))
                };
                
                let bridge_code = BridgeCode {
                    code: random_code.clone(),
                    device_info: device_info.clone(),
                    created_at: now,
                    expires_at: now + 600, // 10 minutes
                };
                
                *current_code.write().await = Some(bridge_code);
                println!("🔄 Bridge code rotated: {}", random_code);
            }
        }
    }

    /// Cleanup task
    async fn cleanup_task(
        pending: Arc<RwLock<HashMap<String, PendingConnection>>>,
        active: Arc<RwLock<HashMap<String, BridgeConnection>>>,
        running: Arc<RwLock<bool>>,
    ) {
        let mut interval = interval(Duration::from_secs(30)); // Cleanup every 30 seconds

        while *running.read().await {
            interval.tick().await;

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            // Clean up expired pending connections
            {
                let mut pending_lock = pending.write().await;
                pending_lock.retain(|_, conn| {
                    now - conn.created_at < 300 // 5 minutes
                });
            }

            // Clean up stale active connections
            {
                let mut active_lock = active.write().await;
                active_lock.retain(|_, conn| {
                    now - conn.established_at < 3600 // 1 hour
                });
            }
        }
    }

    /// Get active bridge connections
    pub async fn get_active_bridges(&self) -> Vec<DeviceInfo> {
        self.active_bridges
            .read()
            .await
            .values()
            .map(|conn| conn.device_info.clone())
            .collect()
    }
}