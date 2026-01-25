// src/lib.rs - IGRIS v3 library exports

// Allow dead code during development - many features are WIP
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::sync::{Arc, Mutex};

// Configuration system
pub mod config;

// UI components
pub mod ui;

// Core voice processing modules
pub mod core;

// Natural language understanding
pub mod nlu;

// Command handlers
pub mod commands;

// Plugin system
pub mod plugins;

// Utility modules
pub mod utils;

// Platform abstraction
pub mod platform;
pub mod platform_utils;

// Setup system
pub mod setup_manager;

// Media capture (camera, video)
pub mod media;

// File Share & Device Discovery
pub mod file_share;

// Global state for search results UI (shared across modules)
#[derive(Clone, Debug, Default)]
pub struct SearchState {
    pub is_open: bool,
    pub is_searching: bool,
    pub query: String,
    pub results: Vec<SearchResultData>,
}

#[derive(Clone, Debug)]
pub struct SearchResultData {
    pub path: String,
    pub name: String,
    pub drive: String,
    pub score: u32,
    pub is_folder: bool,
}

pub static SEARCH_STATE: once_cell::sync::Lazy<Arc<Mutex<SearchState>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(SearchState::default())));

// Re-export camera panel state from commands module
pub use commands::ffmpeg_camera::{CameraPanelState, CAMERA_PANEL_STATE};

// Re-export commonly used types
pub use config::{CONFIG, AppConfig, Personality, Theme};
pub use core::{stt, tts, vad, audio_capture, wake_word};
pub use nlu::{engine, ner, context, sbert};
pub use commands::{system, files, web};
pub use utils::{hotkey, greetings, shared_memory};
pub use ui::{SettingsPanel, SettingsButton};
pub use media::{CameraDevice, open_camera, close_camera, take_photo, start_recording, stop_recording, list_cameras};

// File Share exports
pub use file_share::{
    DeviceConfig, DeviceIdentity, TrustedDevice, OperatingSystem,
    CertificateManager, DeviceCertificate,
    DiscoveryService, DiscoveredDevice, DiscoveryEvent,
    start_discovery, stop_discovery, get_discovered_devices,
    TrustManager, TrustResult,
    establish_trust, check_rate_limit, is_device_trusted, get_all_trusted,
    BridgeManager, BridgeMessage, BridgeEvent, ConnectionState,
    connect_to_device, disconnect_from_device, send_to_device, is_connected_to,
    TransferManager, FileTransfer, TransferEvent, TransferStatus,
    send_file, accept_incoming_transfer, cancel_file_transfer, get_transfer_progress,
};
