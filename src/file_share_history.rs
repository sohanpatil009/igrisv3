// File Share Transfer History
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransferHistoryEntry {
    pub session_id: String,
    pub file_name: String,
    pub file_size: u64,
    pub device_name: String,
    pub device_ip: String,
    pub direction: TransferDirection,
    pub status: TransferStatus,
    pub timestamp: DateTime<Utc>,
    pub error_message: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TransferDirection {
    Sent,
    Received,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TransferStatus {
    Completed,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TransferHistory {
    pub entries: Vec<TransferHistoryEntry>,
}

impl TransferHistory {
    /// Load history from file
    pub fn load() -> Self {
        let path = Self::history_file_path();
        
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(history) = serde_json::from_str(&content) {
                return history;
            }
        }
        
        Self::default()
    }
    
    /// Save history to file
    pub fn save(&self) -> Result<(), String> {
        let path = Self::history_file_path();
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create history directory: {}", e))?;
        }
        
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize history: {}", e))?;
        
        fs::write(&path, json)
            .map_err(|e| format!("Failed to write history: {}", e))?;
        
        Ok(())
    }
    
    /// Add entry to history
    pub fn add_entry(&mut self, entry: TransferHistoryEntry) {
        self.entries.push(entry);
        
        // Keep only last 100 entries
        if self.entries.len() > 100 {
            self.entries.drain(0..self.entries.len() - 100);
        }
        
        let _ = self.save();
    }
    
    /// Get recent entries
    pub fn get_recent(&self, count: usize) -> Vec<TransferHistoryEntry> {
        let start = if self.entries.len() > count {
            self.entries.len() - count
        } else {
            0
        };
        
        self.entries[start..].to_vec()
    }
    
    /// Get entries by status
    pub fn get_by_status(&self, status: TransferStatus) -> Vec<TransferHistoryEntry> {
        self.entries
            .iter()
            .filter(|e| e.status == status)
            .cloned()
            .collect()
    }
    
    /// Clear all history
    pub fn clear(&mut self) -> Result<(), String> {
        self.entries.clear();
        self.save()
    }
    
    /// Get history file path
    fn history_file_path() -> PathBuf {
        let mut path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."));
        path.push("igris");
        path.push("file_share_history.json");
        path
    }
}

/// Add a transfer to history
pub fn add_transfer(
    session_id: String,
    file_name: String,
    file_size: u64,
    device_name: String,
    device_ip: String,
    direction: TransferDirection,
    status: TransferStatus,
    error_message: Option<String>,
) {
    let mut history = TransferHistory::load();
    
    history.add_entry(TransferHistoryEntry {
        session_id,
        file_name,
        file_size,
        device_name,
        device_ip,
        direction,
        status,
        timestamp: Utc::now(),
        error_message,
    });
}

/// Get recent transfer history
pub fn get_recent_transfers(count: usize) -> Vec<TransferHistoryEntry> {
    let history = TransferHistory::load();
    history.get_recent(count)
}

/// Get failed transfers
pub fn get_failed_transfers() -> Vec<TransferHistoryEntry> {
    let history = TransferHistory::load();
    history.get_by_status(TransferStatus::Failed)
}

/// Clear transfer history
pub fn clear_history() -> Result<(), String> {
    let mut history = TransferHistory::load();
    history.clear()
}
