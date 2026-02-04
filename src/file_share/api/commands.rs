// Commands for file sharing operations

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileShareCommand {
    /// Start discovery
    StartDiscovery,
    
    /// Stop discovery
    StopDiscovery,
    
    /// Send files to a device
    SendFiles {
        device_id: String,
        file_paths: Vec<String>,
    },
    
    /// Accept incoming transfer
    AcceptTransfer {
        session_id: String,
    },
    
    /// Reject incoming transfer
    RejectTransfer {
        session_id: String,
    },
    
    /// Cancel ongoing transfer
    CancelTransfer {
        session_id: String,
    },
    
    /// Get transfer progress
    GetProgress {
        session_id: String,
    },
    
    /// Get list of discovered devices
    GetDevices,
    
    /// Refresh device list
    RefreshDevices,
}
