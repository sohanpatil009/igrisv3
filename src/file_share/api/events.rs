// Events emitted by file sharing system

use crate::file_share::discovery::Device;
use crate::file_share::transfer::TransferProgress;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileShareEvent {
    /// New device discovered
    DeviceDiscovered(Device),
    
    /// Device lost (timeout)
    DeviceLost(String), // device_id
    
    /// Incoming transfer request
    TransferRequest {
        session_id: String,
        from_device: String,
        files: Vec<String>,
        total_size: u64,
    },
    
    /// Transfer progress update
    TransferProgress(TransferProgress),
    
    /// Transfer completed
    TransferCompleted {
        session_id: String,
    },
    
    /// Transfer failed
    TransferFailed {
        session_id: String,
        error: String,
    },
    
    /// Transfer cancelled
    TransferCancelled {
        session_id: String,
    },
}
