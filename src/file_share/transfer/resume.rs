// Resume capability for interrupted transfers

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Resume information for a file transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeInfo {
    pub session_id: String,
    pub file_id: String,
    pub file_path: PathBuf,
    pub total_size: u64,
    pub transferred_size: u64,
    pub checksum: Option<String>,
}

/// Manager for resume information
pub struct ResumeManager {
    resume_file: PathBuf,
    sessions: HashMap<String, Vec<ResumeInfo>>,
}

impl ResumeManager {
    pub fn new(resume_file: PathBuf) -> Self {
        let sessions = Self::load_from_file(&resume_file).unwrap_or_default();
        Self {
            resume_file,
            sessions,
        }
    }

    /// Save resume info for a file
    pub fn save_resume_info(&mut self, info: ResumeInfo) {
        self.sessions
            .entry(info.session_id.clone())
            .or_insert_with(Vec::new)
            .push(info);
        let _ = self.save_to_file();
    }

    /// Get resume info for a session
    pub fn get_resume_info(&self, session_id: &str) -> Option<&Vec<ResumeInfo>> {
        self.sessions.get(session_id)
    }

    /// Remove resume info for a session
    pub fn remove_session(&mut self, session_id: &str) {
        self.sessions.remove(session_id);
        let _ = self.save_to_file();
    }

    /// Clear all resume info
    pub fn clear_all(&mut self) {
        self.sessions.clear();
        let _ = self.save_to_file();
    }

    fn save_to_file(&self) -> anyhow::Result<()> {
        let data = serde_json::to_string_pretty(&self.sessions)?;
        fs::write(&self.resume_file, data)?;
        Ok(())
    }

    fn load_from_file(path: &PathBuf) -> anyhow::Result<HashMap<String, Vec<ResumeInfo>>> {
        let data = fs::read_to_string(path)?;
        let sessions = serde_json::from_str(&data)?;
        Ok(sessions)
    }
}
