// Dioxus 0.7 UI component for P2P file sharing - LocalSend Protocol
//
// Features:
// - Device discovery list
// - Transfer progress tracking
// - File picker integration

use dioxus::prelude::*;
use std::sync::Arc;
use tokio::sync::RwLock;

// Import from file_share module
use crate::file_share::{
    FileShareManager, Device, TransferProgress, TransferStatus
};
use crate::ui::FilePicker;

#[derive(Clone, PartialEq)]
struct IncomingTransfer {
    session_id: String,
    device_name: String,
    device_id: String,
    files: Vec<String>,
    total_size: u64,
}

#[component]
pub fn FileSharePanel() -> Element {
    // State
    let mut devices = use_signal(|| Vec::<Device>::new());
    let mut show_file_picker = use_signal(|| false);
    let mut selected_device_id = use_signal(|| None::<String>);
    let mut error_message = use_signal(|| None::<String>);
    let mut selected_files = use_signal(|| Vec::<String>::new());
    let mut pending_transfer = use_signal(|| None::<IncomingTransfer>);
    let mut is_initializing = use_signal(|| true);
    
    // Get FileShareManager from context (provided by parent)
    let file_share = use_context::<Signal<Option<Arc<RwLock<FileShareManager>>>>>();
    
    // Refresh devices periodically
    use_effect(move || {
        spawn(async move {
            // Wait for FileShareManager to be initialized
            loop {
                let fs_signal = file_share();
                if fs_signal.is_some() {
                    *is_initializing.write() = false;
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            
            // Now start refreshing devices
            loop {
                let fs_signal = file_share();
                if let Some(fs_arc) = fs_signal {
                    let fs_lock: tokio::sync::RwLockReadGuard<FileShareManager> = fs_arc.read().await;
                    *devices.write() = fs_lock.get_devices().await;
                    drop(fs_lock);
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        });
    });

    rsx! {
        div {
            class: "file-share-panel",
            style: "padding: 20px; background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%); min-height: 100vh;",
            
            // Header
            div {
                class: "panel-header",
                style: "margin-bottom: 30px;",
                h1 {
                    style: "color: #a855f7; font-size: 28px; margin: 0 0 10px 0; font-weight: 600;",
                    "📁 P2P File Sharing"
                }
                p {
                    style: "color: #94a3b8; font-size: 14px; margin: 0;",
                    "Secure offline file transfer over local network"
                }
            }
            
            // Initializing state
            if is_initializing() {
                div {
                    style: "padding: 60px 20px; text-align: center;",
                    div {
                        style: "font-size: 48px; margin-bottom: 15px; animation: pulse 2s ease-in-out infinite;",
                        "⚙️"
                    }
                    p {
                        style: "color: #a855f7; font-size: 18px; margin: 0 0 10px 0; font-weight: 600;",
                        "Initializing File Share Service..."
                    }
                    p {
                        style: "color: #64748b; font-size: 14px; margin: 0;",
                        "Starting mDNS discovery and HTTP server"
                    }
                }
            } else {
                // Error message
                if let Some(error) = error_message() {
                    div {
                        style: "padding: 15px; background: rgba(239, 68, 68, 0.1); border: 1px solid #ef4444; border-radius: 8px; margin-bottom: 20px;",
                        p {
                            style: "color: #ef4444; margin: 0;",
                            "⚠️ {error}"
                        }
                        button {
                            style: "margin-top: 10px; padding: 5px 10px; background: transparent; border: 1px solid #ef4444; color: #ef4444; border-radius: 4px; cursor: pointer;",
                            onclick: move |_| *error_message.write() = None,
                            "Dismiss"
                        }
                    }
                }
                
                // File Picker Dialog
                if show_file_picker() {
                    FilePicker {
                        on_files_selected: move |files: Vec<String>| {
                            *selected_files.write() = files.clone();
                            *show_file_picker.write() = false;
                            
                            // Send files to selected device
                            if let Some(device_id) = selected_device_id() {
                                let fs = file_share.clone();
                                let mut err = error_message.clone();
                                spawn(async move {
                                    let fs_signal = fs();
                                    if let Some(fs_arc) = fs_signal {
                                        let fs_lock: tokio::sync::RwLockReadGuard<FileShareManager> = fs_arc.read().await;
                                        if let Err(e) = fs_lock.send_files(&device_id, files).await {
                                            *err.write() = Some(format!("Failed to send files: {}", e));
                                        }
                                    }
                                });
                            }
                        },
                        on_close: move |_| {
                            *show_file_picker.write() = false;
                        }
                    }
                }
                
                // Devices Section
                div {
                    class: "devices-section",
                    style: "margin-bottom: 30px;",
                    
                    div {
                        style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px;",
                        h2 {
                            style: "color: #e2e8f0; font-size: 20px; margin: 0;",
                            "Nearby Devices ({devices.read().len()})"
                        }
                    }
                    
                    div {
                        class: "devices-grid",
                        style: "display: grid; gap: 12px;",
                        
                        if devices.read().is_empty() {
                            div {
                                style: "padding: 60px 20px; text-align: center; background: rgba(255,255,255,0.03); border-radius: 12px; border: 2px dashed rgba(168,85,247,0.3);",
                                div {
                                    style: "font-size: 48px; margin-bottom: 15px;",
                                    "🔍"
                                }
                                p {
                                    style: "color: #64748b; font-size: 16px; margin: 0;",
                                    "Searching for devices on local network..."
                                }
                                p {
                                    style: "color: #475569; font-size: 14px; margin-top: 10px;",
                                    "Make sure devices are on the same WiFi or hotspot"
                                }
                            }
                        }
                        
                        for device in devices.read().iter() {
                            DeviceCard {
                                device: device.clone(),
                                on_send: move |device_id: String| {
                                    *selected_device_id.write() = Some(device_id);
                                    *show_file_picker.write() = true;
                                }
                            }
                        }
                    }
                }
                
                // Incoming Transfer Approval Dialog
                if let Some(transfer) = pending_transfer() {
                    ApprovalDialog {
                        transfer: transfer.clone(),
                        on_accept: move |session_id: String| {
                            let fs = file_share.clone();
                            spawn(async move {
                                let fs_signal = fs();
                                if let Some(fs_arc) = fs_signal {
                                    let fs_lock: tokio::sync::RwLockReadGuard<FileShareManager> = fs_arc.read().await;
                                    // Accept the transfer
                                    // TODO: Call accept method when implemented
                                }
                            });
                            *pending_transfer.write() = None;
                        },
                        on_reject: move |session_id: String| {
                            let fs = file_share.clone();
                            spawn(async move {
                                let fs_signal = fs();
                                if let Some(fs_arc) = fs_signal {
                                    let fs_lock: tokio::sync::RwLockReadGuard<FileShareManager> = fs_arc.read().await;
                                    // Reject the transfer
                                    // TODO: Call reject method when implemented
                                }
                            });
                            *pending_transfer.write() = None;
                        }
                    }
                }
            }
        }
    }
}


#[component]
fn DeviceCard(
    device: Device,
    on_send: EventHandler<String>,
) -> Element {
    rsx! {
        div {
            class: "device-card",
            style: "
                padding: 20px;
                background: rgba(255,255,255,0.05);
                border: 1px solid rgba(168,85,247,0.3);
                border-radius: 12px;
                transition: all 0.3s;
                cursor: pointer;
            ",
            
            div {
                style: "display: flex; justify-content: space-between; align-items: start; margin-bottom: 15px;",
                
                div {
                    style: "flex: 1;",
                    h3 {
                        style: "color: #e2e8f0; margin: 0 0 8px 0; font-size: 18px; display: flex; align-items: center; gap: 10px;",
                        span { "💻" }
                        span { "{device.alias}" }
                    }
                    p {
                        style: "color: #64748b; font-size: 12px; margin: 0 0 4px 0; font-family: monospace;",
                        "ID: {device.id.chars().take(8).collect::<String>()}..."
                    }
                    p {
                        style: "color: #64748b; font-size: 12px; margin: 0; font-family: monospace;",
                        "🔑 {device.fingerprint}"
                    }
                }
            }
            
            div {
                style: "display: flex; gap: 10px; align-items: center;",
                
                p {
                    style: "color: #94a3b8; font-size: 13px; margin: 0; flex: 1;",
                    "📡 {device.ip}:{device.port}"
                }
                
                button {
                    style: "
                        padding: 10px 20px;
                        background: linear-gradient(135deg, #a855f7, #7c3aed);
                        border: none;
                        color: white;
                        border-radius: 8px;
                        cursor: pointer;
                        font-size: 14px;
                        font-weight: 600;
                        transition: all 0.2s;
                    ",
                    onclick: move |e| {
                        e.stop_propagation();
                        on_send.call(device.id.clone());
                    },
                    "📤 Send File"
                }
            }
        }
    }
}


#[component]
fn ApprovalDialog(
    transfer: IncomingTransfer,
    on_accept: EventHandler<String>,
    on_reject: EventHandler<String>,
) -> Element {
    let session_id_accept = transfer.session_id.clone();
    let session_id_reject = transfer.session_id.clone();
    let session_id_reject_overlay = transfer.session_id.clone();
    
    rsx! {
        div {
            class: "approval-overlay",
            style: "
                position: fixed;
                top: 0;
                left: 0;
                right: 0;
                bottom: 0;
                background: rgba(0,0,0,0.85);
                display: flex;
                align-items: center;
                justify-content: center;
                z-index: 2000;
                backdrop-filter: blur(8px);
                animation: fadeIn 0.2s ease;
            ",
            onclick: move |_| on_reject.call(session_id_reject_overlay.clone()),
            
            div {
                class: "approval-dialog",
                style: "
                    background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
                    border: 2px solid #a855f7;
                    border-radius: 20px;
                    padding: 35px;
                    max-width: 550px;
                    width: 90%;
                    box-shadow: 0 25px 80px rgba(168,85,247,0.5);
                    animation: slideUp 0.3s ease;
                ",
                onclick: move |e| e.stop_propagation(),
                
                // Icon and Title
                div {
                    style: "text-align: center; margin-bottom: 30px;",
                    div {
                        style: "font-size: 72px; margin-bottom: 20px; animation: bounce 0.5s ease;",
                        "📥"
                    }
                    h2 {
                        style: "color: #e2e8f0; margin: 0 0 12px 0; font-size: 26px; font-weight: 700;",
                        "Incoming File Transfer"
                    }
                    p {
                        style: "color: #94a3b8; margin: 0; font-size: 15px;",
                        "Someone wants to send you files"
                    }
                }
                
                // Transfer Details
                div {
                    style: "background: rgba(255,255,255,0.05); border-radius: 14px; padding: 24px; margin-bottom: 28px; border: 1px solid rgba(168,85,247,0.2);",
                    
                    div {
                        style: "margin-bottom: 18px;",
                        p {
                            style: "color: #64748b; font-size: 12px; margin: 0 0 6px 0; text-transform: uppercase; letter-spacing: 0.5px;",
                            "From Device"
                        }
                        p {
                            style: "color: #e2e8f0; font-size: 18px; margin: 0; font-weight: 600; display: flex; align-items: center; gap: 8px;",
                            span { style: "font-size: 20px;", "💻" }
                            span { "{transfer.device_name}" }
                        }
                        p {
                            style: "color: #64748b; font-size: 13px; margin: 4px 0 0 28px; font-family: monospace;",
                            "{transfer.device_id.chars().take(16).collect::<String>()}..."
                        }
                    }
                    
                    div {
                        style: "margin-bottom: 18px;",
                        p {
                            style: "color: #64748b; font-size: 12px; margin: 0 0 6px 0; text-transform: uppercase; letter-spacing: 0.5px;",
                            "Files ({transfer.files.len()})"
                        }
                        div {
                            style: "max-height: 120px; overflow-y: auto;",
                            for file in transfer.files.iter().take(5) {
                                p {
                                    style: "color: #cbd5e1; font-size: 14px; margin: 4px 0; padding: 6px 10px; background: rgba(255,255,255,0.03); border-radius: 6px;",
                                    "📄 {file}"
                                }
                            }
                            if transfer.files.len() > 5 {
                                p {
                                    style: "color: #64748b; font-size: 13px; margin: 8px 0 0 0; font-style: italic;",
                                    "+ {transfer.files.len() - 5} more files..."
                                }
                            }
                        }
                    }
                    
                    div {
                        p {
                            style: "color: #64748b; font-size: 12px; margin: 0 0 6px 0; text-transform: uppercase; letter-spacing: 0.5px;",
                            "Total Size"
                        }
                        p {
                            style: "color: #a855f7; font-size: 20px; margin: 0; font-weight: 700;",
                            "{format_bytes(transfer.total_size)}"
                        }
                    }
                }
                
                // Warning
                div {
                    style: "background: rgba(245, 158, 11, 0.1); border: 1px solid #f59e0b; border-radius: 10px; padding: 16px; margin-bottom: 28px;",
                    p {
                        style: "color: #fbbf24; font-size: 13px; margin: 0; line-height: 1.6; display: flex; align-items: start; gap: 8px;",
                        span { style: "font-size: 16px;", "⚠️" }
                        span { "Only accept files from devices you trust. Verify the device name matches the sender." }
                    }
                }
                
                // Action Buttons
                div {
                    style: "display: flex; gap: 14px;",
                    
                    button {
                        style: "
                            flex: 1;
                            padding: 16px 28px;
                            background: rgba(255,255,255,0.08);
                            color: #e2e8f0;
                            border: 1px solid rgba(255,255,255,0.15);
                            border-radius: 12px;
                            cursor: pointer;
                            font-size: 16px;
                            font-weight: 600;
                            transition: all 0.2s;
                        ",
                        onmouseenter: move |_| {},
                        onclick: move |_| on_reject.call(session_id_reject.clone()),
                        "✕ Reject"
                    }
                    
                    button {
                        style: "
                            flex: 1;
                            padding: 16px 28px;
                            background: linear-gradient(135deg, #10b981, #059669);
                            color: white;
                            border: none;
                            border-radius: 12px;
                            cursor: pointer;
                            font-size: 16px;
                            font-weight: 700;
                            transition: all 0.2s;
                            box-shadow: 0 6px 20px rgba(16, 185, 129, 0.4);
                        ",
                        onclick: move |_| on_accept.call(session_id_accept.clone()),
                        "✓ Accept & Download"
                    }
                }
            }
        }
    }
}


// Helper function

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
