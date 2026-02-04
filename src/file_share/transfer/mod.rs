// File transfer management

pub mod integrity;
pub mod orchestrator;
pub mod receiver;
pub mod resume;
pub mod sender;

pub use integrity::FileIntegrity;
pub use orchestrator::TransferOrchestrator;
pub use receiver::FileReceiver;
pub use sender::FileSender;

use serde::{Deserialize, Serialize};

/// Transfer status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransferStatus {
    Pending,
    Preparing,
    Transferring,
    Completed,
    Failed(String),
    Cancelled,
}

/// Transfer progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferProgress {
    pub session_id: String,
    pub status: TransferStatus,
    pub total_bytes: u64,
    pub transferred_bytes: u64,
    pub current_file: Option<String>,
    pub files_completed: usize,
    pub files_total: usize,
}

impl TransferProgress {
    pub fn new(session_id: String, total_bytes: u64, files_total: usize) -> Self {
        Self {
            session_id,
            status: TransferStatus::Pending,
            total_bytes,
            transferred_bytes: 0,
            current_file: None,
            files_completed: 0,
            files_total,
        }
    }

    pub fn percentage(&self) -> f64 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        (self.transferred_bytes as f64 / self.total_bytes as f64) * 100.0
    }

    pub fn is_complete(&self) -> bool {
        matches!(self.status, TransferStatus::Completed)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.status, TransferStatus::Failed(_))
    }
}
