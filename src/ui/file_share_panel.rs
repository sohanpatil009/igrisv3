use dioxus::prelude::*;
use crate::file_share_client::{FileShareClient, Device, Transfer};
use crate::file_share_notifications;
use rfd::AsyncFileDialog;

#[component]
pub fn FileSharePanel() -> Element {
    let mut devices = use_signal(|| Vec::<Device>::new());
    let mut transfers = use_signal(|| Vec::<Transfer>::new());
    let mut selected_device = use_signal(|| None::<String>);
    let mut status_message = use_signal(|| String::from("Initializing..."));
    let mut sending = use_signal(|| false);

    // Check if backend is running
    let backend_running = use_resource(move || async move {
        let client = FileShareClient::new(53317);
        client.is_running().await
    });

    // Fetch devices periodically
    use_effect(move || {
        spawn(async move {
            loop {
                let client = FileShareClient::new(53317);
                if let Ok(devs) = client.get_devices().await {
                    devices.set(devs);
                    status_message.set(format!("Found {} devices", devices().len()));
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        });
    });

    // Fetch transfers periodically
    use_effect(move || {
        spawn(async move {
            loop {
                let client = FileShareClient::new(53317);
                if let Ok(trans) = client.get_transfers().await {
                    transfers.set(trans);
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });
    });

    rsx! {
        div {
            class: "file-share-panel",
            style: "padding: 20px; background: #1a1a2e; border-radius: 12px; color: white;",

            // Header
            div {
                style: "margin-bottom: 20px;",
                h2 {
                    style: "margin: 0 0 10px 0; color: #a855f7;",
                    "📡 File Share"
                }
                p {
                    style: "margin: 0; color: #888; font-size: 14px;",
                    "{status_message}"
                }
            }

            // Backend status
            if let Some(running) = backend_running() {
                if !running {
                    div {
                        style: "padding: 15px; background: #ff4444; border-radius: 8px; margin-bottom: 20px;",
                        "⚠️ Go backend not running. Start it with: ./fileshare"
                    }
                }
            }

            // Devices section
            div {
                style: "margin-bottom: 30px;",
                h3 {
                    style: "margin: 0 0 15px 0; color: #06b6d4; font-size: 18px;",
                    "Nearby Devices ({devices().len()})"
                }

                if devices().is_empty() {
                    div {
                        style: "padding: 20px; background: #0a0a0a; border-radius: 8px; text-align: center; color: #666;",
                        "No devices found. Make sure both devices are on the same mobile hotspot."
                    }
                } else {
                    div {
                        style: "display: grid; gap: 10px;",
                        for device in devices() {
                            {
                                let device_id = device.id.clone();
                                let device_alias = device.alias.clone();
                                let device_ip = device.ip.clone();
                                let device_port = device.port;
                                let device_type = device.device_type.clone();
                                
                                let device_id_for_select = device_id.clone();
                                let device_id_for_button = device_id.clone();
                                
                                let is_selected = selected_device() == Some(device_id.clone());
                                let border_style = if is_selected {
                                    "padding: 15px; background: #0a0a0a; border-radius: 8px; border: 2px solid #a855f7;"
                                } else {
                                    "padding: 15px; background: #0a0a0a; border-radius: 8px; border: 2px solid transparent;"
                                };
                                
                                rsx! {
                                    div {
                                        key: "{device_id}",
                                        style: "{border_style}",

                                        div {
                                            style: "display: flex; justify-content: space-between; align-items: center;",
                                            div {
                                                style: "cursor: pointer; flex: 1;",
                                                onclick: move |_| {
                                                    selected_device.set(Some(device_id_for_select.clone()));
                                                },
                                                div {
                                                    style: "font-weight: bold; font-size: 16px; margin-bottom: 5px;",
                                                    "🖥️ {device_alias}"
                                                }
                                                div {
                                                    style: "font-size: 12px; color: #888;",
                                                    "{device_ip}:{device_port} • {device_type}"
                                                }
                                            }
                                            div {
                                                style: "display: flex; gap: 10px; align-items: center;",
                                                div {
                                                    style: "padding: 5px 10px; background: #16a34a; border-radius: 4px; font-size: 12px;",
                                                    "Online"
                                                }
                                                button {
                                                    style: "padding: 8px 16px; background: linear-gradient(135deg, #a855f7, #06b6d4); border: none; border-radius: 6px; color: white; cursor: pointer; font-weight: bold; font-size: 14px;",
                                                    disabled: sending(),
                                                    onclick: move |_| {
                                                        let device_id_clone = device_id_for_button.clone();
                                                        let device_alias_clone = device_alias.clone();
                                                        spawn(async move {
                                                            sending.set(true);
                                                            
                                                            if let Some(file) = AsyncFileDialog::new()
                                                                .set_title(&format!("Send file to {}", device_alias_clone))
                                                                .pick_file()
                                                                .await 
                                                            {
                                                                let path = file.path().to_string_lossy().to_string();
                                                                let file_name = file.file_name();
                                                                status_message.set(format!("Sending file to {}...", device_alias_clone));
                                                                
                                                                file_share_notifications::notify_transfer_started(&device_alias_clone, &file_name);
                                                                
                                                                let client = FileShareClient::new(53317);
                                                                match client.send_file(&device_id_clone, &path).await {
                                                                    Ok(session_id) => {
                                                                        let success_msg = format!("✓ File sent successfully! Session: {}", session_id);
                                                                        status_message.set(success_msg.clone());
                                                                        file_share_notifications::notify_transfer_completed(&device_alias_clone, &file_name);
                                                                    }
                                                                    Err(e) => {
                                                                        let error_msg = format!("✗ Failed to send file: {}", e);
                                                                        status_message.set(error_msg.clone());
                                                                        file_share_notifications::notify_transfer_failed(&device_alias_clone, &file_name, &e);
                                                                    }
                                                                }
                                                            } else {
                                                                status_message.set("File selection cancelled".to_string());
                                                            }
                                                            
                                                            sending.set(false);
                                                        });
                                                    },
                                                    if sending() {
                                                        "⏳ Sending..."
                                                    } else {
                                                        "📤 Send File"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Transfers section
            div {
                h3 {
                    style: "margin: 0 0 15px 0; color: #06b6d4; font-size: 18px;",
                    "Active Transfers ({transfers().len()})"
                }

                if transfers().is_empty() {
                    div {
                        style: "padding: 20px; background: #0a0a0a; border-radius: 8px; text-align: center; color: #666;",
                        "No active transfers"
                    }
                } else {
                    div {
                        style: "display: grid; gap: 10px;",
                        for transfer in transfers() {
                            {
                                let session_id = transfer.session_id.clone();
                                let from_device = transfer.from_device.clone();
                                let status = transfer.status.clone();
                                let bytes_sent = transfer.bytes_sent;
                                let total_bytes = transfer.total_bytes;
                                let progress = transfer.progress();
                                
                                let status_color = match status.as_str() {
                                    "completed" => "#16a34a",
                                    "in_progress" => "#06b6d4",
                                    "failed" => "#ff4444",
                                    _ => "#888",
                                };
                                
                                rsx! {
                                    div {
                                        key: "{session_id}",
                                        style: "padding: 15px; background: #0a0a0a; border-radius: 8px;",

                                        div {
                                            style: "display: flex; justify-content: space-between; margin-bottom: 10px;",
                                            div {
                                                style: "font-weight: bold;",
                                                "From: {from_device}"
                                            }
                                            div {
                                                style: "color: {status_color};",
                                                "{status}"
                                            }
                                        }

                                        div {
                                            style: "width: 100%; height: 8px; background: #333; border-radius: 4px; overflow: hidden;",
                                            div {
                                                style: "height: 100%; background: linear-gradient(90deg, #a855f7, #06b6d4); width: {progress}%;",
                                            }
                                        }

                                        div {
                                            style: "margin-top: 5px; font-size: 12px; color: #888;",
                                            "{bytes_sent / 1024 / 1024} MB / {total_bytes / 1024 / 1024} MB ({progress:.1}%)"
                                        }

                                        if status == "in_progress" {
                                            button {
                                                style: "margin-top: 10px; padding: 8px 16px; background: #ff4444; border: none; border-radius: 4px; color: white; cursor: pointer;",
                                                onclick: move |_| {
                                                    let session_id_clone = session_id.clone();
                                                    spawn(async move {
                                                        let client = FileShareClient::new(53317);
                                                        let _ = client.cancel_transfer(&session_id_clone).await;
                                                    });
                                                },
                                                "Cancel"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Instructions
            div {
                style: "margin-top: 30px; padding: 15px; background: #0a0a0a; border-radius: 8px; border-left: 4px solid #a855f7;",
                h4 {
                    style: "margin: 0 0 10px 0; color: #a855f7;",
                    "Voice Commands"
                }
                div {
                    style: "font-size: 14px; color: #888; line-height: 1.6;",
                    "• \"Show nearby devices\""
                    br {}
                    "• \"Share file with [device name]\""
                    br {}
                    "• \"Show transfers\""
                    br {}
                    "• \"Cancel transfer\""
                }
            }
        }
    }
}
