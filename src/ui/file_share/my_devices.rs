// src/ui/file_share/my_devices.rs - My Devices Settings UI

use dioxus::prelude::*;

/// Trusted device display info (self-contained)
#[derive(Clone, PartialEq, Debug)]
pub struct TrustedDeviceDisplay {
    pub id: String,
    pub label: String,
    pub os: String,
    pub os_icon: String,
    pub trusted_at: String,
    pub last_connected: String,
    pub is_expired: bool,
}

/// My Devices - Manage trusted devices
#[component]
pub fn MyDevices(
    devices: Vec<TrustedDeviceDisplay>,
    on_remove: EventHandler<String>,
    on_close: EventHandler<()>,
) -> Element {
    let mut editing_device = use_signal(|| None::<String>);
    let mut edit_label = use_signal(String::new);
    let mut confirm_remove = use_signal(|| None::<String>);
    
    let mut handle_rename_start = move |device_id: String, current_label: String| {
        editing_device.set(Some(device_id));
        edit_label.set(current_label);
    };
    
    let mut handle_rename_save = move |device_id: String| {
        let new_label = edit_label();
        if !new_label.trim().is_empty() {
            // Rename functionality can be added later
            editing_device.set(None);
        }
    };
    
    let mut handle_remove = move |device_id: String| {
        on_remove.call(device_id);
        confirm_remove.set(None);
    };
    
    let handle_close = move |_| {
        println!("[FileShare UI] MyDevices close button clicked");
        on_close.call(());
    };
    
    rsx! {
        div {
            style: "background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%); border-radius: 16px; padding: 24px; min-width: 450px;",
            
            // Header
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;",
                h2 { style: "color: #fff; margin: 0; font-size: 20px;", "📱 My Devices" }
                button {
                    style: "background: transparent; border: none; color: #888; cursor: pointer; font-size: 20px;",
                    onclick: handle_close,
                    "✕"
                }
            }
            
            // Device list
            div {
                style: "max-height: 400px; overflow-y: auto;",
                
                if devices.is_empty() {
                    div {
                        style: "text-align: center; color: #64748b; padding: 40px;",
                        div { style: "font-size: 48px; margin-bottom: 16px;", "🔗" }
                        div { "No trusted devices yet" }
                    }
                }
                
                for device in devices.iter() {
                    {
                        let device_id = device.id.clone();
                        let device_id2 = device.id.clone();
                        let device_id3 = device.id.clone();
                        let device_id4 = device.id.clone();
                        let device_label = device.label.clone();
                        let is_editing = editing_device() == Some(device.id.clone());
                        let is_confirming = confirm_remove() == Some(device.id.clone());
                        let border_color = if device.is_expired { "#f59e0b" } else { "#334155" };
                        
                        rsx! {
                            div {
                                key: "{device_id}",
                                style: "background: #1e293b; border-radius: 12px; padding: 16px; margin-bottom: 12px; border: 1px solid {border_color};",
                                
                                // Header
                                div {
                                    style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px;",
                                    
                                    div {
                                        style: "display: flex; align-items: center; gap: 12px;",
                                        span { style: "font-size: 32px;", "{device.os_icon}" }
                                        
                                        if is_editing {
                                            div {
                                                style: "display: flex; gap: 8px;",
                                                input {
                                                    style: "background: #0f172a; border: 1px solid #334155; color: #fff; padding: 6px 10px; border-radius: 6px;",
                                                    value: "{edit_label}",
                                                    oninput: move |e| edit_label.set(e.value()),
                                                }
                                                button {
                                                    style: "background: #22c55e; color: white; border: none; padding: 6px 10px; border-radius: 6px; cursor: pointer;",
                                                    onclick: move |_| handle_rename_save(device_id.clone()),
                                                    "✓"
                                                }
                                                button {
                                                    style: "background: #64748b; color: white; border: none; padding: 6px 10px; border-radius: 6px; cursor: pointer;",
                                                    onclick: move |_| editing_device.set(None),
                                                    "✕"
                                                }
                                            }
                                        } else {
                                            div {
                                                div { style: "color: #fff; font-weight: 500;", "{device.label}" }
                                                div { style: "color: #64748b; font-size: 13px;", "{device.os}" }
                                            }
                                        }
                                    }
                                    
                                    if device.is_expired {
                                        span { style: "background: #f59e0b; color: #000; padding: 4px 8px; border-radius: 4px; font-size: 11px;", "⚠ Expired" }
                                    }
                                }
                                
                                // Details
                                div {
                                    style: "display: grid; grid-template-columns: 1fr 1fr; gap: 8px; margin-bottom: 12px;",
                                    div { style: "color: #64748b; font-size: 12px;", "Trusted since" }
                                    div { style: "color: #94a3b8; font-size: 12px; text-align: right;", "{device.trusted_at}" }
                                    div { style: "color: #64748b; font-size: 12px;", "Last connected" }
                                    div { style: "color: #94a3b8; font-size: 12px; text-align: right;", "{device.last_connected}" }
                                }
                                
                                // Actions
                                if is_confirming {
                                    div {
                                        style: "background: #7f1d1d; padding: 12px; border-radius: 8px;",
                                        div { style: "color: #fca5a5; margin-bottom: 10px; font-size: 14px;", "Remove this device?" }
                                        div {
                                            style: "display: flex; gap: 8px;",
                                            button {
                                                style: "background: #ef4444; color: white; border: none; padding: 8px 16px; border-radius: 6px; cursor: pointer; flex: 1;",
                                                onclick: move |_| handle_remove(device_id2.clone()),
                                                "Remove"
                                            }
                                            button {
                                                style: "background: #334155; color: white; border: none; padding: 8px 16px; border-radius: 6px; cursor: pointer; flex: 1;",
                                                onclick: move |_| confirm_remove.set(None),
                                                "Cancel"
                                            }
                                        }
                                    }
                                } else if !is_editing {
                                    div {
                                        style: "display: flex; gap: 8px;",
                                        button {
                                            style: "background: #334155; color: #fff; border: none; padding: 8px 16px; border-radius: 6px; cursor: pointer; flex: 1;",
                                            onclick: move |_| handle_rename_start(device_id3.clone(), device_label.clone()),
                                            "✏️ Rename"
                                        }
                                        button {
                                            style: "background: #7f1d1d; color: #fca5a5; border: none; padding: 8px 16px; border-radius: 6px; cursor: pointer; flex: 1;",
                                            onclick: move |_| confirm_remove.set(Some(device_id4.clone())),
                                            "🗑️ Remove"
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
