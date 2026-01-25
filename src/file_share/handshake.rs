// src/file_share/handshake.rs - Bridge Handshake Protocol

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream as ClientTlsStream;
use tokio_rustls::server::TlsStream as ServerTlsStream;

use super::config::OperatingSystem;

/// Handshake message types for bidirectional device information exchange
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum HandshakeMessage {
    /// Sent by the initiator to introduce themselves to the responder
    InitiatorHello {
        device_id: String,
        hostname: String,
        label: String,
        os: OperatingSystem,
        ip_address: String,
        bridge_port: u16,
        cert_fingerprint: String,
    },
    /// Sent by the responder to acknowledge the connection
    ResponderAck {
        device_id: String,
        cert_fingerprint: String,
        trust_established: bool,
    },
    /// Sent when an error occurs during handshake
    Error {
        message: String,
    },
}

impl HandshakeMessage {
    /// Create an InitiatorHello message
    pub fn initiator_hello(
        device_id: String,
        hostname: String,
        label: String,
        os: OperatingSystem,
        ip_address: String,
        bridge_port: u16,
        cert_fingerprint: String,
    ) -> Self {
        HandshakeMessage::InitiatorHello {
            device_id,
            hostname,
            label,
            os,
            ip_address,
            bridge_port,
            cert_fingerprint,
        }
    }

    /// Create a ResponderAck message
    pub fn responder_ack(
        device_id: String,
        cert_fingerprint: String,
        trust_established: bool,
    ) -> Self {
        HandshakeMessage::ResponderAck {
            device_id,
            cert_fingerprint,
            trust_established,
        }
    }

    /// Create an Error message
    pub fn error(message: String) -> Self {
        HandshakeMessage::Error { message }
    }

    /// Serialize the message to JSON bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize a message from JSON bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

/// Send a handshake message over a client TLS stream
pub async fn send_handshake_client(
    stream: &mut ClientTlsStream<TcpStream>,
    message: &HandshakeMessage,
) -> Result<(), std::io::Error> {
    let bytes = message.to_bytes()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    
    // Send message length as 4-byte big-endian integer
    let len = bytes.len() as u32;
    stream.write_all(&len.to_be_bytes()).await?;
    
    // Send message data
    stream.write_all(&bytes).await?;
    stream.flush().await?;
    
    Ok(())
}

/// Send a handshake message over a server TLS stream
pub async fn send_handshake_server(
    stream: &mut ServerTlsStream<TcpStream>,
    message: &HandshakeMessage,
) -> Result<(), std::io::Error> {
    let bytes = message.to_bytes()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    
    // Send message length as 4-byte big-endian integer
    let len = bytes.len() as u32;
    stream.write_all(&len.to_be_bytes()).await?;
    
    // Send message data
    stream.write_all(&bytes).await?;
    stream.flush().await?;
    
    Ok(())
}

/// Receive a handshake message from a client TLS stream
pub async fn receive_handshake_client(
    stream: &mut ClientTlsStream<TcpStream>,
) -> Result<HandshakeMessage, std::io::Error> {
    // Read message length (4 bytes, big-endian)
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes).await?;
    let len = u32::from_be_bytes(len_bytes) as usize;
    
    // Validate message length (max 1MB for safety)
    if len > 1_048_576 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Handshake message too large",
        ));
    }
    
    // Read message data
    let mut buffer = vec![0u8; len];
    stream.read_exact(&mut buffer).await?;
    
    // Deserialize message
    HandshakeMessage::from_bytes(&buffer)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// Receive a handshake message from a server TLS stream
pub async fn receive_handshake_server(
    stream: &mut ServerTlsStream<TcpStream>,
) -> Result<HandshakeMessage, std::io::Error> {
    // Read message length (4 bytes, big-endian)
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes).await?;
    let len = u32::from_be_bytes(len_bytes) as usize;
    
    // Validate message length (max 1MB for safety)
    if len > 1_048_576 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Handshake message too large",
        ));
    }
    
    // Read message data
    let mut buffer = vec![0u8; len];
    stream.read_exact(&mut buffer).await?;
    
    // Deserialize message
    HandshakeMessage::from_bytes(&buffer)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initiator_hello_creation() {
        let msg = HandshakeMessage::initiator_hello(
            "device123".to_string(),
            "MyComputer".to_string(),
            "My Device".to_string(),
            OperatingSystem::Linux,
            "192.168.1.10".to_string(),
            45679,
            "abc123".to_string(),
        );

        match msg {
            HandshakeMessage::InitiatorHello {
                device_id,
                hostname,
                label,
                os,
                ip_address,
                bridge_port,
                cert_fingerprint,
            } => {
                assert_eq!(device_id, "device123");
                assert_eq!(hostname, "MyComputer");
                assert_eq!(label, "My Device");
                assert_eq!(os, OperatingSystem::Linux);
                assert_eq!(ip_address, "192.168.1.10");
                assert_eq!(bridge_port, 45679);
                assert_eq!(cert_fingerprint, "abc123");
            }
            _ => panic!("Expected InitiatorHello"),
        }
    }

    #[test]
    fn test_responder_ack_creation() {
        let msg = HandshakeMessage::responder_ack(
            "device456".to_string(),
            "def456".to_string(),
            true,
        );

        match msg {
            HandshakeMessage::ResponderAck {
                device_id,
                cert_fingerprint,
                trust_established,
            } => {
                assert_eq!(device_id, "device456");
                assert_eq!(cert_fingerprint, "def456");
                assert!(trust_established);
            }
            _ => panic!("Expected ResponderAck"),
        }
    }

    #[test]
    fn test_error_creation() {
        let msg = HandshakeMessage::error("Connection failed".to_string());

        match msg {
            HandshakeMessage::Error { message } => {
                assert_eq!(message, "Connection failed");
            }
            _ => panic!("Expected Error"),
        }
    }

    #[test]
    fn test_serialization_round_trip() {
        let original = HandshakeMessage::initiator_hello(
            "device123".to_string(),
            "MyComputer".to_string(),
            "My Device".to_string(),
            OperatingSystem::MacOS,
            "192.168.1.10".to_string(),
            45679,
            "abc123".to_string(),
        );

        let bytes = original.to_bytes().expect("Serialization failed");
        let deserialized = HandshakeMessage::from_bytes(&bytes)
            .expect("Deserialization failed");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_responder_ack_serialization() {
        let original = HandshakeMessage::responder_ack(
            "device456".to_string(),
            "def456".to_string(),
            true,
        );

        let bytes = original.to_bytes().expect("Serialization failed");
        let deserialized = HandshakeMessage::from_bytes(&bytes)
            .expect("Deserialization failed");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_error_serialization() {
        let original = HandshakeMessage::error("Test error".to_string());

        let bytes = original.to_bytes().expect("Serialization failed");
        let deserialized = HandshakeMessage::from_bytes(&bytes)
            .expect("Deserialization failed");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_all_os_types_serialization() {
        let os_types = vec![
            OperatingSystem::Windows,
            OperatingSystem::MacOS,
            OperatingSystem::Linux,
            OperatingSystem::Unknown,
        ];

        for os in os_types {
            let msg = HandshakeMessage::initiator_hello(
                "device".to_string(),
                "host".to_string(),
                "label".to_string(),
                os.clone(),
                "127.0.0.1".to_string(),
                8080,
                "cert".to_string(),
            );

            let bytes = msg.to_bytes().expect("Serialization failed");
            let deserialized = HandshakeMessage::from_bytes(&bytes)
                .expect("Deserialization failed");

            assert_eq!(msg, deserialized);
        }
    }

    #[test]
    fn test_invalid_json_deserialization() {
        let invalid_bytes = b"not valid json";
        let result = HandshakeMessage::from_bytes(invalid_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_bytes_deserialization() {
        let empty_bytes = b"";
        let result = HandshakeMessage::from_bytes(empty_bytes);
        assert!(result.is_err());
    }
}
