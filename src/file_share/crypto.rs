// src/file_share/crypto.rs - TLS Certificate Generation & Management

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use sha2::{Sha256, Digest};
use rcgen::generate_simple_self_signed;
use chrono::Utc;
use once_cell::sync::Lazy;

use super::config::get_config_path;

/// Device certificate containing cert and private key
#[derive(Clone)]
pub struct DeviceCertificate {
    pub cert_pem: String,
    pub key_pem: String,
    pub fingerprint: String,
    pub generated_at: chrono::DateTime<Utc>,
}

impl DeviceCertificate {
    pub fn from_pem(cert_pem: String, key_pem: String) -> Self {
        let fingerprint = calculate_cert_fingerprint(&cert_pem);
        DeviceCertificate {
            cert_pem,
            key_pem,
            fingerprint,
            generated_at: Utc::now(),
        }
    }
}

/// Certificate manager handles certificate lifecycle
pub struct CertificateManager {
    certificate: Option<DeviceCertificate>,
    cert_path: PathBuf,
    key_path: PathBuf,
}

impl CertificateManager {
    pub fn new() -> Self {
        let config_path = get_config_path();
        CertificateManager {
            certificate: None,
            cert_path: config_path.join("device_cert.pem"),
            key_path: config_path.join("device_key.pem"),
        }
    }
    
    /// Initialize - load existing or generate new certificate
    pub fn initialize(&mut self) -> Result<&DeviceCertificate, String> {
        if self.cert_path.exists() && self.key_path.exists() {
            match self.load_certificate() {
                Ok(cert) => {
                    self.certificate = Some(cert);
                    return Ok(self.certificate.as_ref().unwrap());
                }
                Err(e) => {
                    println!("[FileShare] Failed to load certificate: {}", e);
                }
            }
        }
        
        let cert = self.generate_certificate()?;
        self.save_certificate(&cert)?;
        self.certificate = Some(cert);
        
        Ok(self.certificate.as_ref().unwrap())
    }
    
    pub fn get_certificate(&self) -> Option<&DeviceCertificate> {
        self.certificate.as_ref()
    }
    
    pub fn get_fingerprint(&self) -> Option<String> {
        self.certificate.as_ref().map(|c| c.fingerprint.clone())
    }
    
    fn generate_certificate(&self) -> Result<DeviceCertificate, String> {
        println!("[FileShare] Generating device certificate...");
        
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "igris-device".to_string());
        
        let subject_alt_names = vec![
            hostname.clone(),
            "localhost".to_string(),
            "127.0.0.1".to_string(),
        ];
        
        let certified_key = generate_simple_self_signed(subject_alt_names)
            .map_err(|e| format!("Failed to generate certificate: {}", e))?;
        
        let cert_pem = certified_key.cert.pem();
        let key_pem = certified_key.key_pair.serialize_pem();
        
        let cert = DeviceCertificate::from_pem(cert_pem, key_pem);
        
        println!("[FileShare] Certificate generated: {}", &cert.fingerprint[..16]);
        
        Ok(cert)
    }
    
    fn load_certificate(&self) -> Result<DeviceCertificate, String> {
        let cert_pem = fs::read_to_string(&self.cert_path)
            .map_err(|e| format!("Failed to read certificate: {}", e))?;
        
        let key_pem = fs::read_to_string(&self.key_path)
            .map_err(|e| format!("Failed to read private key: {}", e))?;
        
        Ok(DeviceCertificate::from_pem(cert_pem, key_pem))
    }
    
    fn save_certificate(&self, cert: &DeviceCertificate) -> Result<(), String> {
        if let Some(parent) = self.cert_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }
        
        fs::write(&self.cert_path, &cert.cert_pem)
            .map_err(|e| format!("Failed to write certificate: {}", e))?;
        
        fs::write(&self.key_path, &cert.key_pem)
            .map_err(|e| format!("Failed to write private key: {}", e))?;
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&self.key_path)
                .map_err(|e| format!("Failed to get permissions: {}", e))?
                .permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&self.key_path, perms)
                .map_err(|e| format!("Failed to set permissions: {}", e))?;
        }
        
        println!("[FileShare] Certificate saved to {:?}", self.cert_path);
        
        Ok(())
    }
    
    pub fn regenerate(&mut self) -> Result<&DeviceCertificate, String> {
        let cert = self.generate_certificate()?;
        self.save_certificate(&cert)?;
        self.certificate = Some(cert);
        Ok(self.certificate.as_ref().unwrap())
    }
}

fn calculate_cert_fingerprint(cert_pem: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(cert_pem.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

pub fn generate_device_fingerprint(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

static CERTIFICATE_MANAGER: Lazy<Arc<Mutex<CertificateManager>>> = Lazy::new(|| {
    Arc::new(Mutex::new(CertificateManager::new()))
});

pub fn get_certificate_manager() -> Arc<Mutex<CertificateManager>> {
    CERTIFICATE_MANAGER.clone()
}

pub fn initialize_certificate() -> Result<DeviceCertificate, String> {
    let manager = get_certificate_manager();
    let mut manager = manager.lock().map_err(|e| format!("Lock error: {}", e))?;
    let cert = manager.initialize()?;
    Ok(cert.clone())
}

pub fn get_device_fingerprint() -> Option<String> {
    let manager = get_certificate_manager();
    let manager = manager.lock().ok()?;
    manager.get_fingerprint()
}

/// Certificate verifier that accepts all certificates (for self-signed certs)
#[derive(Debug)]
pub struct NoCertificateVerification;

impl rustls::client::danger::ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        // Accept all certificates without verification
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ED25519,
        ]
    }
}
