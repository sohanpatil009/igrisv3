// src/ui/file_share/panel.rs - Main File Share Panel (Voice + Manual Control)

use dioxus::prelude::*;
use super::{DeviceRadar, DeviceDisplay, MyDevices, TrustedDeviceDisplay, TransferProgress, TransferDisplay};

/// Panel state - which view is active
#[derive(Clone, PartialEq, Debug, Default)]
pub enum FileSharePanelState {
    #[default]
    Hidden,
    Radar,      // Scanning for devices
    MyDevices,  // Trusted devices list
    Transfers,  // Active transfers
    Pairing { device_id: String, device_label: String }, // Pairing with code
    SendFile { device_id: String, device_label: String }, // File picker
}

/// File Share Panel - Unified UI for voice and manual control
#[component]
pub fn FileSharePanel(
    mut panel_state: Signal<FileSharePanelState>,
) -> Element {
    // Local state
    let mut devices = use_signal(Vec::<DeviceDisplay>::new);
    let mut trusted_devices = use_signal(Vec::<TrustedDeviceDisplay>::new);
    let mut transfers = use_signal(Vec::<TransferDisplay>::new);
    let mut is_scanning = use_signal(|| false);
    let pairing_code = use_signal(|| None::<String>);
    let mut error_message = use_signal(|| None::<String>);
    
    // Refresh data when panel opens and start scanning
    use_effect(move || {
        let state = panel_state();
        if state != FileSharePanelState::Hidden {
            // Start scanning animation
            is_scanning.set(true);
            
            spawn(async move {
                // Start discovery service
                if let Err(e) = start_scan_async().await {
                    error_message.set(Some(e));
                }
                
                // Refresh data immediately
                refresh_data(&mut devices, &mut trusted_devices, &mut transfers).await;
                
                // Keep scanning animation running for 15 seconds
                tokio::time::sleep(std::time::Duration::from_secs(15)).await;
                
                // Refresh again after scan completes
                refresh_data(&mut devices, &mut trusted_devices, &mut transfers).await;
                
                // Stop scanning animation
                is_scanning.set(false);
            });
        } else {
            // Clear data when panel is hidden
            devices.set(Vec::new());
            trusted_devices.set(Vec::new());
            transfers.set(Vec::new());
            is_scanning.set(false);
        }
    });
    
    // Handle device selection from radar
    let on_device_select = move |device_id: String| {
        println!("[FileShare UI] Device selected: {}", device_id);
    };
    
    // Handle connect button - direct IP connection
    let on_connect = move |device_id: String| {
        let device = devices().iter().find(|d| d.id == device_id).cloned();
        if let Some(d) = device {
            println!("[FileShare] Direct connecting to {} at {}", d.label, d.ip);
            spawn(async move {
                match connect_direct_async(&d.ip, 45679, &d.label).await {
                    Ok(result) => {
                        println!("[FileShare] Connected to {}", result.device_label);
                        // Refresh device list to show updated trust status
                        refresh_data(&mut devices, &mut trusted_devices, &mut transfers).await;
                    },
                    Err(e) => {
                        println!("[FileShare] Connection error: {}", e);
                        error_message.set(Some(format!("Connection failed: {}", e)));
                    }
                }
            });
        }
    };
    
    // Handle close
    let on_close = move |_| {
        println!("[FileShare UI] Closing panel and stopping discovery...");
        
        // Stop discovery service when closing
        spawn(async move {
            let _ = stop_discovery_async().await;
        });
        
        // Update global state first
        if let Ok(mut global_state) = crate::commands::file_share::FILE_SHARE_PANEL_STATE.lock() {
            *global_state = FileSharePanelState::Hidden;
        }
        
        // Hide panel (local state)
        panel_state.set(FileSharePanelState::Hidden);
        
        // Reset scanning state
        is_scanning.set(false);
    };
    
    // Handle start scan
    let on_start_scan = move |_| {
        is_scanning.set(true);
        spawn(async move {
            if let Err(e) = start_scan_async().await {
                error_message.set(Some(e));
            }
            // Keep scanning animation running for 15 seconds to show user that scan is in progress
            tokio::time::sleep(std::time::Duration::from_secs(15)).await;
            refresh_data(&mut devices, &mut trusted_devices, &mut transfers).await;
            is_scanning.set(false);
        });
    };
    
    // Handle trusted device remove
    let on_remove_device = move |device_id: String| {
        spawn(async move {
            match remove_trusted_async(&device_id).await {
                Ok(_) => refresh_data(&mut devices, &mut trusted_devices, &mut transfers).await,
                Err(e) => error_message.set(Some(e)),
            }
        });
    };
    
    // Handle transfer actions
    let on_cancel_transfer = move |transfer_id: String| {
        spawn(async move {
            let _ = cancel_transfer_async(&transfer_id).await;
            refresh_data(&mut devices, &mut trusted_devices, &mut transfers).await;
        });
    };
    
    let on_accept_transfer = move |transfer_id: String| {
        spawn(async move {
            let _ = accept_transfer_async(&transfer_id).await;
            refresh_data(&mut devices, &mut trusted_devices, &mut transfers).await;
        });
    };
    
    let on_reject_transfer = move |transfer_id: String| {
        spawn(async move {
            let _ = reject_transfer_async(&transfer_id).await;
            refresh_data(&mut devices, &mut trusted_devices, &mut transfers).await;
        });
    };

    
    // Render based on state
    match panel_state() {
        FileSharePanelState::Hidden => rsx! {},
        
        FileSharePanelState::Radar => rsx! {
            div {
                class: "file-share-overlay",
                style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 1000;",
                
                div {
                    style: "position: relative;",
                    
                    if let Some(err) = error_message() {
                        div {
                            style: "position: absolute; top: -50px; left: 50%; transform: translateX(-50%); background: #ef4444; color: white; padding: 8px 16px; border-radius: 8px;",
                            "{err}"
                            button {
                                style: "margin-left: 8px; background: none; border: none; color: white; cursor: pointer;",
                                onclick: move |_| error_message.set(None),
                                "✕"
                            }
                        }
                    }
                    
                    DeviceRadar {
                        devices: devices(),
                        is_scanning: is_scanning(),
                        on_device_select: on_device_select,
                        on_connect: on_connect,
                        on_close: on_close,
                    }
                    
                    if !is_scanning() {
                        button {
                            style: "position: absolute; bottom: -50px; left: 50%; transform: translateX(-50%); background: #3b82f6; color: white; border: none; padding: 10px 24px; border-radius: 8px; cursor: pointer;",
                            onclick: on_start_scan,
                            "🔍 Scan Again"
                        }
                    }
                }
            }
        },
        
        FileSharePanelState::MyDevices => rsx! {
            div {
                class: "file-share-overlay",
                style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 1000;",
                
                div {
                    MyDevices {
                        devices: trusted_devices(),
                        on_remove: on_remove_device,
                        on_close: on_close,
                    }
                }
            }
        },
        
        FileSharePanelState::Transfers => rsx! {
            div {
                class: "file-share-overlay", 
                style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 1000;",
                
                div {
                    TransferProgress {
                        transfers: transfers(),
                        on_cancel: on_cancel_transfer,
                        on_accept: on_accept_transfer,
                        on_reject: on_reject_transfer,
                        on_close: on_close,
                    }
                }
            }
        },
        
        FileSharePanelState::Pairing { device_id: _, device_label } => rsx! {
            div {
                class: "file-share-overlay",
                style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 1000;",
                
                div {
                    PairingDialog {
                        device_label: device_label,
                        code: pairing_code().unwrap_or_default(),
                        on_close: move |_| panel_state.set(FileSharePanelState::Radar),
                    }
                }
            }
        },
        
        FileSharePanelState::SendFile { device_id, device_label } => rsx! {
            div {
                class: "file-share-overlay",
                style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 1000;",
                
                div {
                    SendFileDialog {
                        device_id: device_id,
                        device_label: device_label,
                        on_close: move |_| panel_state.set(FileSharePanelState::MyDevices),
                    }
                }
            }
        },
    }
}

/// Pairing dialog with 4-digit code
#[component]
fn PairingDialog(
    device_label: String,
    code: String,
    on_close: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            style: "background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%); border-radius: 16px; padding: 32px; min-width: 350px; text-align: center;",
            
            h2 { style: "color: #fff; margin: 0 0 8px;", "🔗 Pairing" }
            p { style: "color: #94a3b8; margin: 0 0 24px;", "Connect to {device_label}" }
            
            div {
                style: "display: flex; justify-content: center; gap: 12px; margin-bottom: 24px;",
                for digit in code.chars() {
                    div {
                        style: "width: 60px; height: 70px; background: #0f172a; border: 2px solid #3b82f6; border-radius: 12px; display: flex; align-items: center; justify-content: center; font-size: 32px; font-weight: bold; color: #3b82f6;",
                        "{digit}"
                    }
                }
            }
            
            p { style: "color: #64748b; font-size: 14px; margin-bottom: 24px;", "Enter this code on the other device" }
            div { style: "color: #f59e0b; font-size: 14px; margin-bottom: 24px;", "⏱️ Code expires in 60 seconds" }
            
            button {
                style: "background: #334155; color: #fff; border: none; padding: 12px 32px; border-radius: 8px; cursor: pointer;",
                onclick: move |_| on_close.call(()),
                "Cancel"
            }
        }
    }
}

/// Send file dialog
#[component]
fn SendFileDialog(
    device_id: String,
    device_label: String,
    on_close: EventHandler<()>,
) -> Element {
    let selected_file = use_signal(|| None::<String>);
    let mut is_sending = use_signal(|| false);
    
    let device_id_clone = device_id.clone();
    let on_send = move |_| {
        if let Some(file_path) = selected_file() {
            is_sending.set(true);
            let did = device_id_clone.clone();
            spawn(async move {
                match send_file_async(&did, &file_path).await {
                    Ok(_) => println!("[FileShare] File sent"),
                    Err(e) => println!("[FileShare] Error: {}", e),
                }
                is_sending.set(false);
            });
        }
    };
    
    rsx! {
        div {
            style: "background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%); border-radius: 16px; padding: 32px; min-width: 400px;",
            
            h2 { style: "color: #fff; margin: 0 0 8px;", "📤 Send File" }
            p { style: "color: #94a3b8; margin: 0 0 24px;", "Send to {device_label}" }
            
            div {
                style: "border: 2px dashed #334155; border-radius: 12px; padding: 40px; text-align: center; margin-bottom: 24px; cursor: pointer;",
                
                if let Some(file) = selected_file() {
                    div { style: "color: #4ade80;", "📄 {file}" }
                } else {
                    div {
                        div { style: "font-size: 48px; margin-bottom: 12px;", "📁" }
                        div { style: "color: #64748b;", "Click to select a file" }
                    }
                }
            }
            
            div {
                style: "display: flex; gap: 12px; justify-content: flex-end;",
                button {
                    style: "background: #334155; color: #fff; border: none; padding: 12px 24px; border-radius: 8px; cursor: pointer;",
                    onclick: move |_| on_close.call(()),
                    "Cancel"
                }
                button {
                    style: "background: #3b82f6; color: #fff; border: none; padding: 12px 24px; border-radius: 8px; cursor: pointer;",
                    disabled: selected_file().is_none() || is_sending(),
                    onclick: on_send,
                    if is_sending() { "Sending..." } else { "Send" }
                }
            }
        }
    }
}


// ═══════════════════════════════════════════════════════
// ASYNC HELPERS
// ═══════════════════════════════════════════════════════

async fn refresh_data(
    devices: &mut Signal<Vec<DeviceDisplay>>,
    trusted: &mut Signal<Vec<TrustedDeviceDisplay>>,
    transfers: &mut Signal<Vec<TransferDisplay>>,
) {
    // Get discovered devices
    if let Ok(discovered) = crate::file_share::discovery::get_discovered_devices() {
        let display: Vec<DeviceDisplay> = discovered.iter().map(|d| {
            let is_trusted = crate::file_share::trust::is_device_trusted(&d.id).unwrap_or(false);
            DeviceDisplay {
                id: d.id.clone(),
                label: d.label.clone(),
                os: d.os.as_str().to_string(),
                os_icon: get_os_icon(d.os.as_str()),
                ip: d.ip_address.to_string(),
                is_trusted,
                is_online: d.is_online(),
                code: d.code.clone(), // Include the broadcasted code
            }
        }).collect();
        devices.set(display);
    }
    
    // Get trusted devices
    if let Ok(trusted_list) = crate::file_share::trust::get_all_trusted() {
        let display: Vec<TrustedDeviceDisplay> = trusted_list.iter().map(|d| {
            TrustedDeviceDisplay {
                id: d.id.clone(),
                label: d.label.clone(),
                os: d.os.as_str().to_string(),
                os_icon: get_os_icon(d.os.as_str()),
                trusted_at: d.trusted_at.format("%Y-%m-%d").to_string(),
                last_connected: d.last_connected.map(|dt| dt.format("%Y-%m-%d %H:%M").to_string()).unwrap_or_else(|| "Never".to_string()),
                is_expired: d.is_expired(),
            }
        }).collect();
        trusted.set(display);
    }
    
    // Get active transfers
    let manager = crate::file_share::transfer::get_transfer_manager();
    if let Ok(m) = manager.lock() {
        let display: Vec<TransferDisplay> = m.get_all_transfers().iter().map(|t| {
            let is_sending = matches!(t.direction, crate::file_share::transfer::TransferDirection::Sending);
            let is_pending = matches!(t.status, crate::file_share::transfer::TransferStatus::Pending);
            let is_active = matches!(t.status, crate::file_share::transfer::TransferStatus::InProgress);
            
            TransferDisplay {
                id: t.id.clone(),
                filename: t.filename.clone(),
                size: crate::file_share::transfer::format_file_size(t.size),
                transferred: crate::file_share::transfer::format_file_size(t.transferred),
                progress: t.progress_percent(),
                speed: t.speed_display(),
                eta: t.eta_display(),
                direction: if is_sending { "Sending".to_string() } else { "Receiving".to_string() },
                direction_icon: if is_sending { "📤".to_string() } else { "📥".to_string() },
                status: format!("{:?}", t.status),
                status_color: match &t.status {
                    crate::file_share::transfer::TransferStatus::Pending => "#f59e0b".to_string(),
                    crate::file_share::transfer::TransferStatus::InProgress => "#3b82f6".to_string(),
                    crate::file_share::transfer::TransferStatus::Completed => "#4ade80".to_string(),
                    crate::file_share::transfer::TransferStatus::Failed(_) => "#ef4444".to_string(),
                    crate::file_share::transfer::TransferStatus::Cancelled => "#64748b".to_string(),
                    crate::file_share::transfer::TransferStatus::Paused => "#94a3b8".to_string(),
                },
                is_pending,
                is_active,
                is_incoming: !is_sending,
                can_cancel: is_active || is_pending,
            }
        }).collect();
        transfers.set(display);
    };
}

fn get_os_icon(os: &str) -> String {
    match os.to_lowercase().as_str() {
        "windows" => "🪟".to_string(),
        "macos" | "mac" => "🍎".to_string(),
        "linux" => "🐧".to_string(),
        _ => "💻".to_string(),
    }
}

async fn start_scan_async() -> Result<(), String> {
    crate::file_share::discovery::start_discovery().await
}

async fn stop_discovery_async() -> Result<(), String> {
    crate::file_share::discovery::stop_discovery()
}

async fn connect_to_device_async(device_id: &str) -> Result<(), String> {
    let devices = crate::file_share::discovery::get_discovered_devices()?;
    let device = devices.iter().find(|d| d.id == device_id)
        .ok_or_else(|| "Device not found".to_string())?;
    crate::file_share::bridge::connect_to_device(device)
}

// NOTE: This function is deprecated and will be replaced by ConnectionCoordinator
// in task 6. It uses the old verification code system that has been removed from TrustManager.
/*
async fn generate_pairing_code_async(device_id: &str) -> Result<String, String> {
    let code = crate::file_share::trust::generate_verification_code(device_id)?;
    Ok(code.code)
}
*/

async fn remove_trusted_async(device_id: &str) -> Result<(), String> {
    crate::file_share::trust::remove_trusted(device_id)?;
    Ok(())
}

async fn cancel_transfer_async(transfer_id: &str) -> Result<(), String> {
    crate::file_share::transfer::cancel_file_transfer(transfer_id)
}

async fn accept_transfer_async(transfer_id: &str) -> Result<(), String> {
    crate::file_share::transfer::accept_incoming_transfer(transfer_id)
}

async fn reject_transfer_async(transfer_id: &str) -> Result<(), String> {
    crate::file_share::transfer::reject_incoming_transfer(transfer_id, "User rejected")
}

async fn send_file_async(device_id: &str, file_path: &str) -> Result<(), String> {
    let path = std::path::Path::new(file_path);
    crate::file_share::transfer::send_file(device_id, path)?;
    Ok(())
}

/// Result of a code connection
struct CodeConnectionResult {
    device_label: String,
}

/// Connect to a device directly using IP address (bypassing code system)
async fn connect_direct_async(ip_address: &str, bridge_port: u16, device_label: &str) -> Result<CodeConnectionResult, String> {
    use std::sync::Arc;
    
    // Get the connection coordinator
    let coordinator = get_connection_coordinator()
        .map_err(|e| format!("Failed to get coordinator: {}", e))?;
    
    // Connect directly using IP
    let result = coordinator.connect_direct(ip_address, bridge_port, device_label).await
        .map_err(|e| e.user_message())?;
    
    println!("[FileShare] Direct connection successful to: {} (type: {:?})", 
        result.device.label, result.connection_type);
    
    Ok(CodeConnectionResult {
        device_label: result.device.label,
    })
}

/// Get or create the ConnectionCoordinator instance
fn get_connection_coordinator() -> Result<std::sync::Arc<crate::file_share::connection::ConnectionCoordinator>, String> {
    use std::sync::Mutex;
    use lazy_static::lazy_static;
    
    lazy_static! {
        static ref COORDINATOR: Mutex<Option<std::sync::Arc<crate::file_share::connection::ConnectionCoordinator>>> = Mutex::new(None);
    }
    
    let mut coord_lock = COORDINATOR.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    if let Some(ref coordinator) = *coord_lock {
        return Ok(std::sync::Arc::clone(coordinator));
    }
    
    // Use the GLOBAL relay service instead of creating a new one
    let relay = crate::file_share::relay::get_relay_service();
    
    let trust = std::sync::Arc::new(Mutex::new(crate::file_share::trust::TrustManager::new()));
    
    // Get the GLOBAL discovery service (not None!)
    let discovery = crate::file_share::discovery::get_discovery_service()
        .map_err(|e| format!("Failed to get discovery service: {}", e))?;
    
    let coordinator = std::sync::Arc::new(crate::file_share::connection::ConnectionCoordinator::new(
        relay,
        trust,
        discovery,
    ));
    
    *coord_lock = Some(std::sync::Arc::clone(&coordinator));
    
    Ok(coordinator)
}
