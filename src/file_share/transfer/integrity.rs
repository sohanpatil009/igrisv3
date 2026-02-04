// File integrity checking with SHA-256

use anyhow::Result;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub struct FileIntegrity;

impl FileIntegrity {
    /// Calculate SHA-256 hash of a file
    pub fn calculate_hash<P: AsRef<Path>>(path: P) -> Result<String> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    /// Verify file hash
    pub fn verify_hash<P: AsRef<Path>>(path: P, expected_hash: &str) -> Result<bool> {
        let actual_hash = Self::calculate_hash(path)?;
        Ok(actual_hash.eq_ignore_ascii_case(expected_hash))
    }

    /// Calculate hash of bytes
    pub fn hash_bytes(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        format!("{:x}", result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hash_calculation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"Hello, World!").unwrap();
        
        let hash = FileIntegrity::calculate_hash(temp_file.path()).unwrap();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA-256 produces 64 hex characters
    }

    #[test]
    fn test_hash_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"Test data").unwrap();
        
        let hash = FileIntegrity::calculate_hash(temp_file.path()).unwrap();
        let verified = FileIntegrity::verify_hash(temp_file.path(), &hash).unwrap();
        assert!(verified);
    }
}
