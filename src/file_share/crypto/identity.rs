// Device identity management

use anyhow::Result;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

pub struct DeviceIdentity {
    fingerprint: String,
    identity_file: PathBuf,
}

impl DeviceIdentity {
    pub fn new(identity_file: PathBuf) -> Result<Self> {
        let fingerprint = if identity_file.exists() {
            Self::load_fingerprint(&identity_file)?
        } else {
            let fp = Self::generate_fingerprint();
            Self::save_fingerprint(&identity_file, &fp)?;
            fp
        };

        Ok(Self {
            fingerprint,
            identity_file,
        })
    }

    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    fn generate_fingerprint() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        
        let mut hasher = Sha256::new();
        hasher.update(timestamp.to_string().as_bytes());
        hasher.update(uuid::Uuid::new_v4().as_bytes());
        
        format!("{:x}", hasher.finalize())
    }

    fn save_fingerprint(path: &PathBuf, fingerprint: &str) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, fingerprint)?;
        Ok(())
    }

    fn load_fingerprint(path: &PathBuf) -> Result<String> {
        Ok(fs::read_to_string(path)?.trim().to_string())
    }
}
