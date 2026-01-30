// src/file_share/protocol.rs
// Protocol definitions for file sharing communication

use serde::{Serialize, Deserialize};
use super::*;

/// Protocol version
pub const PROTOCOL_VERSION: u32 = 1;

/// Protocol messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolMessage {
    // Handshake
    Hello {
        version: u32,
        device_info: DeviceInfo,
    },
    HelloAck {
        version: u32,
        device_info: DeviceInfo,
    },
    
    // File transfer
    FileOffer {
        transfer_id: String,
        file_info: FileInfo,
    },
    FileAccept {
        transfer_id: String,
    },
    FileReject {
        transfer_id: String,
        reason: String,
    },
    FileData {
        transfer_id: String,
        chunk_index: u64,
        data: Vec<u8>,
    },
    FileComplete {
        transfer_id: String,
        checksum: String,
    },
    
    // Transfer control
    Pause {
        transfer_id: String,
    },
    Resume {
        transfer_id: String,
    },
    Cancel {
        transfer_id: String,
    },
    
    // Status
    Progress {
        transfer_id: String,
        bytes_transferred: u64,
    },
    Error {
        transfer_id: String,
        error: String,
    },
    
    // Keep-alive
    Ping,
    Pong,
}

/// Protocol error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolError {
    VersionMismatch,
    InvalidMessage,
    TransferNotFound,
    FileNotFound,
    PermissionDenied,
    NetworkError,
    ChecksumMismatch,
    Timeout,
    Unknown(String),
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolError::VersionMismatch => write!(f, "Protocol version mismatch"),
            ProtocolError::InvalidMessage => write!(f, "Invalid protocol message"),
            ProtocolError::TransferNotFound => write!(f, "Transfer not found"),
            ProtocolError::FileNotFound => write!(f, "File not found"),
            ProtocolError::PermissionDenied => write!(f, "Permission denied"),
            ProtocolError::NetworkError => write!(f, "Network error"),
            ProtocolError::ChecksumMismatch => write!(f, "Checksum mismatch"),
            ProtocolError::Timeout => write!(f, "Operation timeout"),
            ProtocolError::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for ProtocolError {}
