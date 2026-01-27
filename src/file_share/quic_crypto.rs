// src/file_share/quic_crypto.rs - QUIC Certificate Management

use quinn::{ServerConfig, ClientConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

/// QUIC certificate manager
pub struct QuicCertManager {
    cert_chain: Vec<CertificateDer<'static>>,
    private_key: PrivateKeyDer<'static>,
    fingerprint: String,
}

impl QuicCertManager {
    /// Generate self-signed certificate for QUIC
    pub fn new() -> Result<Self, String> {
        use rcgen::generate_simple_self_signed;
        
        let subject_alt_names = vec!["localhost".to_string()];
        let cert = generate_simple_self_signed(subject_alt_names)
            .map_err(|e| format!("Failed to generate certificate: {}", e))?;
        
        let cert_der = CertificateDer::from(cert.cert.der().to_vec());
        let key_der = PrivateKeyDer::try_from(cert.key_pair.serialize_der())
            .map_err(|e| format!("Failed to serialize key: {}", e))?;
        
        // Calculate fingerprint (SHA-256 of certificate)
        let fingerprint = {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(cert_der.as_ref());
            format!("{:x}", hasher.finalize())
        };
        
        println!("[QuicCrypto] Certificate generated with fingerprint: {}", &fingerprint[..16]);
        
        Ok(QuicCertManager {
            cert_chain: vec![cert_der],
            private_key: key_der,
            fingerprint,
        })
    }
    
    /// Create QUIC server config
    pub fn server_config(&self) -> Result<ServerConfig, String> {
        let mut server_config = ServerConfig::with_single_cert(
            self.cert_chain.clone(),
            self.private_key.clone_key(),
        ).map_err(|e| format!("Failed to create server config: {}", e))?;
        
        // Configure transport
        let mut transport = quinn::TransportConfig::default();
        transport.max_concurrent_bidi_streams(100u32.into());
        transport.max_concurrent_uni_streams(100u32.into());
        transport.keep_alive_interval(Some(std::time::Duration::from_secs(5)));
        transport.max_idle_timeout(Some(std::time::Duration::from_secs(30).try_into().unwrap()));
        
        server_config.transport_config(Arc::new(transport));
        
        Ok(server_config)
    }
    
    /// Create QUIC client config (accepts self-signed certs)
    pub fn client_config() -> Result<ClientConfig, String> {
        let crypto = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(SkipServerVerification))
            .with_no_client_auth();
        
        let mut client_config = ClientConfig::new(Arc::new(
            quinn::crypto::rustls::QuicClientConfig::try_from(crypto)
                .map_err(|e| format!("Failed to create QUIC config: {}", e))?
        ));
        
        // Configure transport
        let mut transport = quinn::TransportConfig::default();
        transport.max_concurrent_bidi_streams(100u32.into());
        transport.max_concurrent_uni_streams(100u32.into());
        transport.keep_alive_interval(Some(std::time::Duration::from_secs(5)));
        transport.max_idle_timeout(Some(std::time::Duration::from_secs(30).try_into().unwrap()));
        
        client_config.transport_config(Arc::new(transport));
        
        Ok(client_config)
    }
    
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }
}

/// Skip certificate verification for self-signed certs
#[derive(Debug)]
struct SkipServerVerification;

impl rustls::client::danger::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }
    
    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    
    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
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

// Global certificate manager
static QUIC_CERT_MANAGER: Lazy<Arc<Mutex<Option<QuicCertManager>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(None)));

pub fn get_quic_cert_manager() -> Result<Arc<Mutex<Option<QuicCertManager>>>, String> {
    let mut manager = QUIC_CERT_MANAGER.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    if manager.is_none() {
        *manager = Some(QuicCertManager::new()?);
    }
    
    Ok(QUIC_CERT_MANAGER.clone())
}

/// Initialize QUIC certificate manager
pub fn initialize_quic_crypto() -> Result<(), String> {
    let _ = get_quic_cert_manager()?;
    println!("[QuicCrypto] Certificate manager initialized");
    Ok(())
}
