use dioxus::prelude::*;
use rfd::AsyncFileDialog;
use crate::fastswap::{Device, FileProgress, ProgressStatus};

#[component]
pub fn FastSwapPanel() -> Element {
    let devices = use_signal(|| Vec::<Device>::new());
    let is_scanning = use_signal(|| false);
    let active_transfers = use_signal(|| Vec::<FileProgress>::new());
    let status_message = use_signal(|| String::from("FastSwap Ready"));
    let mut selected_device = use_signal(|| None::<Device>);

    // Auto-scan on mount
    use_effect(move || {
        spawn(async move {
            scan_for_devices(devices, is_scanning, status_message).await;
        });
    });

    // Periodic refresh of devices and transfers
    use_effect(move || {
        spawn(async move {
            loop {
                async_std::task::sleep(std::time::Duration::from_secs(5)).await;
                if !is_scanning() {
                    scan_for_devices(devices, is_scanning, status_message).await;
                }
            }
        });
    });

    rsx! {
        div {
            class: "fastswap-panel",
            style: "padding: 24px; background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%); border-radius: 16px; color: white; max-height: 80vh; overflow-y: auto;",

            // Header
            div {
                style: "margin-bottom: 24px; border-bottom: 2px solid rgba(168, 85, 247, 0.3); padding-bottom: 16px;",
                div {
                    style: "display: flex; align-items: center; justify-content: space-between;",
                    h2 {
                        style: "margin: 0; color: #a855f7; font-size: 28px; font-weight: bold;",
                        "⚡ FastSwap"
                    }
                    button {
                        style: "padding: 8px 16px; background: rgba(168, 85, 247, 0.2); border: 1px solid #a855f7; border-radius: 8px; color: #a855f7; cursor: pointer; font-size: 14px; transition: all 0.3s;",
                        onclick: move |_| {
                            spawn(async move {
                                scan_for_devices(devices, is_scanning, status_message).await;
                            });
                        },
                        disabled: is_scanning(),
                        if is_scanning() {
                            "🔄 Scanning..."
                        } else {
                            "🔍 Scan Network"
                        }
                    }
                }
                p {
                    style: "margin: 8px 0 0 0; color: #888; font-size: 14px;",
                    "{status_message}"
                }
            }

            // Device List
            div {
                style: "margin-bottom: 24px;",
                h3 {
                    style: "margin: 0 0 12px 0; color: #e9d5ff; font-size: 18px;",
                    "📱 Nearby Devices ({devices().len()})"
                }
                
                if devices().is_empty() {
                    div {
                        style: "padding: 32px; text-align: center; background: rgba(0, 0, 0, 0.3); border-radius: 12px; border: 2px dashed rgba(168, 85, 247, 0.3);",
                        div {
                            style: "font-size: 48px; margin-bottom: 16px;",
                            "🔍"
                        }
                        p {
                            style: "margin: 0; color: #888; font-size: 16px;",
                            if is_scanning() {
                                "Scanning network for devices..."
                            } else {
                                "No devices found. Click 'Scan Network' to search."
                            }
                        }
                    }
                } else {
                    div {
                        style: "display: grid; gap: 12px;",
                        for device in devices().iter() {
                            div {
                                key: "{device.id}",
                                style: "padding: 16px; background: rgba(168, 85, 247, 0.1); border: 1px solid rgba(168, 85, 247, 0.3); border-radius: 12px; cursor: pointer; transition: all 0.3s; hover:background: rgba(168, 85, 247, 0.2);",
                                onclick: {
                                    let dev = device.clone();
                                    let active_transfers = active_transfers;
                                    let status_message = status_message;
                                    move |_| {
                                        selected_device.set(Some(dev.clone()));
                                        let dev_clone = dev.clone();
                                        spawn(async move {
                                            send_files_to_device(dev_clone, active_transfers, status_message).await;
                                        });
                                    }
                                },
                                
                                div {
                                    style: "display: flex; align-items: center; gap: 12px;",
                                    div {
                                        style: "font-size: 32px;",
                                        match device.device_type {
                                            crate::fastswap::DeviceType::Mobile => "📱",
                                            crate::fastswap::DeviceType::Desktop => "💻",
                                            crate::fastswap::DeviceType::Web => "🌐",
                                            crate::fastswap::DeviceType::Headless => "🖥️",
                                        }
                                    }
                                    div {
                                        style: "flex: 1;",
                                        div {
                                            style: "font-size: 16px; font-weight: bold; color: #e9d5ff; margin-bottom: 4px;",
                                            "{device.alias}"
                                        }
                                        div {
                                            style: "font-size: 12px; color: #888;",
                                            "{device.device_model} • {device.ip}:{device.port}"
                                        }
                                    }
                                    div {
                                        style: "padding: 6px 12px; background: rgba(34, 197, 94, 0.2); border: 1px solid #22c55e; border-radius: 6px; font-size: 12px; color: #22c55e;",
                                        "Send Files"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Active Transfers
            if !active_transfers().is_empty() {
                div {
                    style: "margin-top: 24px; padding-top: 24px; border-top: 2px solid rgba(168, 85, 247, 0.3);",
                    h3 {
                        style: "margin: 0 0 12px 0; color: #e9d5ff; font-size: 18px;",
                        "📤 Active Transfers"
                    }
                    
                    div {
                        style: "display: grid; gap: 12px;",
                        for transfer in active_transfers().iter() {
                            div {
                                key: "{transfer.file_id}",
                                style: "padding: 16px; background: rgba(0, 0, 0, 0.3); border: 1px solid rgba(168, 85, 247, 0.3); border-radius: 12px;",
                                
                                div {
                                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;",
                                    div {
                                        style: "font-size: 14px; font-weight: bold; color: #e9d5ff;",
                                        "📄 {transfer.file_name}"
                                    }
                                    div {
                                        style: format!(
                                            "font-size: 12px; color: {};",
                                            match transfer.status {
                                                ProgressStatus::Completed => "#22c55e",
                                                ProgressStatus::Failed(_) => "#ef4444",
                                                ProgressStatus::Cancelled => "#f59e0b",
                                                _ => "#a855f7",
                                            }
                                        ),
                                        {
                                            match &transfer.status {
                                                ProgressStatus::Pending => "⏳ Pending".to_string(),
                                                ProgressStatus::Transferring => "🔄 Transferring".to_string(),
                                                ProgressStatus::Completed => "✅ Completed".to_string(),
                                                ProgressStatus::Failed(e) => format!("❌ Failed: {}", e),
                                                ProgressStatus::Cancelled => "🚫 Cancelled".to_string(),
                                            }
                                        }
                                    }
                                }
                                
                                // Progress bar
                                div {
                                    style: "width: 100%; height: 8px; background: rgba(0, 0, 0, 0.5); border-radius: 4px; overflow: hidden; margin-bottom: 8px;",
                                    div {
                                        style: format!(
                                            "height: 100%; background: linear-gradient(90deg, #a855f7, #7c3aed); width: {}%; transition: width 0.3s;",
                                            transfer.progress_percent()
                                        ),
                                    }
                                }
                                
                                // Transfer stats
                                div {
                                    style: "display: flex; justify-content: space-between; font-size: 12px; color: #888;",
                                    div {
                                        "{format_bytes(transfer.bytes_sent)} / {format_bytes(transfer.total_bytes)} ({transfer.progress_percent():.1}%)"
                                    }
                                    div {
                                        "{transfer.format_speed()} • ETA: {transfer.format_eta()}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Server Info
            div {
                style: "margin-top: 24px; padding: 16px; background: rgba(0, 0, 0, 0.3); border-radius: 12px; border-left: 4px solid #a855f7;",
                h4 {
                    style: "margin: 0 0 8px 0; color: #a855f7; font-size: 14px;",
                    "ℹ️ Server Information"
                }
                div {
                    style: "font-size: 13px; color: #888; line-height: 1.8;",
                    "• Server running on port 53317"
                    br {}
                    "• Compatible with LocalSend v2.0 protocol"
                    br {}
                    "• Supports cross-platform file sharing"
                    br {}
                    "• Other devices can send files to this device"
                }
            }
        }
    }
}

// Helper function to scan for devices
async fn scan_for_devices(
    mut devices: Signal<Vec<Device>>,
    mut is_scanning: Signal<bool>,
    mut status_message: Signal<String>,
) {
    is_scanning.set(true);
    status_message.set("Scanning network for devices...".to_string());
    
    // Get local IP
    let local_ip = local_ip_address::local_ip()
        .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 100)))
        .to_string();
    
    // Create discovery service and scan
    let discovery = crate::fastswap::network::DiscoveryService::new();
    match discovery.scan_network(&local_ip).await {
        Ok(found_devices) => {
            devices.set(found_devices.clone());
            status_message.set(format!("Found {} device(s)", found_devices.len()));
        }
        Err(e) => {
            status_message.set(format!("Scan failed: {}", e));
        }
    }
    
    is_scanning.set(false);
}

// Helper function to send files to a device
async fn send_files_to_device(
    device: Device,
    mut active_transfers: Signal<Vec<FileProgress>>,
    mut status_message: Signal<String>,
) {
    // Open file picker
    let files = AsyncFileDialog::new()
        .set_title("Select files to send")
        .pick_files()
        .await;
    
    if let Some(files) = files {
        status_message.set(format!("Preparing to send {} file(s) to {}", files.len(), device.alias));
        
        // Create progress entries for each file
        let mut progress_list = Vec::new();
        for (i, file) in files.iter().enumerate() {
            let file_name = file.file_name();
            let file_size = file.read().await.len() as u64;
            
            let progress = FileProgress::new(
                format!("file_{}", i),
                file_name,
                file_size,
            );
            progress_list.push(progress);
        }
        
        active_transfers.set(progress_list);
        
        // TODO: Implement actual file sending using FastSwap client
        // For now, just simulate progress
        status_message.set(format!("Sending files to {}...", device.alias));
        
        // Note: Full implementation would use:
        // crate::fastswap::network::send_files(&device, files).await
    }
}

// Helper function to format bytes
fn format_bytes(bytes: u64) -> String {
    crate::fastswap::models::progress::format_bytes(bytes)
}
