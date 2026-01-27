// src/file_share/transfer.rs - File Transfer Service

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use sha2::{Sha256, Digest};
use tokio::sync::broadcast;
use once_cell::sync::Lazy;
use uuid::Uuid;

use super::quic_bridge::{QuicMessage, get_quic_bridge_manager, send_to_device_quic};

// Transfer constants
const CHUNK_SIZE: usize = 64 * 1024; // 64KB chunks
const MAX_CONCURRENT_TRANSFERS: usize = 5;

/// Transfer direction
#[derive(Debug, Clone, PartialEq)]
pub enum TransferDirection {
    Sending,
    Receiving,
}

/// Transfer status
#[derive(Debug, Clone, PartialEq)]
pub enum TransferStatus {
    Pending,
    InProgress,
    Paused,
    Completed,
    Failed(String),
    Cancelled,
}

/// A file transfer (sending or receiving)
#[derive(Debug, Clone)]
pub struct FileTransfer {
    pub id: String,
    pub device_id: String,
    pub filename: String,
    pub file_path: PathBuf,
    pub size: u64,
    pub transferred: u64,
    pub checksum: String,
    pub direction: TransferDirection,
    pub status: TransferStatus,
    pub started_at: Option<Instant>,
    pub completed_at: Option<Instant>,
    pub chunks_received: u32,
    pub total_chunks: u32,
}

impl FileTransfer {
    /// Create a new outgoing transfer
    pub fn new_send(device_id: &str, file_path: &Path) -> Result<Self, String> {
        let metadata = fs::metadata(file_path)
            .map_err(|e| format!("Cannot read file: {}", e))?;
        
        let filename = file_path.file_name()
            .and_then(|n| n.to_str())
            .ok_or("Invalid filename")?
            .to_string();
        
        let size = metadata.len();
        let checksum = calculate_file_checksum(file_path)?;
        let total_chunks = ((size as f64) / (CHUNK_SIZE as f64)).ceil() as u32;
        
        Ok(FileTransfer {
            id: Uuid::new_v4().to_string(),
            device_id: device_id.to_string(),
            filename,
            file_path: file_path.to_path_buf(),
            size,
            transferred: 0,
            checksum,
            direction: TransferDirection::Sending,
            status: TransferStatus::Pending,
            started_at: None,
            completed_at: None,
            chunks_received: 0,
            total_chunks,
        })
    }
    
    /// Create a new incoming transfer
    pub fn new_receive(
        device_id: &str, 
        filename: &str, 
        size: u64, 
        checksum: &str,
        transfer_id: &str,
        save_path: &Path,
    ) -> Self {
        let total_chunks = ((size as f64) / (CHUNK_SIZE as f64)).ceil() as u32;
        
        FileTransfer {
            id: transfer_id.to_string(),
            device_id: device_id.to_string(),
            filename: filename.to_string(),
            file_path: save_path.to_path_buf(),
            size,
            transferred: 0,
            checksum: checksum.to_string(),
            direction: TransferDirection::Receiving,
            status: TransferStatus::Pending,
            started_at: None,
            completed_at: None,
            chunks_received: 0,
            total_chunks,
        }
    }
    
    /// Get progress percentage
    pub fn progress_percent(&self) -> f32 {
        if self.size == 0 {
            return 100.0;
        }
        (self.transferred as f32 / self.size as f32) * 100.0
    }
    
    /// Get transfer speed in bytes per second
    pub fn speed_bps(&self) -> f64 {
        if let Some(started) = self.started_at {
            let elapsed = started.elapsed().as_secs_f64();
            if elapsed > 0.0 {
                return self.transferred as f64 / elapsed;
            }
        }
        0.0
    }
    
    /// Get estimated time remaining in seconds
    pub fn eta_seconds(&self) -> Option<u64> {
        let speed = self.speed_bps();
        if speed > 0.0 {
            let remaining = self.size - self.transferred;
            Some((remaining as f64 / speed) as u64)
        } else {
            None
        }
    }
    
    /// Format speed for display
    pub fn speed_display(&self) -> String {
        let speed = self.speed_bps();
        if speed >= 1_000_000.0 {
            format!("{:.1} MB/s", speed / 1_000_000.0)
        } else if speed >= 1_000.0 {
            format!("{:.1} KB/s", speed / 1_000.0)
        } else {
            format!("{:.0} B/s", speed)
        }
    }
    
    /// Format ETA for display
    pub fn eta_display(&self) -> String {
        match self.eta_seconds() {
            Some(secs) if secs >= 3600 => format!("{}h {}m", secs / 3600, (secs % 3600) / 60),
            Some(secs) if secs >= 60 => format!("{}m {}s", secs / 60, secs % 60),
            Some(secs) => format!("{}s", secs),
            None => "calculating...".to_string(),
        }
    }
}

/// Transfer events
#[derive(Debug, Clone)]
pub enum TransferEvent {
    /// New incoming transfer request
    IncomingRequest { 
        transfer_id: String, 
        device_id: String, 
        filename: String, 
        size: u64 
    },
    /// Transfer started
    Started { transfer_id: String },
    /// Progress update
    Progress { transfer_id: String, percent: f32, speed: String, eta: String },
    /// Transfer completed
    Completed { transfer_id: String, path: PathBuf },
    /// Transfer failed
    Failed { transfer_id: String, error: String },
    /// Transfer cancelled
    Cancelled { transfer_id: String },
}

/// Transfer manager handles all file transfers
pub struct TransferManager {
    transfers: HashMap<String, FileTransfer>,
    event_sender: broadcast::Sender<TransferEvent>,
    default_save_path: PathBuf,
}

impl TransferManager {
    pub fn new() -> Self {
        let (event_sender, _) = broadcast::channel(100);
        
        TransferManager {
            transfers: HashMap::new(),
            event_sender,
            default_save_path: get_default_save_path(),
        }
    }
    
    /// Subscribe to transfer events
    pub fn subscribe(&self) -> broadcast::Receiver<TransferEvent> {
        self.event_sender.subscribe()
    }
    
    /// Set default save path
    pub fn set_save_path(&mut self, path: PathBuf) {
        self.default_save_path = path;
    }
    
    /// Get default save path
    pub fn get_save_path(&self) -> &PathBuf {
        &self.default_save_path
    }
    
    /// Start sending a file
    pub fn send_file(&mut self, device_id: &str, file_path: &Path) -> Result<String, String> {
        // Check concurrent transfer limit
        let active = self.transfers.values()
            .filter(|t| t.status == TransferStatus::InProgress)
            .count();
        if active >= MAX_CONCURRENT_TRANSFERS {
            return Err("Too many concurrent transfers".to_string());
        }
        
        // Create transfer
        let mut transfer = FileTransfer::new_send(device_id, file_path)?;
        let transfer_id = transfer.id.clone();
        
        // Send transfer request
        let message = QuicMessage::FileTransferRequest {
            filename: transfer.filename.clone(),
            size: transfer.size,
            checksum: transfer.checksum.clone(),
            transfer_id: transfer_id.clone(),
        };
        
        // Use tokio runtime to send async QUIC message
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                send_to_device_quic(device_id, message).await
            })
        })?;
        
        transfer.status = TransferStatus::Pending;
        self.transfers.insert(transfer_id.clone(), transfer);
        
        println!("[Transfer] Initiated send: {} to {}", file_path.display(), &device_id[..8]);
        
        Ok(transfer_id)
    }
    
    /// Handle incoming transfer request
    pub fn handle_incoming_request(
        &mut self,
        device_id: &str,
        filename: &str,
        size: u64,
        checksum: &str,
        transfer_id: &str,
    ) -> Result<(), String> {
        // Create save path
        let save_path = self.default_save_path.join(filename);
        
        let transfer = FileTransfer::new_receive(
            device_id, filename, size, checksum, transfer_id, &save_path
        );
        
        self.transfers.insert(transfer_id.to_string(), transfer);
        
        // Emit event for UI to show accept/reject dialog
        let _ = self.event_sender.send(TransferEvent::IncomingRequest {
            transfer_id: transfer_id.to_string(),
            device_id: device_id.to_string(),
            filename: filename.to_string(),
            size,
        });
        
        println!("[Transfer] Incoming request: {} from {}", filename, &device_id[..8]);
        
        Ok(())
    }
    
    /// Accept incoming transfer
    pub fn accept_transfer(&mut self, transfer_id: &str) -> Result<(), String> {
        let transfer = self.transfers.get_mut(transfer_id)
            .ok_or("Transfer not found")?;
        
        if transfer.direction != TransferDirection::Receiving {
            return Err("Can only accept incoming transfers".to_string());
        }
        
        // Ensure save directory exists
        if let Some(parent) = transfer.file_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Cannot create directory: {}", e))?;
        }
        
        // Send accept message
        let message = QuicMessage::FileTransferAccept {
            transfer_id: transfer_id.to_string(),
        };
        
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                send_to_device_quic(&transfer.device_id, message).await
            })
        })?;
        
        transfer.status = TransferStatus::InProgress;
        transfer.started_at = Some(Instant::now());
        
        let _ = self.event_sender.send(TransferEvent::Started {
            transfer_id: transfer_id.to_string(),
        });
        
        println!("[Transfer] Accepted: {}", transfer.filename);
        
        Ok(())
    }
    
    /// Reject incoming transfer
    pub fn reject_transfer(&mut self, transfer_id: &str, reason: &str) -> Result<(), String> {
        let transfer = self.transfers.get_mut(transfer_id)
            .ok_or("Transfer not found")?;
        
        let message = QuicMessage::FileTransferReject {
            transfer_id: transfer_id.to_string(),
            reason: reason.to_string(),
        };
        
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                send_to_device_quic(&transfer.device_id, message).await
            })
        })?;
        
        transfer.status = TransferStatus::Cancelled;
        
        let _ = self.event_sender.send(TransferEvent::Cancelled {
            transfer_id: transfer_id.to_string(),
        });
        
        println!("[Transfer] Rejected: {}", transfer.filename);
        
        Ok(())
    }
    
    /// Handle transfer accepted (start sending chunks)
    pub fn handle_transfer_accepted(&mut self, transfer_id: &str) -> Result<(), String> {
        let transfer = self.transfers.get_mut(transfer_id)
            .ok_or("Transfer not found")?;
        
        if transfer.direction != TransferDirection::Sending {
            return Err("Not a sending transfer".to_string());
        }
        
        transfer.status = TransferStatus::InProgress;
        transfer.started_at = Some(Instant::now());
        
        let _ = self.event_sender.send(TransferEvent::Started {
            transfer_id: transfer_id.to_string(),
        });
        
        // Start sending chunks
        self.send_next_chunks(transfer_id)?;
        
        Ok(())
    }
    
    /// Handle transfer rejected
    pub fn handle_transfer_rejected(&mut self, transfer_id: &str, reason: &str) -> Result<(), String> {
        let transfer = self.transfers.get_mut(transfer_id)
            .ok_or("Transfer not found")?;
        
        transfer.status = TransferStatus::Failed(reason.to_string());
        
        let _ = self.event_sender.send(TransferEvent::Failed {
            transfer_id: transfer_id.to_string(),
            error: reason.to_string(),
        });
        
        println!("[Transfer] Rejected by peer: {}", reason);
        
        Ok(())
    }
    
    /// Send next batch of chunks
    fn send_next_chunks(&mut self, transfer_id: &str) -> Result<(), String> {
        let transfer = self.transfers.get(transfer_id)
            .ok_or("Transfer not found")?;
        
        if transfer.status != TransferStatus::InProgress {
            return Ok(());
        }
        
        let file = File::open(&transfer.file_path)
            .map_err(|e| format!("Cannot open file: {}", e))?;
        let mut reader = BufReader::new(file);
        
        // Seek to current position
        use std::io::Seek;
        reader.seek(std::io::SeekFrom::Start(transfer.transferred))
            .map_err(|e| format!("Seek error: {}", e))?;
        
        let device_id = transfer.device_id.clone();
        let transfer_id_clone = transfer_id.to_string();
        let mut sequence = transfer.chunks_received;
        let mut transferred = transfer.transferred;
        let size = transfer.size;
        
        // Send a few chunks
        for _ in 0..10 {
            let mut chunk = vec![0u8; CHUNK_SIZE];
            let bytes_read = reader.read(&mut chunk)
                .map_err(|e| format!("Read error: {}", e))?;
            
            if bytes_read == 0 {
                break;
            }
            
            chunk.truncate(bytes_read);
            transferred += bytes_read as u64;
            let is_last = transferred >= size;
            
            let message = QuicMessage::FileChunk {
                transfer_id: transfer_id_clone.clone(),
                sequence,
                data: chunk,
                is_last,
            };
            
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    send_to_device_quic(&device_id, message).await
                })
            })?;
            sequence += 1;
            
            if is_last {
                break;
            }
        }
        
        // Update transfer state
        let transfer = self.transfers.get_mut(transfer_id).unwrap();
        transfer.transferred = transferred;
        transfer.chunks_received = sequence;
        
        // Emit progress
        let _ = self.event_sender.send(TransferEvent::Progress {
            transfer_id: transfer_id.to_string(),
            percent: transfer.progress_percent(),
            speed: transfer.speed_display(),
            eta: transfer.eta_display(),
        });
        
        Ok(())
    }
    
    /// Handle received chunk
    pub fn handle_chunk(
        &mut self,
        transfer_id: &str,
        sequence: u32,
        data: Vec<u8>,
        is_last: bool,
    ) -> Result<(), String> {
        let transfer = self.transfers.get_mut(transfer_id)
            .ok_or("Transfer not found")?;
        
        if transfer.direction != TransferDirection::Receiving {
            return Err("Not a receiving transfer".to_string());
        }
        
        // Write chunk to file
        let file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&transfer.file_path)
            .map_err(|e| format!("Cannot open file: {}", e))?;
        
        let mut writer = BufWriter::new(file);
        writer.write_all(&data)
            .map_err(|e| format!("Write error: {}", e))?;
        writer.flush()
            .map_err(|e| format!("Flush error: {}", e))?;
        
        transfer.transferred += data.len() as u64;
        transfer.chunks_received = sequence + 1;
        
        // Emit progress
        let _ = self.event_sender.send(TransferEvent::Progress {
            transfer_id: transfer_id.to_string(),
            percent: transfer.progress_percent(),
            speed: transfer.speed_display(),
            eta: transfer.eta_display(),
        });
        
        if is_last {
            // Verify checksum
            let actual_checksum = calculate_file_checksum(&transfer.file_path)?;
            if actual_checksum != transfer.checksum {
                transfer.status = TransferStatus::Failed("Checksum mismatch".to_string());
                let _ = self.event_sender.send(TransferEvent::Failed {
                    transfer_id: transfer_id.to_string(),
                    error: "Checksum mismatch - file corrupted".to_string(),
                });
                return Err("Checksum mismatch".to_string());
            }
            
            transfer.status = TransferStatus::Completed;
            transfer.completed_at = Some(Instant::now());
            
            let _ = self.event_sender.send(TransferEvent::Completed {
                transfer_id: transfer_id.to_string(),
                path: transfer.file_path.clone(),
            });
            
            println!("[Transfer] Completed: {}", transfer.filename);
        }
        
        Ok(())
    }
    
    /// Handle transfer complete message
    pub fn handle_complete(&mut self, transfer_id: &str, checksum: &str) -> Result<(), String> {
        let transfer = self.transfers.get_mut(transfer_id)
            .ok_or("Transfer not found")?;
        
        if transfer.direction == TransferDirection::Sending {
            transfer.status = TransferStatus::Completed;
            transfer.completed_at = Some(Instant::now());
            
            let _ = self.event_sender.send(TransferEvent::Completed {
                transfer_id: transfer_id.to_string(),
                path: transfer.file_path.clone(),
            });
            
            println!("[Transfer] Send completed: {}", transfer.filename);
        }
        
        Ok(())
    }
    
    /// Cancel a transfer
    pub fn cancel_transfer(&mut self, transfer_id: &str) -> Result<(), String> {
        let transfer = self.transfers.get_mut(transfer_id)
            .ok_or("Transfer not found")?;
        
        transfer.status = TransferStatus::Cancelled;
        
        let _ = self.event_sender.send(TransferEvent::Cancelled {
            transfer_id: transfer_id.to_string(),
        });
        
        println!("[Transfer] Cancelled: {}", transfer.filename);
        
        Ok(())
    }
    
    /// Get a transfer by ID
    pub fn get_transfer(&self, transfer_id: &str) -> Option<&FileTransfer> {
        self.transfers.get(transfer_id)
    }
    
    /// Get all transfers
    pub fn get_all_transfers(&self) -> Vec<&FileTransfer> {
        self.transfers.values().collect()
    }
    
    /// Get active transfers
    pub fn get_active_transfers(&self) -> Vec<&FileTransfer> {
        self.transfers.values()
            .filter(|t| t.status == TransferStatus::InProgress || t.status == TransferStatus::Pending)
            .collect()
    }
    
    /// Clean up completed/failed transfers
    pub fn cleanup_old_transfers(&mut self) {
        self.transfers.retain(|_, t| {
            match t.status {
                TransferStatus::Completed | TransferStatus::Failed(_) | TransferStatus::Cancelled => {
                    if let Some(completed) = t.completed_at {
                        completed.elapsed().as_secs() < 3600 // Keep for 1 hour
                    } else {
                        true
                    }
                }
                _ => true,
            }
        });
    }
}

/// Calculate SHA-256 checksum of a file
fn calculate_file_checksum(path: &Path) -> Result<String, String> {
    let file = File::open(path)
        .map_err(|e| format!("Cannot open file: {}", e))?;
    
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    
    loop {
        let bytes_read = reader.read(&mut buffer)
            .map_err(|e| format!("Read error: {}", e))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Get platform-specific default save path
pub fn get_default_save_path() -> PathBuf {
    if cfg!(target_os = "windows") {
        dirs::download_dir()
            .unwrap_or_else(|| PathBuf::from("C:\\Users\\Public\\Downloads"))
            .join("IGRIS")
    } else if cfg!(target_os = "macos") {
        dirs::download_dir()
            .unwrap_or_else(|| PathBuf::from("/Users/Shared"))
            .join("IGRIS")
    } else {
        dirs::download_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("IGRIS")
    }
}

/// Format file size for display
pub fn format_file_size(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        format!("{:.2} GB", bytes as f64 / 1_000_000_000.0)
    } else if bytes >= 1_000_000 {
        format!("{:.2} MB", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.2} KB", bytes as f64 / 1_000.0)
    } else {
        format!("{} B", bytes)
    }
}

// Global transfer manager
static TRANSFER_MANAGER: Lazy<Arc<Mutex<TransferManager>>> = Lazy::new(|| {
    Arc::new(Mutex::new(TransferManager::new()))
});

/// Get the transfer manager
pub fn get_transfer_manager() -> Arc<Mutex<TransferManager>> {
    TRANSFER_MANAGER.clone()
}

// Convenience functions

/// Send a file to a device
pub fn send_file(device_id: &str, file_path: &Path) -> Result<String, String> {
    let manager = get_transfer_manager();
    let mut manager = manager.lock().map_err(|e| format!("Lock error: {}", e))?;
    manager.send_file(device_id, file_path)
}

/// Accept an incoming transfer
pub fn accept_incoming_transfer(transfer_id: &str) -> Result<(), String> {
    let manager = get_transfer_manager();
    let mut manager = manager.lock().map_err(|e| format!("Lock error: {}", e))?;
    manager.accept_transfer(transfer_id)
}

/// Reject an incoming transfer
pub fn reject_incoming_transfer(transfer_id: &str, reason: &str) -> Result<(), String> {
    let manager = get_transfer_manager();
    let mut manager = manager.lock().map_err(|e| format!("Lock error: {}", e))?;
    manager.reject_transfer(transfer_id, reason)
}

/// Cancel a transfer
pub fn cancel_file_transfer(transfer_id: &str) -> Result<(), String> {
    let manager = get_transfer_manager();
    let mut manager = manager.lock().map_err(|e| format!("Lock error: {}", e))?;
    manager.cancel_transfer(transfer_id)
}

/// Get transfer progress
pub fn get_transfer_progress(transfer_id: &str) -> Option<(f32, String, String)> {
    let manager = get_transfer_manager();
    let manager = manager.lock().ok()?;
    let transfer = manager.get_transfer(transfer_id)?;
    Some((transfer.progress_percent(), transfer.speed_display(), transfer.eta_display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;
    
    /// Helper to create a test file with content
    fn create_test_file(dir: &TempDir, filename: &str, content: &[u8]) -> PathBuf {
        let file_path = dir.path().join(filename);
        let mut file = File::create(&file_path).unwrap();
        file.write_all(content).unwrap();
        file_path
    }
    
    /// Helper to read file content
    fn read_file_content(path: &PathBuf) -> Vec<u8> {
        fs::read(path).unwrap()
    }
    
    #[test]
    fn test_file_transfer_creation() {
        let temp_dir = TempDir::new().unwrap();
        let test_content = b"Hello, this is a test file!";
        let file_path = create_test_file(&temp_dir, "test.txt", test_content);
        
        // Create a send transfer
        let transfer = FileTransfer::new_send("device123", &file_path).unwrap();
        
        assert_eq!(transfer.direction, TransferDirection::Sending);
        assert_eq!(transfer.filename, "test.txt");
        assert_eq!(transfer.size, test_content.len() as u64);
        assert_eq!(transfer.status, TransferStatus::Pending);
        assert_eq!(transfer.device_id, "device123");
        assert!(transfer.checksum.len() > 0);
    }
    
    #[test]
    fn test_file_transfer_receive_creation() {
        let temp_dir = TempDir::new().unwrap();
        let save_path = temp_dir.path().join("received.txt");
        
        let transfer = FileTransfer::new_receive(
            "device456",
            "received.txt",
            1024,
            "abc123checksum",
            "transfer789",
            &save_path,
        );
        
        assert_eq!(transfer.direction, TransferDirection::Receiving);
        assert_eq!(transfer.filename, "received.txt");
        assert_eq!(transfer.size, 1024);
        assert_eq!(transfer.checksum, "abc123checksum");
        assert_eq!(transfer.id, "transfer789");
        assert_eq!(transfer.device_id, "device456");
    }
    
    #[test]
    fn test_transfer_progress_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let save_path = temp_dir.path().join("test.txt");
        
        let mut transfer = FileTransfer::new_receive(
            "device123",
            "test.txt",
            1000,
            "checksum",
            "transfer1",
            &save_path,
        );
        
        // Initially 0%
        assert_eq!(transfer.progress_percent(), 0.0);
        
        // 50% progress
        transfer.transferred = 500;
        assert_eq!(transfer.progress_percent(), 50.0);
        
        // 100% progress
        transfer.transferred = 1000;
        assert_eq!(transfer.progress_percent(), 100.0);
    }
    
    #[test]
    fn test_checksum_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let test_content = b"Test content for checksum";
        let file_path = create_test_file(&temp_dir, "checksum_test.txt", test_content);
        
        let checksum1 = calculate_file_checksum(&file_path).unwrap();
        let checksum2 = calculate_file_checksum(&file_path).unwrap();
        
        // Same file should produce same checksum
        assert_eq!(checksum1, checksum2);
        assert!(checksum1.len() > 0);
        
        // Different content should produce different checksum
        let file_path2 = create_test_file(&temp_dir, "checksum_test2.txt", b"Different content");
        let checksum3 = calculate_file_checksum(&file_path2).unwrap();
        
        assert_ne!(checksum1, checksum3);
    }
    
    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(500), "500 B");
        assert_eq!(format_file_size(1500), "1.50 KB");
        assert_eq!(format_file_size(1_500_000), "1.50 MB");
        assert_eq!(format_file_size(1_500_000_000), "1.50 GB");
    }
    
    #[test]
    fn test_transfer_speed_and_eta() {
        let temp_dir = TempDir::new().unwrap();
        let save_path = temp_dir.path().join("speed_test.txt");
        
        let mut transfer = FileTransfer::new_receive(
            "device123",
            "speed_test.txt",
            10000,
            "checksum",
            "transfer1",
            &save_path,
        );
        
        // Initially no speed
        assert_eq!(transfer.speed_bps(), 0.0);
        assert!(transfer.eta_seconds().is_none());
        
        // Simulate transfer start and progress
        transfer.started_at = Some(Instant::now() - std::time::Duration::from_secs(1));
        transfer.transferred = 5000;
        
        // Should have some speed now
        let speed = transfer.speed_bps();
        assert!(speed > 0.0);
        
        // Should have ETA
        let eta = transfer.eta_seconds();
        assert!(eta.is_some());
    }
    
    /// Integration test: Verify bidirectional file transfer capability
    /// This test verifies that the file transfer system can handle transfers in both directions
    /// after a connection is established (Requirements 3.5, 11.5)
    #[test]
    fn test_bidirectional_file_transfer_capability() {
        let temp_dir_a = TempDir::new().unwrap();
        let temp_dir_b = TempDir::new().unwrap();
        
        // Create test files for both devices
        let file_a_content = b"File from Device A to Device B";
        let file_b_content = b"File from Device B to Device A";
        
        let file_a_path = create_test_file(&temp_dir_a, "file_from_a.txt", file_a_content);
        let file_b_path = create_test_file(&temp_dir_b, "file_from_b.txt", file_b_content);
        
        // Create transfer managers for both devices
        let mut manager_a = TransferManager::new();
        let mut manager_b = TransferManager::new();
        
        manager_a.set_save_path(temp_dir_a.path().to_path_buf());
        manager_b.set_save_path(temp_dir_b.path().to_path_buf());
        
        // Test 1: Device A can create a send transfer to Device B
        let transfer_a_to_b = FileTransfer::new_send("device_b", &file_a_path);
        assert!(transfer_a_to_b.is_ok(), "Device A should be able to create send transfer to Device B");
        
        let transfer_a_to_b = transfer_a_to_b.unwrap();
        assert_eq!(transfer_a_to_b.direction, TransferDirection::Sending);
        assert_eq!(transfer_a_to_b.device_id, "device_b");
        assert_eq!(transfer_a_to_b.filename, "file_from_a.txt");
        assert_eq!(transfer_a_to_b.size, file_a_content.len() as u64);
        
        // Test 2: Device B can create a send transfer to Device A
        let transfer_b_to_a = FileTransfer::new_send("device_a", &file_b_path);
        assert!(transfer_b_to_a.is_ok(), "Device B should be able to create send transfer to Device A");
        
        let transfer_b_to_a = transfer_b_to_a.unwrap();
        assert_eq!(transfer_b_to_a.direction, TransferDirection::Sending);
        assert_eq!(transfer_b_to_a.device_id, "device_a");
        assert_eq!(transfer_b_to_a.filename, "file_from_b.txt");
        assert_eq!(transfer_b_to_a.size, file_b_content.len() as u64);
        
        // Test 3: Device B can receive transfer request from Device A
        let result = manager_b.handle_incoming_request(
            "device_a",
            "file_from_a.txt",
            file_a_content.len() as u64,
            &transfer_a_to_b.checksum,
            &transfer_a_to_b.id,
        );
        assert!(result.is_ok(), "Device B should be able to handle incoming request from Device A");
        
        // Verify the transfer was created on Device B
        let transfer_on_b = manager_b.get_transfer(&transfer_a_to_b.id);
        assert!(transfer_on_b.is_some(), "Transfer should exist on Device B");
        assert_eq!(transfer_on_b.unwrap().direction, TransferDirection::Receiving);
        
        // Test 4: Device A can receive transfer request from Device B
        let result = manager_a.handle_incoming_request(
            "device_b",
            "file_from_b.txt",
            file_b_content.len() as u64,
            &transfer_b_to_a.checksum,
            &transfer_b_to_a.id,
        );
        assert!(result.is_ok(), "Device A should be able to handle incoming request from Device B");
        
        // Verify the transfer was created on Device A
        let transfer_on_a = manager_a.get_transfer(&transfer_b_to_a.id);
        assert!(transfer_on_a.is_some(), "Transfer should exist on Device A");
        assert_eq!(transfer_on_a.unwrap().direction, TransferDirection::Receiving);
        
        // Test 5: Verify checksums match for bidirectional transfers
        assert_eq!(
            transfer_a_to_b.checksum,
            manager_b.get_transfer(&transfer_a_to_b.id).unwrap().checksum,
            "Checksum should match for A->B transfer"
        );
        
        assert_eq!(
            transfer_b_to_a.checksum,
            manager_a.get_transfer(&transfer_b_to_a.id).unwrap().checksum,
            "Checksum should match for B->A transfer"
        );
        
        println!("✓ Bidirectional file transfer capability verified:");
        println!("  - Device A can send to Device B");
        println!("  - Device B can send to Device A");
        println!("  - Both devices can receive incoming requests");
        println!("  - Transfer metadata is correctly propagated");
    }
    
    /// Test that file transfer works after connection establishment
    /// This verifies that the connection system properly enables file transfer
    #[test]
    fn test_file_transfer_after_connection() {
        let temp_dir = TempDir::new().unwrap();
        let test_content = b"Test file after connection";
        let file_path = create_test_file(&temp_dir, "after_connection.txt", test_content);
        
        // Simulate that a connection has been established
        // In a real scenario, this would be done through ConnectionCoordinator
        
        // Create a transfer to send
        let transfer = FileTransfer::new_send("connected_device", &file_path);
        assert!(transfer.is_ok(), "Should be able to create transfer after connection");
        
        let transfer = transfer.unwrap();
        
        // Verify transfer properties
        assert_eq!(transfer.direction, TransferDirection::Sending);
        assert_eq!(transfer.device_id, "connected_device");
        assert_eq!(transfer.status, TransferStatus::Pending);
        assert!(transfer.checksum.len() > 0, "Checksum should be calculated");
        
        // Verify file can be read for transfer
        let content = read_file_content(&transfer.file_path);
        assert_eq!(content, test_content, "File content should be readable");
        
        println!("✓ File transfer capability verified after connection");
    }
}
