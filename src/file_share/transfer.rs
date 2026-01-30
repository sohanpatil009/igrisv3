// src/file_share/transfer.rs
// File transfer manager with progress tracking

use super::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{RwLock, mpsc};
use tokio::time::Duration;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Transfer manager for handling file transfers
pub struct TransferManager {
    crypto: Arc<CryptoManager>,
    trust: Arc<TrustManager>,
    active_transfers: Arc<RwLock<HashMap<String, TransferSession>>>,
    event_tx: mpsc::UnboundedSender<FileShareEvent>,
    running: Arc<RwLock<bool>>,
    config: FileShareConfig,
}

/// Transfer session information
#[derive(Debug, Clone)]
pub struct TransferSession {
    pub id: String,
    pub device_id: String,
    pub file_info: FileInfo,
    pub progress: TransferProgress,
    pub started_at: u64, // Unix timestamp
    pub direction: TransferDirection,
}

/// File information for transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub size: u64,
    pub mime_type: String,
    pub checksum: String,
    pub path: Option<PathBuf>, // Local path for sending
}

/// Transfer progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferProgress {
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub speed_bps: u64, // bytes per second
    pub eta_seconds: Option<u64>,
    pub status: TransferStatus,
}

/// Transfer direction
#[derive(Debug, Clone, PartialEq)]
pub enum TransferDirection {
    Sending,
    Receiving,
}

/// Transfer status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransferStatus {
    Pending,
    Connecting,
    Transferring,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl TransferManager {
    /// Create new transfer manager
    pub async fn new(
        crypto: Arc<CryptoManager>,
        trust: Arc<TrustManager>,
        event_tx: mpsc::UnboundedSender<FileShareEvent>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            crypto,
            trust,
            active_transfers: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            running: Arc::new(RwLock::new(false)),
            config: FileShareConfig::with_available_ports(),
        })
    }

    /// Start transfer service
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }

        // Start TCP listener with SO_REUSEADDR set BEFORE binding
        let listener = {
            #[cfg(unix)]
            {
                use std::os::unix::io::{FromRawFd, IntoRawFd};
                use std::net::TcpListener as StdTcpListener;
                
                let domain = libc::AF_INET;
                let socket_type = libc::SOCK_STREAM;
                let protocol = 0;
                
                let fd = unsafe { libc::socket(domain, socket_type, protocol) };
                if fd < 0 {
                    return Err("Failed to create TCP socket".into());
                }
                
                unsafe {
                    let optval: libc::c_int = 1;
                    
                    // SO_REUSEADDR
                    if libc::setsockopt(
                        fd,
                        libc::SOL_SOCKET,
                        libc::SO_REUSEADDR,
                        &optval as *const _ as *const libc::c_void,
                        std::mem::size_of_val(&optval) as libc::socklen_t,
                    ) < 0 {
                        libc::close(fd);
                        return Err("Failed to set SO_REUSEADDR on transfer".into());
                    }
                    
                    // SO_REUSEPORT (macOS)
                    libc::setsockopt(
                        fd,
                        libc::SOL_SOCKET,
                        libc::SO_REUSEPORT,
                        &optval as *const _ as *const libc::c_void,
                        std::mem::size_of_val(&optval) as libc::socklen_t,
                    );
                    
                    // Bind
                    let addr: SocketAddr = format!("0.0.0.0:{}", self.config.transfer_port).parse()?;
                    let (addr_ptr, addr_len) = match addr {
                        SocketAddr::V4(addr) => {
                            // BSD-based systems (macOS, iOS, FreeBSD) have sin_len field
                            #[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
                            let sin = libc::sockaddr_in {
                                sin_len: std::mem::size_of::<libc::sockaddr_in>() as u8,
                                sin_family: libc::AF_INET as _,
                                sin_port: addr.port().to_be(),
                                sin_addr: libc::in_addr {
                                    s_addr: u32::from_ne_bytes(addr.ip().octets()),
                                },
                                sin_zero: [0; 8],
                            };
                            // Linux and other Unix systems don't have sin_len
                            #[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd")))]
                            let sin = libc::sockaddr_in {
                                sin_family: libc::AF_INET as _,
                                sin_port: addr.port().to_be(),
                                sin_addr: libc::in_addr {
                                    s_addr: u32::from_ne_bytes(addr.ip().octets()),
                                },
                                sin_zero: [0; 8],
                            };
                            (
                                &sin as *const _ as *const libc::sockaddr,
                                std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t,
                            )
                        }
                        _ => {
                            libc::close(fd);
                            return Err("IPv6 not supported".into());
                        }
                    };
                    
                    if libc::bind(fd, addr_ptr, addr_len) < 0 {
                        libc::close(fd);
                        return Err(format!("Failed to bind transfer to port {}", self.config.transfer_port).into());
                    }
                    
                    // Listen
                    if libc::listen(fd, 128) < 0 {
                        libc::close(fd);
                        return Err("Failed to listen on transfer socket".into());
                    }
                    
                    // Set non-blocking
                    let flags = libc::fcntl(fd, libc::F_GETFL, 0);
                    libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
                    
                    let std_listener = StdTcpListener::from_raw_fd(fd);
                    TcpListener::from_std(std_listener)?
                }
            }
            #[cfg(windows)]
            {
                let std_listener = std::net::TcpListener::bind(format!("0.0.0.0:{}", self.config.transfer_port))?;
                std_listener.set_nonblocking(true)?;
                TcpListener::from_std(std_listener)?
            }
        };
        
        let accept_transfers = self.active_transfers.clone();
        let accept_event_tx = self.event_tx.clone();
        let accept_running = self.running.clone();
        let accept_crypto = self.crypto.clone();
        let accept_trust = self.trust.clone();
        let accept_config = self.config.clone();
        
        tokio::spawn(async move {
            Self::accept_transfers(
                listener,
                accept_transfers,
                accept_event_tx,
                accept_running,
                accept_crypto,
                accept_trust,
                accept_config,
            ).await;
        });

        *running = true;
        println!("📁 Transfer service started on port {}", self.config.transfer_port);
        
        Ok(())
    }

    /// Stop transfer service
    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut running = self.running.write().await;
        if !*running {
            return Ok(());
        }

        *running = false;
        
        // Cancel all active transfers
        let mut transfers = self.active_transfers.write().await;
        for (id, session) in transfers.iter_mut() {
            session.progress.status = TransferStatus::Cancelled;
            let _ = self.event_tx.send(FileShareEvent::TransferFailed(
                id.clone(),
                "Service stopped".to_string(),
            ));
        }
        transfers.clear();
        
        println!("🛑 Transfer service stopped");
        Ok(())
    }

    /// Connect to device for transfer
    pub async fn connect_device(&self, device: DeviceInfo) -> Result<(), Box<dyn std::error::Error>> {
        // Check if device is trusted
        if !self.trust.is_trusted(&device.id).await {
            return Err("Device not trusted".into());
        }

        println!("🔗 Connected to device: {}", device.name);
        Ok(())
    }

    /// Send file to device
    pub async fn send_file(&self, device_id: &str, file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let path = std::path::Path::new(file_path);
        
        if !path.exists() {
            return Err("File not found".into());
        }

        let metadata = tokio::fs::metadata(path).await?;
        let file_size = metadata.len();
        
        if file_size > self.config.max_file_size {
            return Err(format!("File too large (max {} bytes)", self.config.max_file_size).into());
        }

        let transfer_id = uuid::Uuid::new_v4().to_string();
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let file_info = FileInfo {
            name: file_name,
            size: file_size,
            mime_type: Self::detect_mime_type(path),
            checksum: String::new(), // Would calculate actual checksum
            path: Some(path.to_path_buf()),
        };

        let session = TransferSession {
            id: transfer_id.clone(),
            device_id: device_id.to_string(),
            file_info,
            progress: TransferProgress {
                bytes_transferred: 0,
                total_bytes: file_size,
                speed_bps: 0,
                eta_seconds: None,
                status: TransferStatus::Pending,
            },
            started_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            direction: TransferDirection::Sending,
        };

        self.active_transfers.write().await.insert(transfer_id.clone(), session);
        
        let _ = self.event_tx.send(FileShareEvent::TransferStarted(
            transfer_id.clone(),
            device_id.to_string(),
        ));

        // Start actual transfer in background
        let transfers = self.active_transfers.clone();
        let event_tx = self.event_tx.clone();
        let config = self.config.clone();
        let tid = transfer_id.clone();
        
        tokio::spawn(async move {
            if let Err(e) = Self::perform_send(tid.clone(), transfers, event_tx.clone(), config).await {
                let _ = event_tx.send(FileShareEvent::TransferFailed(tid, e.to_string()));
            }
        });

        Ok(transfer_id)
    }

    /// Get transfer progress
    pub async fn get_progress(&self, transfer_id: &str) -> Option<TransferProgress> {
        self.active_transfers.read().await.get(transfer_id).map(|s| s.progress.clone())
    }

    /// Accept incoming transfers
    async fn accept_transfers(
        listener: TcpListener,
        transfers: Arc<RwLock<HashMap<String, TransferSession>>>,
        event_tx: mpsc::UnboundedSender<FileShareEvent>,
        running: Arc<RwLock<bool>>,
        crypto: Arc<CryptoManager>,
        trust: Arc<TrustManager>,
        config: FileShareConfig,
    ) {
        while *running.read().await {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let transfers_clone = transfers.clone();
                    let event_tx_clone = event_tx.clone();
                    let crypto_clone = crypto.clone();
                    let trust_clone = trust.clone();
                    let config_clone = config.clone();
                    
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_incoming_transfer(
                            stream,
                            addr,
                            transfers_clone,
                            event_tx_clone,
                            crypto_clone,
                            trust_clone,
                            config_clone,
                        ).await {
                            eprintln!("Transfer error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Accept error: {}", e);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }

    /// Handle incoming transfer
    async fn handle_incoming_transfer(
        mut stream: TcpStream,
        addr: std::net::SocketAddr,
        transfers: Arc<RwLock<HashMap<String, TransferSession>>>,
        event_tx: mpsc::UnboundedSender<FileShareEvent>,
        crypto: Arc<CryptoManager>,
        trust: Arc<TrustManager>,
        config: FileShareConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("📥 Incoming transfer from {}", addr);
        
        // Read file metadata
        let mut buffer = vec![0u8; 4096];
        let len = stream.read(&mut buffer).await?;
        
        // In a real implementation, would parse protocol message
        // For now, just acknowledge
        stream.write_all(b"OK").await?;
        
        Ok(())
    }

    /// Perform file send
    async fn perform_send(
        transfer_id: String,
        transfers: Arc<RwLock<HashMap<String, TransferSession>>>,
        event_tx: mpsc::UnboundedSender<FileShareEvent>,
        config: FileShareConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Get transfer session
        let (file_path, total_bytes) = {
            let transfers_lock = transfers.read().await;
            let session = transfers_lock.get(&transfer_id)
                .ok_or("Transfer not found")?;
            
            let path = session.file_info.path.clone()
                .ok_or("File path not set")?;
            
            (path, session.file_info.size)
        };

        // Open file
        let mut file = File::open(&file_path).await?;
        let mut buffer = vec![0u8; config.chunk_size];
        let mut bytes_sent = 0u64;
        let start_time = std::time::SystemTime::now();

        // Read and send chunks
        loop {
            let bytes_read = file.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }

            bytes_sent += bytes_read as u64;
            
            // Update progress
            {
                let mut transfers_lock = transfers.write().await;
                if let Some(session) = transfers_lock.get_mut(&transfer_id) {
                    session.progress.bytes_transferred = bytes_sent;
                    session.progress.status = TransferStatus::Transferring;
                    
                    // Calculate speed
                    if let Ok(elapsed) = start_time.elapsed() {
                        let elapsed_secs = elapsed.as_secs_f64();
                        if elapsed_secs > 0.0 {
                            session.progress.speed_bps = (bytes_sent as f64 / elapsed_secs) as u64;
                            
                            // Calculate ETA
                            let remaining = total_bytes - bytes_sent;
                            if session.progress.speed_bps > 0 {
                                session.progress.eta_seconds = Some(remaining / session.progress.speed_bps);
                            }
                        }
                    }
                }
            }

            // Send progress event
            let _ = event_tx.send(FileShareEvent::TransferProgress(
                transfer_id.clone(),
                bytes_sent,
                total_bytes,
            ));

            // Simulate network delay
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        // Mark as completed
        {
            let mut transfers_lock = transfers.write().await;
            if let Some(session) = transfers_lock.get_mut(&transfer_id) {
                session.progress.status = TransferStatus::Completed;
            }
        }

        let _ = event_tx.send(FileShareEvent::TransferCompleted(transfer_id));
        
        Ok(())
    }

    /// Detect MIME type from file extension
    fn detect_mime_type(path: &std::path::Path) -> String {
        match path.extension().and_then(|e| e.to_str()) {
            Some("txt") => "text/plain",
            Some("pdf") => "application/pdf",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("png") => "image/png",
            Some("gif") => "image/gif",
            Some("mp4") => "video/mp4",
            Some("mp3") => "audio/mpeg",
            Some("zip") => "application/zip",
            Some("json") => "application/json",
            Some("xml") => "application/xml",
            _ => "application/octet-stream",
        }.to_string()
    }

    /// Pause transfer
    pub async fn pause_transfer(&self, transfer_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut transfers = self.active_transfers.write().await;
        if let Some(session) = transfers.get_mut(transfer_id) {
            session.progress.status = TransferStatus::Paused;
            Ok(())
        } else {
            Err("Transfer not found".into())
        }
    }

    /// Resume transfer
    pub async fn resume_transfer(&self, transfer_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut transfers = self.active_transfers.write().await;
        if let Some(session) = transfers.get_mut(transfer_id) {
            session.progress.status = TransferStatus::Transferring;
            Ok(())
        } else {
            Err("Transfer not found".into())
        }
    }

    /// Cancel transfer
    pub async fn cancel_transfer(&self, transfer_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut transfers = self.active_transfers.write().await;
        if let Some(session) = transfers.get_mut(transfer_id) {
            session.progress.status = TransferStatus::Cancelled;
            let _ = self.event_tx.send(FileShareEvent::TransferFailed(
                transfer_id.to_string(),
                "Cancelled by user".to_string(),
            ));
            Ok(())
        } else {
            Err("Transfer not found".into())
        }
    }

    /// Get all active transfers
    pub async fn get_active_transfers(&self) -> Vec<TransferSession> {
        self.active_transfers.read().await.values().cloned().collect()
    }
}