// LocalShare Panel for IGRIS
// Integrated file sharing UI using Dioxus 0.7

use dioxus::prelude::*;
use crate::localshare::{Device, DiscoveryService, TransferClient, TransferProgress, ProgressStatus};
use std::sync::Arc;

#[component]
pub fn LocalSharePanel() -> Element {
    let mut devices = use_signal(|| Vec::<Device>::new());
    let mut status_message = use_signal(|| String::from("Ready to share files"));
    let mut is_scanning = use_signal(|| false);
    let mut current_transfer = use_signal(|| None::<String>); // Current session ID
    let mut transfer_progress = use_signal(|| None::<TransferProgress>);
    let progress_tracker = use_signal(|| crate::localshare::models::create_progress_tracker());

    // Scan for devices
    let scan_devices = move |_| {
        spawn(async move {
            is_scanning.set(true);
            status_message.set("Scanning for devices...".to_string());
            
            let discovery = DiscoveryService::new();
            let local_ip = local_ip_address::local_ip()
                .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 1)))
                .to_string();
            
            match discovery.scan_network(&local_ip).await {
                Ok(found) => {
                    devices.set(found.clone());
                    if found.is_empty() {
                        status_message.set("No devices found".to_string());
                    } else {
                        status_message.set(format!("Found {} device(s)", found.len()));
                    }
                }
                Err(e) => {
                    status_message.set(format!("Scan failed: {}", e));
                }
            }
            
            is_scanning.set(false);
        });
    };

    // Progress polling effect
    use_effect(move || {
        if let Some(session_id) = current_transfer() {
            spawn(async move {
                loop {
                    // Check if transfer is still active
                    let tracker = progress_tracker.read();
                    let tracker_guard = tracker.read().await;
                    
                    if let Some(progress) = tracker_guard.get(&session_id) {
                        let progress_clone = progress.clone();
                        drop(tracker_guard);
                        drop(tracker);
                        
                        // Update UI with progress
                        transfer_progress.set(Some(progress_clone.clone()));
                        
                        // Check if complete
                        if progress_clone.is_complete() {
                            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                            current_transfer.set(None);
                            transfer_progress.set(None);
                            
                            if progress_clone.is_cancelled {
                                status_message.set("Transfer cancelled".to_string());
                            } else {
                                status_message.set("✓ Transfer complete!".to_string());
                            }
                            break;
                        }
                    } else {
                        // Session not found, stop polling
                        current_transfer.set(None);
                        transfer_progress.set(None);
                        break;
                    }
                    
                    // Poll every 100ms for smooth updates
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            });
        }
    });

    // Send file handler
    let send_file = move |device: Device| {
        spawn(async move {
            // Open file picker
            let file_handle = rfd::AsyncFileDialog::new()
                .set_title("Select file to send")
                .pick_file()
                .await;
            
            if let Some(file) = file_handle {
                let file_path = file.path().to_path_buf();
                let file_name = file.file_name();
                
                status_message.set(format!("Sending {} to {}...", file_name, device.alias));
                
                // Create local device info
                let local_ip = local_ip_address::local_ip()
                    .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 1)))
                    .to_string();
                let local_device = Device::new_local(whoami::username(), 53317, local_ip);
                
                let tracker = progress_tracker.read();
                let client = TransferClient::new(Arc::clone(&tracker));
                
                match client.send_files(&device, vec![file_path], &local_device).await {
                    Ok(session_id) => {
                        current_transfer.set(Some(session_id.clone()));
                        status_message.set(format!("Transferring to {}...", device.alias));
                    }
                    Err(e) => {
                        status_message.set(format!("✗ Failed to send file: {}", e));
                        current_transfer.set(None);
                        transfer_progress.set(None);
                    }
                }
            }
        });
    };

    // Cancel transfer handler
    let cancel_transfer = move |_| {
        if let Some(session_id) = current_transfer() {
            spawn(async move {
                let tracker = progress_tracker.read();
                let mut tracker_guard = tracker.write().await;
                if let Some(progress) = tracker_guard.get_mut(&session_id) {
                    progress.cancel();
                }
                status_message.set("Cancelling transfer...".to_string());
            });
        }
    };

    rsx! {
        div {
            style: "width: 100%; height: 100%; background: linear-gradient(135deg, #1e293b 0%, #0f172a 100%); color: white; padding: 24px; overflow-y: auto;",
            
            // Header
            div {
                style: "margin-bottom: 24px;",
                h2 {
                    style: "font-size: 28px; font-weight: bold; margin-bottom: 8px;",
                    "📁 LocalShare"
                }
                p {
                    style: "color: #94a3b8; font-size: 14px;",
                    "Share files with nearby devices"
                }
            }

            // Status bar
            div {
                style: "background: rgba(255, 255, 255, 0.05); border-radius: 12px; padding: 16px; margin-bottom: 24px; border: 1px solid rgba(255, 255, 255, 0.1);",
                p {
                    style: "font-size: 14px; color: #e2e8f0;",
                    "{status_message}"
                }
            }

            // Progress bar (shown when transfer is active)
            if let Some(progress) = transfer_progress() {
                div {
                    style: "background: rgba(59, 130, 246, 0.1); border-radius: 12px; padding: 20px; margin-bottom: 24px; border: 1px solid rgba(59, 130, 246, 0.3);",
                    
                    // Header
                    div {
                        style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;",
                        div {
                            style: "font-size: 14px; font-weight: 600; color: #93c5fd;",
                            "📤 Transfer in Progress"
                        }
                        button {
                            style: "background: rgba(239, 68, 68, 0.2); color: #fca5a5; padding: 6px 12px; border-radius: 6px; border: 1px solid rgba(239, 68, 68, 0.3); cursor: pointer; font-size: 12px;",
                            onclick: cancel_transfer,
                            "✕ Cancel"
                        }
                    }

                    // Overall progress bar
                    div {
                        style: "margin-bottom: 12px;",
                        div {
                            style: "background: rgba(255, 255, 255, 0.1); border-radius: 8px; height: 8px; overflow: hidden;",
                            div {
                                style: format!("background: linear-gradient(90deg, #3b82f6 0%, #2563eb 100%); height: 100%; width: {:.1}%; transition: width 0.3s;", progress.overall_progress()),
                            }
                        }
                    }

                    // Progress stats
                    div {
                        style: "display: flex; justify-content: space-between; font-size: 12px; color: #93c5fd; margin-bottom: 16px;",
                        div {
                            {format!("{:.1}% • {}/{}", 
                                progress.overall_progress(),
                                crate::localshare::models::format_bytes(progress.transferred_bytes),
                                crate::localshare::models::format_bytes(progress.total_bytes)
                            )}
                        }
                        div {
                            {
                                let remaining = progress.total_bytes.saturating_sub(progress.transferred_bytes);
                                let speed = progress.overall_speed();
                                let eta = if speed > 0.0 {
                                    let eta_secs = (remaining as f64 / speed) as u64;
                                    crate::localshare::models::format_duration(eta_secs)
                                } else {
                                    "Calculating...".to_string()
                                };
                                format!("{} • ETA: {}", 
                                    crate::localshare::models::format_bytes_per_second(speed),
                                    eta
                                )
                            }
                        }
                    }

                    // Individual file progress
                    if progress.files.len() > 1 {
                        div {
                            style: "border-top: 1px solid rgba(59, 130, 246, 0.2); padding-top: 12px;",
                            for file in progress.files.iter() {
                                div {
                                    key: "{file.file_id}",
                                    style: "margin-bottom: 8px;",
                                    
                                    div {
                                        style: "display: flex; justify-content: space-between; font-size: 11px; color: #93c5fd; margin-bottom: 4px;",
                                        div {
                                            style: "overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 60%;",
                                            "{file.file_name}"
                                        }
                                        div {
                                            {
                                                match &file.status {
                                                    ProgressStatus::Completed => "✓ Complete".to_string(),
                                                    ProgressStatus::Transferring => format!("{:.1}%", file.progress_percent()),
                                                    ProgressStatus::Pending => "Pending...".to_string(),
                                                    ProgressStatus::Failed(e) => format!("✗ Failed: {}", e),
                                                    ProgressStatus::Cancelled => "✗ Cancelled".to_string(),
                                                }
                                            }
                                        }
                                    }
                                    
                                    if matches!(file.status, ProgressStatus::Transferring) {
                                        div {
                                            style: "background: rgba(255, 255, 255, 0.1); border-radius: 4px; height: 4px; overflow: hidden;",
                                            div {
                                                style: format!("background: #3b82f6; height: 100%; width: {:.1}%; transition: width 0.2s;", file.progress_percent()),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Scan button
            div {
                style: "margin-bottom: 24px;",
                button {
                    style: "background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%); color: white; padding: 12px 24px; border-radius: 8px; border: none; cursor: pointer; font-size: 14px; font-weight: 600; transition: all 0.3s; width: 100%;",
                    onclick: scan_devices,
                    disabled: is_scanning(),
                    if is_scanning() {
                        "🔍 Scanning..."
                    } else {
                        "🔍 Scan for Devices"
                    }
                }
            }

            // Devices list
            div {
                style: "display: flex; flex-direction: column; gap: 12px;",
                
                if devices().is_empty() {
                    div {
                        style: "text-align: center; padding: 48px 24px; color: #64748b;",
                        p { "No devices found" }
                        p {
                            style: "font-size: 12px; margin-top: 8px;",
                            "Click 'Scan for Devices' to search"
                        }
                    }
                } else {
                    for device in devices().iter() {
                        {
                            let device_clone = device.clone();
                            rsx! {
                                div {
                                    key: "{device.id}",
                                    style: "background: rgba(255, 255, 255, 0.05); border-radius: 12px; padding: 16px; border: 1px solid rgba(255, 255, 255, 0.1); transition: all 0.3s; cursor: pointer;",
                                    onmouseenter: move |_| {
                                        // Hover effect handled by CSS
                                    },
                                    
                                    div {
                                        style: "display: flex; justify-content: space-between; align-items: center;",
                                        
                                        div {
                                            style: "flex: 1;",
                                            div {
                                                style: "font-size: 16px; font-weight: 600; margin-bottom: 4px;",
                                                "💻 {device.alias}"
                                            }
                                            div {
                                                style: "font-size: 12px; color: #94a3b8;",
                                                "{device.device_model} • {device.ip}:{device.port}"
                                            }
                                        }
                                        
                                        button {
                                            style: "background: linear-gradient(135deg, #10b981 0%, #059669 100%); color: white; padding: 8px 16px; border-radius: 6px; border: none; cursor: pointer; font-size: 12px; font-weight: 600;",
                                            onclick: move |_| {
                                                send_file(device_clone.clone());
                                            },
                                            "📤 Send File"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Info section
            div {
                style: "margin-top: 32px; padding: 16px; background: rgba(59, 130, 246, 0.1); border-radius: 12px; border: 1px solid rgba(59, 130, 246, 0.2);",
                p {
                    style: "font-size: 12px; color: #93c5fd; margin-bottom: 8px;",
                    "ℹ️ LocalShare is compatible with LocalSend protocol"
                }
                p {
                    style: "font-size: 11px; color: #60a5fa;",
                    "Make sure devices are on the same network"
                }
            }
        }

        // Hover styles
        style { "
            button:hover {{
                transform: translateY(-2px);
                box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
            }}
            button:active {{
                transform: translateY(0);
            }}
            button:disabled {{
                opacity: 0.5;
                cursor: not-allowed;
            }}
        " }
    }
}
