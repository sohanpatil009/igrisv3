// src/file_share/mod.rs - IGRIS File Share & Device Discovery Module
//
// This module provides a unified file sharing and device connection system with:
// - Automatic device discovery on local networks
// - Code-based connection for cross-subnet/remote devices
// - Bidirectional trust establishment with one-way code entry
// - Secure TLS connections with certificate pinning
// - File transfer with progress tracking
//
// ## Quick Start
//
// ```rust
// use igris::file_share::{generate_connection_code, connect_to_device_with_code};
//
// // Generate a code for others to connect to you
// let my_code = generate_connection_code()?;
// println!("Share this code: {}", my_code.code);
//
// // Connect to another device using their code
// let result = connect_to_device_with_code("1234").await?;
// println!("Connected to: {}", result.device.label);
// ```

use std::sync::Arc;

// Module declarations
pub mod config;
pub mod crypto;
pub mod discovery;
pub mod trust;
pub mod transfer;
pub mod manager;
pub mod relay;
pub mod connection_types;
pub mod handshake;
pub mod connection;

// QUIC modules (replaces old TCP+TLS bridge)
pub mod quic_crypto;
pub mod quic_bridge;
pub mod quic_relay;
pub mod relay_server;

// Re-export core types
pub use config::{
    DeviceConfig, DeviceIdentity, TrustedDevice, OperatingSystem,
    get_config_path, load_config, save_config, get_or_create_device_identity,
};
pub use crypto::{
    CertificateManager, DeviceCertificate,
    generate_device_fingerprint, get_certificate_manager,
};
pub use discovery::{
    DiscoveryService, DiscoveredDevice, DiscoveryMessage, DiscoveryEvent,
    get_discovery_service, start_discovery, stop_discovery, get_discovered_devices,
};
pub use trust::{
    TrustManager, TrustResult,
    get_trust_manager, establish_trust, check_rate_limit, record_failed_attempt,
    add_trusted, remove_trusted, is_device_trusted, get_all_trusted, rename_trusted_device,
};
pub use transfer::{
    TransferManager, FileTransfer, TransferEvent, TransferStatus, TransferDirection,
    get_transfer_manager, send_file, accept_incoming_transfer, reject_incoming_transfer,
    cancel_file_transfer, get_transfer_progress, get_default_save_path, format_file_size,
};
pub use relay::{
    generate_my_code, connect_with_code, invalidate_code, get_my_device_code,
    DeviceRegistration,
};
pub use connection_types::{
    DeviceInfo, ConnectionCode, ConnectionResult, ConnectionType, ConnectionError,
};
pub use handshake::{
    HandshakeMessage,
};
pub use connection::{
    ConnectionCoordinator,
};
pub use quic_crypto::{
    QuicCertManager, get_quic_cert_manager, initialize_quic_crypto,
};
pub use quic_bridge::{
    QuicBridgeManager, QuicMessage, QuicBridgeEvent,
    get_quic_bridge_manager, initialize_quic_bridge,
    connect_to_device_quic, send_to_device_quic, is_connected_to_quic,
};

// ============================================================================
// Convenience Functions for Common Operations
// ============================================================================

/// Generate a connection code for this device
///
/// Returns a 4-digit code that other devices can use to connect to you.
/// The code is valid for 10 minutes and will automatically regenerate when expired.
///
/// # Example
///
/// ```rust
/// use igris::file_share::generate_connection_code;
///
/// let code = generate_connection_code()?;
/// println!("Share this code: {}", code.code);
/// println!("Expires in {} seconds", code.remaining_seconds);
/// ```
pub fn generate_connection_code() -> Result<ConnectionCode, ConnectionError> {
    let relay = Arc::new(relay::RelayService::new());
    let trust = get_trust_manager();
    let discovery = get_discovery_service()
        .map_err(|e| ConnectionError::NetworkError(e))?;
    
    let coordinator = ConnectionCoordinator::new(relay, trust, discovery);
    coordinator.generate_my_code()
}

/// Connect to a device using their 4-digit code
///
/// This function performs the complete connection flow:
/// 1. Validates the code format
/// 2. Looks up the device in the relay service
/// 3. Establishes a TLS connection
/// 4. Performs bidirectional handshake
/// 5. Establishes mutual trust
///
/// # Arguments
///
/// * `code` - The 4-digit numeric code from the remote device
///
/// # Returns
///
/// A `ConnectionResult` containing the connected device information and trust status.
///
/// # Errors
///
/// Returns `ConnectionError` if:
/// - The code format is invalid (not 4 digits)
/// - The code is not found or expired
/// - Network connection fails
/// - Trust establishment fails
/// - Rate limiting is active (too many failed attempts)
///
/// # Example
///
/// ```rust
/// use igris::file_share::connect_to_device_with_code;
///
/// match connect_to_device_with_code("1234").await {
///     Ok(result) => {
///         println!("Connected to: {}", result.device.label);
///         println!("Trust established: {}", result.trust_established);
///     }
///     Err(e) => {
///         eprintln!("Connection failed: {}", e.user_message());
///     }
/// }
/// ```
pub async fn connect_to_device_with_code(code: &str) -> Result<ConnectionResult, ConnectionError> {
    let relay = Arc::new(relay::RelayService::new());
    let trust = get_trust_manager();
    let discovery = get_discovery_service()
        .map_err(|e| ConnectionError::NetworkError(e))?;
    
    let coordinator = ConnectionCoordinator::new(relay, trust, discovery);
    coordinator.connect_with_code(code).await
}

/// Handle an incoming connection from another device
///
/// This function is called by the bridge server when receiving an InitiatorHello handshake.
/// It establishes trust for the initiator and adds them to the discovery cache.
///
/// # Arguments
///
/// * `handshake` - The InitiatorHello message from the connecting device
///
/// # Returns
///
/// A `HandshakeMessage` response (either ResponderAck or Error)
///
/// # Example
///
/// ```rust
/// use igris::file_share::{handle_incoming_device_connection, HandshakeMessage};
///
/// async fn on_connection(handshake: HandshakeMessage) {
///     match handle_incoming_device_connection(handshake).await {
///         Ok(response) => {
///             // Send response back to initiator
///         }
///         Err(e) => {
///             eprintln!("Failed to handle connection: {}", e);
///         }
///     }
/// }
/// ```
pub async fn handle_incoming_device_connection(
    handshake: HandshakeMessage,
) -> Result<HandshakeMessage, ConnectionError> {
    let relay = Arc::new(relay::RelayService::new());
    let trust = get_trust_manager();
    let discovery = get_discovery_service()
        .map_err(|e| ConnectionError::NetworkError(e))?;
    
    let coordinator = ConnectionCoordinator::new(relay, trust, discovery);
    coordinator.handle_incoming_connection(handshake).await
}
