// TLS configuration for HTTPS - Self-signed certificates

use anyhow::Result;
use rcgen::{Certificate, CertificateParams, DistinguishedName, DnType, KeyPair};
use rustls::ServerConfig;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

pub struct TlsConfig {
    pub server_config: Arc<ServerConfig>,
    pub cert_pem: String,
    pub key_pem: String,
    pub fingerprint: String,
}

impl TlsConfig {
    /// Create new TLS config with self-signed certificate
    pub fn new(device_name: &str, cert_path: PathBuf) -> Result<Self> {
        // Try to load existing certificate
        if cert_path.exists() {
            if let Ok(config) = Self::load_from_file(&cert_path) {
                return Ok(config);
            }
        }

        // Generate new self-signed certificate
        let (cert_pem, key_pem, fingerprint) = Self::generate_self_signed_cert(device_name)?;

        // Save to file
        Self::save_to_file(&cert_path, &cert_pem, &key_pem)?;

        // Create rustls ServerConfig
        let server_config = Self::create_server_config(&cert_pem, &key_pem)?;

        Ok(Self {
            server_config: Arc::new(server_config),
            cert_pem,
            key_pem,
            fingerprint,
        })
    }

    /// Generate self-signed certificate
    fn generate_self_signed_cert(device_name: &str) -> Result<(String, String, String)> {
        // Create certificate parameters
        let mut params = CertificateParams::default();
        
        // Set subject name
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, device_name);
        dn.push(DnType::OrganizationName, "LocalSend");
        params.distinguished_name = dn;

        // Add subject alternative names
        params.subject_alt_names = vec![
            rcgen::SanType::DnsName(rcgen::Ia5String::try_from("localhost")?),
            rcgen::SanType::IpAddress(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))),
        ];

        // Set validity period (1 year) - use time crate instead of chrono
        use time::{OffsetDateTime, Duration};
        let now = OffsetDateTime::now_utc();
        params.not_before = now;
        params.not_after = now + Duration::days(365);

        // Generate key pair
        let key_pair = KeyPair::generate()?;
        
        // Generate certificate
        let cert = params.self_signed(&key_pair)?;
        
        // Get PEM strings
        let cert_pem = cert.pem();
        let key_pem = key_pair.serialize_pem();

        // Calculate SHA-256 fingerprint
        let cert_der = cert.der();
        let fingerprint = Self::calculate_fingerprint(cert_der);

        Ok((cert_pem, key_pem, fingerprint))
    }

    /// Calculate SHA-256 fingerprint of certificate
    fn calculate_fingerprint(cert_der: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(cert_der);
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    /// Create rustls ServerConfig from PEM strings
    fn create_server_config(cert_pem: &str, key_pem: &str) -> Result<ServerConfig> {
        use rustls::pki_types::{CertificateDer, PrivateKeyDer};
        use rustls::ServerConfig;

        // Parse certificate
        let cert_chain: Vec<CertificateDer> = rustls_pemfile::certs(&mut cert_pem.as_bytes())
            .collect::<Result<Vec<_>, _>>()?;

        // Parse private key
        let key_der = rustls_pemfile::private_key(&mut key_pem.as_bytes())?
            .ok_or_else(|| anyhow::anyhow!("No private key found"))?;

        // Create server config
        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, key_der)?;

        Ok(config)
    }

    /// Save certificate and key to file
    fn save_to_file(path: &PathBuf, cert_pem: &str, key_pem: &str) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let data = serde_json::json!({
            "cert": cert_pem,
            "key": key_pem,
        });

        fs::write(path, serde_json::to_string_pretty(&data)?)?;
        Ok(())
    }

    /// Load certificate and key from file
    fn load_from_file(path: &PathBuf) -> Result<Self> {
        let data = fs::read_to_string(path)?;
        let json: serde_json::Value = serde_json::from_str(&data)?;

        let cert_pem = json["cert"].as_str().ok_or_else(|| anyhow::anyhow!("No cert"))?.to_string();
        let key_pem = json["key"].as_str().ok_or_else(|| anyhow::anyhow!("No key"))?.to_string();

        // Calculate fingerprint
        let cert_der = rustls_pemfile::certs(&mut cert_pem.as_bytes())
            .next()
            .ok_or_else(|| anyhow::anyhow!("No cert"))??;
        let fingerprint = Self::calculate_fingerprint(&cert_der);

        // Create server config
        let server_config = Self::create_server_config(&cert_pem, &key_pem)?;

        Ok(Self {
            server_config: Arc::new(server_config),
            cert_pem,
            key_pem,
            fingerprint,
        })
    }

    /// Get fingerprint for display
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    /// Get formatted fingerprint (XX:XX:XX:...)
    pub fn formatted_fingerprint(&self) -> String {
        self.fingerprint
            .as_bytes()
            .chunks(2)
            .map(|chunk| std::str::from_utf8(chunk).unwrap())
            .collect::<Vec<_>>()
            .join(":")
            .to_uppercase()
    }
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self::new("LocalSend", PathBuf::from("./pkg/file_share/cert.json")).unwrap()
    }
}
