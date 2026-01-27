// src/ui/file_share/device_radar.rs - Device Radar UI Component

use dioxus::prelude::*;
use std::sync::Arc;

/// Device info for display (self-contained)
#[derive(Clone, PartialEq, Debug)]
pub struct DeviceDisplay {
    pub id: String,
    pub label: String,
    pub os: String,
    pub os_icon: String,
    pub ip: String,
    pub is_trusted: bool,
    pub is_online: bool,
    pub code: Option<String>, // Pairing code from discovery
}

/// Connection status for UI feedback
#[derive(Clone, PartialEq, Debug)]
enum ConnectionStatus {
    Idle,
    Connecting(String), // device label
    Connected(String),  // device label
    Error(String),      // error message
}

/// Device Radar - Shows nearby IGRIS devices
#[component]
pub fn DeviceRadar(
    devices: Vec<DeviceDisplay>,
    is_scanning: bool,
    on_device_select: EventHandler<String>,
    on_connect: EventHandler<String>,
    on_close: EventHandler<()>,
) -> Element {
    let mut selected_device = use_signal(|| None::<String>);
    
    // No code generation needed - direct connections only
    
    let mut handle_device_click = move |device_id: String| {
        selected_device.set(Some(device_id.clone()));
        on_device_select.call(device_id);
    };
    
    let handle_connect = move |device_id: String| {
        on_connect.call(device_id);
    };
    
    let handle_close = move |_| {
        println!("[FileShare UI] Close button clicked");
        on_close.call(());
    };
    
    rsx! {
        div {
            class: "device-radar-container",
            style: "background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%); border-radius: 12px; padding: 16px; min-width: 360px; max-width: 500px; width: 400px; max-height: 80vh; overflow-y: auto; overflow-x: hidden; resize: both;",
            
            // Header with close button
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;",
                
                h2 {
                    style: "color: #fff; margin: 0; font-size: 18px;",
                    "📡 File Share"
                }
                
                button {
                    style: "background: transparent; border: none; color: #888; cursor: pointer; font-size: 18px; padding: 4px;",
                    onclick: handle_close,
                    "✕"
                }
            }
            
            // My Device Info Section (no code needed)
            div {
                style: "background: #0f172a; border: 2px solid #3b82f6; border-radius: 10px; padding: 12px; margin-bottom: 16px; text-align: center;",
                
                div { style: "color: #94a3b8; font-size: 12px; margin-bottom: 6px;", "Your Device" }
                div { style: "color: #3b82f6; font-size: 16px; font-weight: bold; margin-bottom: 4px;", "Ready for Direct Connections" }
                div { style: "color: #64748b; font-size: 12px;", "Other devices can connect to you directly" }
            }
            
            // Scanning status
            if is_scanning {
                div {
                    style: "text-align: center; margin-bottom: 16px;",
                    span {
                        style: "color: #4ade80; font-size: 14px; display: inline-flex; align-items: center; gap: 6px;",
                        span { style: "animation: pulse 1.5s infinite;", "●" }
                        "Scanning for devices..."
                    }
                }
            }
            
            // Radar visualization
            div {
                style: "position: relative; width: 200px; height: 200px; margin: 0 auto 14px; background: radial-gradient(circle, #0f172a 0%, #1e293b 100%); border-radius: 50%; border: 2px solid #334155; flex-shrink: 0;",
                
                // Radar rings
                div { style: "position: absolute; top: 25%; left: 25%; width: 50%; height: 50%; border: 1px solid #334155; border-radius: 50%;" }
                div { style: "position: absolute; top: 10%; left: 10%; width: 80%; height: 80%; border: 1px solid #334155; border-radius: 50%;" }
                
                // Center point
                div { style: "position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); width: 12px; height: 12px; background: #3b82f6; border-radius: 50%; box-shadow: 0 0 10px #3b82f6;" }
                
                // Device dots
                for (i, device) in devices.iter().enumerate() {
                    {
                        let angle = (i as f64) * (360.0 / devices.len().max(1) as f64);
                        let radius = if device.is_trusted { 40.0 } else { 70.0 };
                        let x = 100.0 + radius * (angle * std::f64::consts::PI / 180.0).cos();
                        let y = 100.0 + radius * (angle * std::f64::consts::PI / 180.0).sin();
                        let color = if device.is_trusted { "#4ade80" } else { "#f59e0b" };
                        let device_id = device.id.clone();
                        
                        rsx! {
                            div {
                                key: "{device.id}",
                                style: "position: absolute; left: {x}px; top: {y}px; transform: translate(-50%, -50%); cursor: pointer;",
                                onclick: move |_| handle_device_click(device_id.clone()),
                                
                                div { style: "width: 16px; height: 16px; background: {color}; border-radius: 50%; box-shadow: 0 0 8px {color};" }
                                div {
                                    style: "position: absolute; top: 20px; left: 50%; transform: translateX(-50%); white-space: nowrap; color: #fff; font-size: 11px;",
                                    "{device.os_icon}"
                                }
                            }
                        }
                    }
                }
                
                // Scanning sweep
                if is_scanning {
                    div { style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; background: conic-gradient(from 0deg, transparent 0deg, rgba(59, 130, 246, 0.3) 30deg, transparent 60deg); border-radius: 50%; animation: radar-sweep 3s linear infinite;" }
                }
            }
            
            // Device list
            div {
                style: "margin-bottom: 14px; flex: 1; min-height: 0;",
                
                div { style: "color: #94a3b8; font-size: 13px; margin-bottom: 10px; font-weight: 500;", "Discovered Devices" }
                
                div {
                    style: "max-height: 160px; overflow-y: auto;",
                    
                    if devices.is_empty() {
                        div {
                            style: "text-align: center; color: #64748b; padding: 20px; background: #1e293b; border-radius: 8px;",
                            "No devices found yet..."
                        }
                    }
                    
                    for device in devices.iter() {
                        {
                            let device_id = device.id.clone();
                            let device_id2 = device.id.clone();
                            let is_selected = selected_device() == Some(device.id.clone());
                            let bg = if is_selected { "#334155" } else { "#1e293b" };
                            
                            rsx! {
                                div {
                                    key: "{device.id}",
                                    style: "display: flex; align-items: center; justify-content: space-between; padding: 8px 10px; background: {bg}; border-radius: 6px; margin-bottom: 6px; cursor: pointer;",
                                    onclick: move |_| handle_device_click(device_id.clone()),
                                    
                                    div {
                                        style: "display: flex; align-items: center; gap: 8px;",
                                        span { style: "font-size: 20px;", "{device.os_icon}" }
                                        div {
                                            div { style: "color: #fff; font-weight: 500; font-size: 13px;", "{device.label}" }
                                            div { 
                                                style: "color: #64748b; font-size: 11px;", 
                                                "{device.os} • {device.ip}"
                                            }
                                        }
                                    }
                                    
                                    div {
                                        style: "display: flex; align-items: center; gap: 8px;",
                                        // Check actual QUIC connection status, not just trust
                                        {
                                            let is_quic_connected = crate::file_share::is_connected_to_quic(&device.id).unwrap_or(false);
                                            if is_quic_connected {
                                                rsx! {
                                                    span { style: "color: #4ade80; font-size: 12px;", "✓ Connected (QUIC)" }
                                                }
                                            } else if device.is_trusted {
                                                rsx! {
                                                    button {
                                                        style: "background: #3b82f6; color: white; border: none; padding: 6px 16px; border-radius: 6px; cursor: pointer; font-size: 12px; font-weight: 500;",
                                                        onclick: move |e| {
                                                            e.stop_propagation();
                                                            handle_connect(device_id2.clone());
                                                        },
                                                        "🔗 Reconnect"
                                                    }
                                                }
                                            } else {
                                                rsx! {
                                                    button {
                                                        style: "background: #10b981; color: white; border: none; padding: 6px 16px; border-radius: 6px; cursor: pointer; font-size: 12px; font-weight: 500;",
                                                        onclick: move |e| {
                                                            e.stop_propagation();
                                                            handle_connect(device_id2.clone());
                                                        },
                                                        "🔗 Connect"
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
        }
        
        style { r#"
            @keyframes pulse {{ 0%, 100% {{ opacity: 1; }} 50% {{ opacity: 0.5; }} }}
            @keyframes radar-sweep {{ from {{ transform: rotate(0deg); }} to {{ transform: rotate(360deg); }} }}
        "# }
    }
}

// ═══════════════════════════════════════════════════════
// ASYNC HELPERS
// ═══════════════════════════════════════════════════════

/// Result of a direct connection
struct CodeConnectionResult {
    device_label: String,
}

/// Get or create the ConnectionCoordinator instance
fn get_connection_coordinator() -> Result<Arc<crate::file_share::connection::ConnectionCoordinator>, String> {
    use std::sync::Mutex;
    use lazy_static::lazy_static;
    
    lazy_static! {
        static ref COORDINATOR: Mutex<Option<Arc<crate::file_share::connection::ConnectionCoordinator>>> = Mutex::new(None);
    }
    
    let mut coord_lock = COORDINATOR.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    if let Some(ref coordinator) = *coord_lock {
        return Ok(Arc::clone(coordinator));
    }
    
    // Use the GLOBAL relay service instead of creating a new one
    let relay = crate::file_share::relay::get_relay_service();
    
    let trust = Arc::new(Mutex::new(crate::file_share::trust::TrustManager::new()));
    
    // Discovery service is optional - will be initialized by the main file share system
    let discovery = Arc::new(Mutex::new(None));
    
    let coordinator = Arc::new(crate::file_share::connection::ConnectionCoordinator::new(
        relay,
        trust,
        discovery,
    ));
    
    *coord_lock = Some(Arc::clone(&coordinator));
    
    Ok(coordinator)
}
