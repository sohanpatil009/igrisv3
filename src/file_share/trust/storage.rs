// Trusted device storage

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedDevice {
    pub fingerprint: String,
    pub alias: String,
    pub last_seen: u64,
}

pub struct TrustedDeviceStorage {
    storage_file: PathBuf,
    trusted_devices: HashSet<String>, // fingerprints
}

impl TrustedDeviceStorage {
    pub fn new(storage_file: PathBuf) -> Result<Self> {
        let trusted_devices = if storage_file.exists() {
            Self::load_from_file(&storage_file)?
        } else {
            HashSet::new()
        };

        Ok(Self {
            storage_file,
            trusted_devices,
        })
    }

    pub fn is_trusted(&self, fingerprint: &str) -> bool {
        self.trusted_devices.contains(fingerprint)
    }

    pub fn add_trusted(&mut self, fingerprint: String) -> Result<()> {
        self.trusted_devices.insert(fingerprint);
        self.save_to_file()
    }

    pub fn remove_trusted(&mut self, fingerprint: &str) -> Result<()> {
        self.trusted_devices.remove(fingerprint);
        self.save_to_file()
    }

    pub fn get_all_trusted(&self) -> Vec<String> {
        self.trusted_devices.iter().cloned().collect()
    }

    fn save_to_file(&self) -> Result<()> {
        let data = serde_json::to_string_pretty(&self.trusted_devices)?;
        if let Some(parent) = self.storage_file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.storage_file, data)?;
        Ok(())
    }

    fn load_from_file(path: &PathBuf) -> Result<HashSet<String>> {
        let data = fs::read_to_string(path)?;
        let devices = serde_json::from_str(&data)?;
        Ok(devices)
    }
}
