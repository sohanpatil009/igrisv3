// src/commands/file_share.rs - File Share Voice Command Handler

use std::error::Error;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

// Import from parent crate's file_share module
use crate::file_share::{
    discovery::{start_discovery, get_discovered_devices},
    bridge::{connect_to_device, disconnect_from_device},
    trust::get_all_trusted,
    transfer::{
        accept_incoming_transfer, reject_incoming_transfer, 
        cancel_file_transfer, get_transfer_manager, TransferStatus, TransferDirection,
    },
};

// Import FileSharePanelState from UI module
use crate::ui::file_share::FileSharePanelState;

/// Global file share panel state - controls which UI view is shown
pub static FILE_SHARE_PANEL_STATE: Lazy<Arc<Mutex<FileSharePanelState>>> =
    Lazy::new(|| Arc::new(Mutex::new(FileSharePanelState::Hidden)));

/// File share action types
#[derive(Debug, Clone)]
pub enum FileShareAction {
    Scan,
    MyDevices,
    Connect { device: String },
    Disconnect { device: String },
    SendFile { device: String },
    ShowTransfers,
    AcceptTransfer,
    RejectTransfer,
    CancelTransfer,
}

/// Handle file share voice commands
pub async fn handle_file_share_command(
    action: &str,
    params: &std::collections::HashMap<String, String>,
) -> Result<String, Box<dyn Error>> {
    match action {
        "discover" | "scan" | "file_share_scan" => handle_scan().await,
        "list" | "show" => handle_show_devices(),
        "trusted" | "file_share_my_devices" => handle_my_devices(),
        "connect" | "file_share_connect" => {
            let device = params.get("device").map(|s| s.as_str()).unwrap_or("");
            handle_connect(device).await
        }
        "disconnect" | "file_share_disconnect" => {
            let device = params.get("device").map(|s| s.as_str()).unwrap_or("");
            handle_disconnect(device)
        }
        "send" | "file_share_send" => {
            let device = params.get("device").map(|s| s.as_str()).unwrap_or("");
            handle_send_file(device)
        }
        "transfers" | "file_share_transfers" => handle_show_transfers(),
        "accept" | "file_share_accept" => handle_accept_transfer(),
        "reject" | "file_share_reject" => handle_reject_transfer(),
        "cancel" | "file_share_cancel" => handle_cancel_transfer(),
        _ => Err(format!("Unknown file share action: {}", action).into()),
    }
}

/// Scan for nearby devices
async fn handle_scan() -> Result<String, Box<dyn Error>> {
    println!("[FileShare] Starting device scan...");
    
    // Start discovery
    match start_discovery().await {
        Ok(_) => println!("[FileShare] Discovery started successfully"),
        Err(e) => {
            println!("[FileShare] Discovery start error: {}", e);
            return Err(format!("Failed to start discovery: {}", e).into());
        }
    }
    
    // Wait a moment for initial discovery
    println!("[FileShare] Waiting 3 seconds for device responses...");
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    
    // Get discovered devices
    let devices = match get_discovered_devices() {
        Ok(d) => {
            println!("[FileShare] Retrieved {} devices from discovery service", d.len());
            for device in &d {
                println!("[FileShare]   - {} ({}) at {} - online: {}", 
                    device.label, device.id, device.ip_address, device.is_online());
            }
            d
        }
        Err(e) => {
            println!("[FileShare] Error getting devices: {}", e);
            return Err(format!("Failed to get devices: {}", e).into());
        }
    };
    
    // Open radar panel
    if let Ok(mut state) = FILE_SHARE_PANEL_STATE.lock() {
        *state = FileSharePanelState::Radar;
        println!("[FileShare] Radar panel state set");
    }
    
    if devices.is_empty() {
        Ok("Scanning for devices. No IGRIS devices found yet. Opening device radar.".to_string())
    } else {
        let count = devices.len();
        let names: Vec<String> = devices.iter().map(|d| d.label.clone()).collect();
        Ok(format!(
            "Found {} device{}. {}. Opening device radar.",
            count,
            if count == 1 { "" } else { "s" },
            names.join(", ")
        ))
    }
}

/// Show discovered devices (without scanning)
fn handle_show_devices() -> Result<String, Box<dyn Error>> {
    let devices = get_discovered_devices()?;
    
    // Open radar panel
    if let Ok(mut state) = FILE_SHARE_PANEL_STATE.lock() {
        *state = FileSharePanelState::Radar;
    }
    
    if devices.is_empty() {
        Ok("No devices found. Try scanning for devices first.".to_string())
    } else {
        let count = devices.len();
        let names: Vec<String> = devices.iter().map(|d| d.label.clone()).collect();
        Ok(format!(
            "Found {} device{}. {}. Opening device radar.",
            count,
            if count == 1 { "" } else { "s" },
            names.join(", ")
        ))
    }
}

/// Show trusted devices
fn handle_my_devices() -> Result<String, Box<dyn Error>> {
    let trusted = get_all_trusted()?;
    
    // Open my devices panel
    if let Ok(mut state) = FILE_SHARE_PANEL_STATE.lock() {
        *state = FileSharePanelState::MyDevices;
    }
    
    if trusted.is_empty() {
        Ok("You don't have any trusted devices yet. Connect to a device to add it.".to_string())
    } else {
        let count = trusted.len();
        let names: Vec<String> = trusted.iter().map(|d| d.label.clone()).collect();
        Ok(format!(
            "You have {} trusted device{}. {}. Opening my devices.",
            count,
            if count == 1 { "" } else { "s" },
            names.join(", ")
        ))
    }
}

/// Connect to a device
async fn handle_connect(device_name: &str) -> Result<String, Box<dyn Error>> {
    if device_name.is_empty() {
        return Ok("Which device would you like to connect to?".to_string());
    }
    
    // Find device by name
    let devices = get_discovered_devices()?;
    let device = devices.iter().find(|d| {
        d.label.to_lowercase().contains(&device_name.to_lowercase())
    });
    
    match device {
        Some(d) => {
            let label = d.label.clone();
            connect_to_device(d)?;
            Ok(format!("Connecting to {}...", label))
        }
        None => {
            Ok(format!(
                "Device '{}' not found. Try scanning for devices first.",
                device_name
            ))
        }
    }
}

/// Disconnect from a device
fn handle_disconnect(device_name: &str) -> Result<String, Box<dyn Error>> {
    if device_name.is_empty() {
        return Ok("Which device would you like to disconnect from?".to_string());
    }
    
    // Find device by name in trusted list
    let trusted = get_all_trusted()?;
    let device = trusted.iter().find(|d| {
        d.label.to_lowercase().contains(&device_name.to_lowercase())
    });
    
    match device {
        Some(d) => {
            let label = d.label.clone();
            disconnect_from_device(&d.id)?;
            Ok(format!("Disconnected from {}.", label))
        }
        None => {
            Ok(format!(
                "Device '{}' not found in your trusted devices.",
                device_name
            ))
        }
    }
}

/// Send file to a device
fn handle_send_file(device_name: &str) -> Result<String, Box<dyn Error>> {
    if device_name.is_empty() {
        return Ok("Which device would you like to send a file to?".to_string());
    }
    
    // Find device
    let trusted = get_all_trusted()?;
    let device = trusted.iter().find(|d| {
        d.label.to_lowercase().contains(&device_name.to_lowercase())
    });
    
    match device {
        Some(d) => {
            // In a real implementation, this would open a file picker
            // For now, we just acknowledge the command
            Ok(format!(
                "Opening file picker to send to {}. Select a file to transfer.",
                d.label
            ))
        }
        None => {
            Ok(format!(
                "Device '{}' not found. Make sure it's in your trusted devices.",
                device_name
            ))
        }
    }
}

/// Show transfer status
fn handle_show_transfers() -> Result<String, Box<dyn Error>> {
    let manager = get_transfer_manager();
    let manager = manager.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    let active = manager.get_active_transfers();
    
    // Open transfers panel
    if let Ok(mut state) = FILE_SHARE_PANEL_STATE.lock() {
        *state = FileSharePanelState::Transfers;
    }
    
    if active.is_empty() {
        Ok("No active transfers. Opening transfer panel.".to_string())
    } else {
        let count = active.len();
        Ok(format!(
            "{} active transfer{}. Opening transfer panel.",
            count,
            if count == 1 { "" } else { "s" }
        ))
    }
}

/// Accept pending transfer
fn handle_accept_transfer() -> Result<String, Box<dyn Error>> {
    let manager = get_transfer_manager();
    let manager = manager.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    // Find first pending incoming transfer
    let pending = manager.get_all_transfers()
        .iter()
        .find(|t| {
            t.status == TransferStatus::Pending 
            && t.direction == TransferDirection::Receiving
        })
        .map(|t| t.id.clone());
    
    drop(manager);
    
    match pending {
        Some(id) => {
            accept_incoming_transfer(&id)?;
            Ok("Transfer accepted. Receiving file...".to_string())
        }
        None => {
            Ok("No pending transfers to accept.".to_string())
        }
    }
}

/// Reject pending transfer
fn handle_reject_transfer() -> Result<String, Box<dyn Error>> {
    let manager = get_transfer_manager();
    let manager = manager.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    // Find first pending incoming transfer
    let pending = manager.get_all_transfers()
        .iter()
        .find(|t| {
            t.status == TransferStatus::Pending 
            && t.direction == TransferDirection::Receiving
        })
        .map(|t| t.id.clone());
    
    drop(manager);
    
    match pending {
        Some(id) => {
            reject_incoming_transfer(&id, "User rejected via voice command")?;
            Ok("Transfer rejected.".to_string())
        }
        None => {
            Ok("No pending transfers to reject.".to_string())
        }
    }
}

/// Cancel active transfer
fn handle_cancel_transfer() -> Result<String, Box<dyn Error>> {
    let manager = get_transfer_manager();
    let manager = manager.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    // Find first active transfer
    let active = manager.get_active_transfers()
        .first()
        .map(|t| t.id.clone());
    
    drop(manager);
    
    match active {
        Some(id) => {
            cancel_file_transfer(&id)?;
            Ok("Transfer cancelled.".to_string())
        }
        None => {
            Ok("No active transfers to cancel.".to_string())
        }
    }
}

/// Get friendly action description
pub fn get_action_description(action: &str) -> &'static str {
    match action {
        "discover" | "scan" | "file_share_scan" => "Scanning for nearby devices",
        "list" | "show" => "Showing discovered devices",
        "trusted" | "file_share_my_devices" => "Opening my devices",
        "connect" | "file_share_connect" => "Connecting to device",
        "disconnect" | "file_share_disconnect" => "Disconnecting from device",
        "send" | "file_share_send" => "Sending file",
        "transfers" | "file_share_transfers" => "Showing transfers",
        "accept" | "file_share_accept" => "Accepting transfer",
        "reject" | "file_share_reject" => "Rejecting transfer",
        "cancel" | "file_share_cancel" => "Cancelling transfer",
        _ => "File share action",
    }
}
