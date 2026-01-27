// src/file_share/manager.rs - Central File Share Manager
// Coordinates all file sharing services: discovery, trust, bridge, transfer

use std::sync::{Arc, Mutex};
use std::path::{Path, PathBuf};
use tokio::sync::broadcast;
use once_cell::sync::Lazy;

use super::config::{DeviceIdentity, TrustedDevice, get_or_create_device_identity};
use super::crypto::{get_certificate_manager, initialize_certificate};
use super::discovery::{
    DiscoveryService, DiscoveredDevice, DiscoveryEvent,
    get_discovery_service, start_discovery, stop_discovery, get_discovered_devices,
};
use super::trust::{
    TrustManager, TrustResult,
    get_trust_manager, establish_trust, check_rate_limit,
    add_trusted, remove_trusted, is_device_trusted, get_all_trusted,
};
use super::quic_bridge::{
    is_connected_to_quic as is_connected_to,
    connect_to_device_quic,
    send_to_device_quic,
};
use super::transfer::{
    TransferManager, FileTransfer, TransferEvent, TransferStatus,
    get_transfer_manager, send_file, accept_incoming_transfer,
    reject_incoming_transfer, cancel_file_transfer, get_transfer_progress,
};

/// Events emitted by the FileShareManager
#[derive(Debug, Clone)]
pub enum FileShareEvent {
    // Lifecycle
    Initialized,
    ShuttingDown,
    
    // Discovery
    DeviceDiscovered(DiscoveredDevice),
    DeviceLost(String),
    
    // Trust
    VerificationCodeGenerated { device_id: String, code: String, expires_in: u64 },
    VerificationReceived { device_id: String },
    TrustEstablished { device_id: String, label: String },
    TrustRemoved { device_id: String },
    
    // Connection
    Connected { device_id: String, label: String },
    Disconnected { device_id: String, reason: String },
    ConnectionFailed { device_id: String, error: String },
    
    // Transfer
    TransferStarted { transfer_id: String, file_name: String, device_label: String },
    TransferProgress { transfer_id: String, percent: f32, speed: String, eta: String },
    TransferCompleted { transfer_id: String, file_name: String },
    TransferFailed { transfer_id: String, error: String },
    IncomingTransfer { transfer_id: String, file_name: String, size: u64, from_device: String },
}

/// Current state of the file share system
#[derive(Debug, Clone, PartialEq)]
pub enum FileShareState {
    Uninitialized,
    Initializing,
    Ready,
    Scanning,
    Error(String),
    ShuttingDown,
}

/// Central manager for all file sharing functionality
pub struct FileShareManager {
    state: FileShareState,
    device_identity: Option<DeviceIdentity>,
    event_tx: broadcast::Sender<FileShareEvent>,
    save_path: PathBuf,
    auto_accept_from_trusted: bool,
}

impl FileShareManager {
    /// Create a new FileShareManager
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(100);
        
        Self {
            state: FileShareState::Uninitialized,
            device_identity: None,
            event_tx,
            save_path: super::transfer::get_default_save_path(),
            auto_accept_from_trusted: false,
        }
    }
    
    /// Subscribe to file share events
    pub fn subscribe(&self) -> broadcast::Receiver<FileShareEvent> {
        self.event_tx.subscribe()
    }
    
    /// Get current state
    pub fn state(&self) -> &FileShareState {
        &self.state
    }
    
    /// Initialize all services
    pub async fn initialize(&mut self) -> Result<(), String> {
        if self.state != FileShareState::Uninitialized {
            return Err("Already initialized".to_string());
        }
        
        self.state = FileShareState::Initializing;
        println!("[FileShare] Initializing file share services...");
        
        // Step 0: Initialize rustls crypto provider (CRITICAL for QUIC)
        let _ = rustls::crypto::ring::default_provider().install_default();
        println!("[FileShare] Rustls crypto provider initialized");
        
        // Step 1: Get or create device identity
        self.device_identity = Some(get_or_create_device_identity()?);
        println!("[FileShare] Device identity ready");
        
        // Step 2: Initialize QUIC certificate (replaces old TLS)
        super::quic_crypto::initialize_quic_crypto()?;
        println!("[FileShare] QUIC certificate ready");
        
        // Step 3: Initialize QUIC bridge (replaces old TCP bridge)
        let config = super::config::load_config()?;
        super::quic_bridge::initialize_quic_bridge(config.bridge_port).await?;
        println!("[FileShare] QUIC bridge ready");
        
        // Step 4: Other services
        let _ = get_trust_manager();
        println!("[FileShare] Trust manager ready");
        
        let _ = get_transfer_manager();
        println!("[FileShare] Transfer manager ready");
        
        self.state = FileShareState::Ready;
        let _ = self.event_tx.send(FileShareEvent::Initialized);
        
        println!("[FileShare] All services initialized successfully");
        Ok(())
    }

    
    /// Start scanning for nearby devices
    pub async fn start_scanning(&mut self) -> Result<(), String> {
        if self.state != FileShareState::Ready && self.state != FileShareState::Scanning {
            return Err(format!("Cannot scan in state: {:?}", self.state));
        }
        
        self.state = FileShareState::Scanning;
        start_discovery().await?;
        
        println!("[FileShare] Device scanning started");
        Ok(())
    }
    
    /// Stop scanning for devices
    pub fn stop_scanning(&mut self) -> Result<(), String> {
        stop_discovery()?;
        self.state = FileShareState::Ready;
        
        println!("[FileShare] Device scanning stopped");
        Ok(())
    }
    
    /// Get all discovered devices
    pub fn get_nearby_devices(&self) -> Result<Vec<DiscoveredDevice>, String> {
        get_discovered_devices()
    }
    
    /// Get all trusted devices
    pub fn get_trusted_devices(&self) -> Result<Vec<TrustedDevice>, String> {
        get_all_trusted()
    }
    
    /// Check if a device is trusted
    pub fn is_trusted(&self, device_id: &str) -> Result<bool, String> {
        is_device_trusted(device_id)
    }
    
    // ═══════════════════════════════════════════════════════
    // TRUST MANAGEMENT
    // ═══════════════════════════════════════════════════════
    
    // NOTE: The following methods are deprecated and will be replaced by ConnectionCoordinator
    // in task 6. They use the old verification code system that has been removed from TrustManager.
    
    /*
    /// Generate a verification code for pairing with a device
    /// DEPRECATED: Use ConnectionCoordinator::generate_my_code() instead
    pub fn generate_pairing_code(&self, device_id: &str) -> Result<VerificationCode, String> {
        let code = generate_verification_code(device_id)?;
        
        let _ = self.event_tx.send(FileShareEvent::VerificationCodeGenerated {
            device_id: device_id.to_string(),
            code: code.code.clone(),
            expires_in: code.remaining_seconds(),
        });
        
        Ok(code)
    }
    
    /// Verify a code entered by user (when receiving pairing request)
    /// DEPRECATED: Use ConnectionCoordinator::connect_with_code() instead
    pub fn verify_pairing_code(&self, device_id: &str, code: &str) -> Result<TrustResult, String> {
        verify_code(device_id, code)
    }
    */
    
    /// Add a device to trusted list after successful verification
    pub fn trust_device(&self, device: &DiscoveredDevice, cert_fingerprint: &str) -> Result<(), String> {
        add_trusted(device, cert_fingerprint)?;
        
        let _ = self.event_tx.send(FileShareEvent::TrustEstablished {
            device_id: device.id.clone(),
            label: device.label.clone(),
        });
        
        Ok(())
    }
    
    /// Remove a device from trusted list
    pub fn untrust_device(&self, device_id: &str) -> Result<bool, String> {
        let removed = remove_trusted(device_id)?;
        
        if removed {
            let _ = self.event_tx.send(FileShareEvent::TrustRemoved {
                device_id: device_id.to_string(),
            });
        }
        
        Ok(removed)
    }
    
    // ═══════════════════════════════════════════════════════
    // CONNECTION MANAGEMENT
    // ═══════════════════════════════════════════════════════
    
    /// Connect to a discovered device
    pub fn connect(&self, device: &DiscoveredDevice) -> Result<(), String> {
        // Check if already connected
        if is_connected_to(&device.id)? {
            return Ok(());
        }
        
        // Use async runtime to connect via QUIC
        let device_clone = device.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                connect_to_device_quic(&device_clone).await
            })
        })?;
        
        let _ = self.event_tx.send(FileShareEvent::Connected {
            device_id: device.id.clone(),
            label: device.label.clone(),
        });
        
        Ok(())
    }
    
    /// Disconnect from a device
    pub fn disconnect(&self, device_id: &str, reason: &str) -> Result<(), String> {
        // QUIC connections are managed by QuicBridgeManager
        let manager_lock = super::quic_bridge::get_quic_bridge_manager()?;
        let mut manager = manager_lock.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(ref mut mgr) = *manager {
            mgr.disconnect(device_id, reason)?;
        }
        
        let _ = self.event_tx.send(FileShareEvent::Disconnected {
            device_id: device_id.to_string(),
            reason: reason.to_string(),
        });
        
        Ok(())
    }
    
    /// Check if connected to a device
    pub fn is_connected(&self, device_id: &str) -> Result<bool, String> {
        is_connected_to(device_id)
    }
    
    /// Get all connected device IDs
    pub fn get_connected_devices(&self) -> Result<Vec<String>, String> {
        let manager_lock = super::quic_bridge::get_quic_bridge_manager()?;
        let manager = manager_lock.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(ref mgr) = *manager {
            Ok(mgr.get_connected_devices())
        } else {
            Ok(Vec::new())
        }
    }

    
    // ═══════════════════════════════════════════════════════
    // FILE TRANSFER
    // ═══════════════════════════════════════════════════════
    
    /// Send a file to a connected device
    pub fn send_file_to(&self, device_id: &str, file_path: &Path) -> Result<String, String> {
        // Verify connection
        if !is_connected_to(device_id)? {
            return Err(format!("Not connected to device: {}", device_id));
        }
        
        let transfer_id = send_file(device_id, file_path)?;
        
        let file_name = file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        
        let _ = self.event_tx.send(FileShareEvent::TransferStarted {
            transfer_id: transfer_id.clone(),
            file_name,
            device_label: device_id.to_string(), // TODO: Get actual label
        });
        
        Ok(transfer_id)
    }
    
    /// Accept an incoming file transfer
    pub fn accept_transfer(&self, transfer_id: &str) -> Result<(), String> {
        accept_incoming_transfer(transfer_id)?;
        Ok(())
    }
    
    /// Reject an incoming file transfer
    pub fn reject_transfer(&self, transfer_id: &str, reason: &str) -> Result<(), String> {
        reject_incoming_transfer(transfer_id, reason)?;
        Ok(())
    }
    
    /// Cancel an ongoing transfer
    pub fn cancel_transfer(&self, transfer_id: &str) -> Result<(), String> {
        cancel_file_transfer(transfer_id)?;
        
        let _ = self.event_tx.send(FileShareEvent::TransferFailed {
            transfer_id: transfer_id.to_string(),
            error: "Cancelled by user".to_string(),
        });
        
        Ok(())
    }
    
    /// Get transfer progress (percent, speed, eta)
    pub fn get_progress(&self, transfer_id: &str) -> Option<(f32, String, String)> {
        get_transfer_progress(transfer_id)
    }
    
    /// Set the default save path for received files
    pub fn set_save_path(&mut self, path: PathBuf) {
        self.save_path = path.clone();
        
        let manager = get_transfer_manager();
        if let Ok(mut m) = manager.lock() {
            m.set_save_path(path);
        };
    }
    
    /// Get the current save path
    pub fn get_save_path(&self) -> &PathBuf {
        &self.save_path
    }
    
    /// Enable/disable auto-accept from trusted devices
    pub fn set_auto_accept(&mut self, enabled: bool) {
        self.auto_accept_from_trusted = enabled;
    }
    
    /// Check if auto-accept is enabled
    pub fn is_auto_accept_enabled(&self) -> bool {
        self.auto_accept_from_trusted
    }
    
    // ═══════════════════════════════════════════════════════
    // LIFECYCLE
    // ═══════════════════════════════════════════════════════
    
    /// Shutdown all services gracefully
    pub fn shutdown(&mut self) -> Result<(), String> {
        self.state = FileShareState::ShuttingDown;
        let _ = self.event_tx.send(FileShareEvent::ShuttingDown);
        
        // Stop discovery
        let _ = stop_discovery();
        
        // Disconnect all devices via QUIC
        if let Ok(manager_lock) = super::quic_bridge::get_quic_bridge_manager() {
            if let Ok(mut manager) = manager_lock.lock() {
                if let Some(ref mut mgr) = *manager {
                    let connected = mgr.get_connected_devices();
                    for device_id in connected {
                        let _ = mgr.disconnect(&device_id, "Shutdown");
                    }
                }
            }
        }
        
        println!("[FileShare] All services shut down");
        self.state = FileShareState::Uninitialized;
        Ok(())
    }
    
    /// Get device identity
    pub fn get_device_identity(&self) -> Option<&DeviceIdentity> {
        self.device_identity.as_ref()
    }
    
    /// Get this device's label (hostname)
    pub fn get_device_label(&self) -> String {
        self.device_identity
            .as_ref()
            .map(|i| i.label.clone())
            .unwrap_or_else(|| "Unknown Device".to_string())
    }
    
    /// Get this device's unique ID
    pub fn get_device_id(&self) -> Option<String> {
        self.device_identity.as_ref().map(|i| i.id.clone())
    }
}

// ═══════════════════════════════════════════════════════
// GLOBAL INSTANCE
// ═══════════════════════════════════════════════════════

static FILE_SHARE_MANAGER: Lazy<Arc<Mutex<FileShareManager>>> = Lazy::new(|| {
    Arc::new(Mutex::new(FileShareManager::new()))
});

/// Get the global FileShareManager instance
pub fn get_file_share_manager() -> Arc<Mutex<FileShareManager>> {
    FILE_SHARE_MANAGER.clone()
}

/// Initialize the file share system
pub async fn initialize_file_share() -> Result<(), String> {
    let manager = get_file_share_manager();
    let mut m = manager.lock().map_err(|e| format!("Lock error: {}", e))?;
    m.initialize().await
}

/// Shutdown the file share system
pub fn shutdown_file_share() -> Result<(), String> {
    let manager = get_file_share_manager();
    let mut m = manager.lock().map_err(|e| format!("Lock error: {}", e))?;
    m.shutdown()
}

/// Quick access: Start scanning for devices
pub async fn scan_for_devices() -> Result<Vec<DiscoveredDevice>, String> {
    let manager = get_file_share_manager();
    {
        let mut m = manager.lock().map_err(|e| format!("Lock error: {}", e))?;
        m.start_scanning().await?;
    }
    
    // Wait a moment for initial discovery
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    
    let m = manager.lock().map_err(|e| format!("Lock error: {}", e))?;
    m.get_nearby_devices()
}

/// Quick access: Send a file to a device
pub fn quick_send_file(device_id: &str, file_path: &Path) -> Result<String, String> {
    let manager = get_file_share_manager();
    let m = manager.lock().map_err(|e| format!("Lock error: {}", e))?;
    m.send_file_to(device_id, file_path)
}
