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
    let mut my_code = use_signal(|| String::new());
    let mut remaining_seconds = use_signal(|| 0u64);
    let mut input_code = use_signal(|| String::new());
    let mut connection_status = use_signal(|| ConnectionStatus::Idle);
    let mut connecting_device_id = use_signal(|| None::<String>);
    
    // Generate my device code on mount and refresh periodically
    use_effect(move || {
        spawn(async move {
            loop {
                match generate_device_code().await {
                    Ok((code, remaining)) => {
                        println!("[Radar] Generated/refreshed code: {} ({}s remaining)", code, remaining);
                        my_code.set(code);
                        remaining_seconds.set(remaining);
                    },
                    Err(e) => println!("[Radar] Error generating code: {}", e),
                }
                
                // Wait 5 seconds before checking again
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        });
    });
    
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
    
    // Handle manual code connection
    let handle_code_connect = move |_| {
        let code = input_code();
        if code.len() != 4 {
            connection_status.set(ConnectionStatus::Error("Code must be 4 digits".to_string()));
            return;
        }
        
        connection_status.set(ConnectionStatus::Connecting("device".to_string()));
        connecting_device_id.set(None);
        
        spawn(async move {
            match connect_via_code(&code).await {
                Ok(result) => {
                    println!("[Radar] Connected via code to: {}", result.device_label);
                    
                    // Show success message
                    connection_status.set(ConnectionStatus::Connected(result.device_label.clone()));
                    
                    // Clear input
                    input_code.set(String::new());
                    
                    // Clear success message after 3 seconds
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                    connection_status.set(ConnectionStatus::Idle);
                },
                Err(e) => {
                    println!("[Radar] Code connection error: {}", e);
                    connection_status.set(ConnectionStatus::Error(e));
                }
            }
        });
    };
    
    rsx! {
        div {
            class: "device-radar-container",
            style: "background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%); border-radius: 16px; padding: 24px; min-width: 450px; max-width: min(800px, 90vw); width: min(600px, 85vw); max-height: 90vh; overflow-y: auto; overflow-x: hidden;",
            
            // Header with close button
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;",
                
                h2 {
                    style: "color: #fff; margin: 0; font-size: 20px;",
                    "📡 File Share"
                }
                
                button {
                    style: "background: transparent; border: none; color: #888; cursor: pointer; font-size: 20px;",
                    onclick: handle_close,
                    "✕"
                }
            }
            
            // My Device Code Section
            div {
                style: "background: #0f172a; border: 2px solid #3b82f6; border-radius: 12px; padding: 16px; margin-bottom: 20px; text-align: center;",
                
                div { style: "color: #94a3b8; font-size: 14px; margin-bottom: 8px;", "Your Device Code" }
                
                if my_code().is_empty() {
                    div { style: "color: #64748b; font-size: 14px;", "Generating code..." }
                } else {
                    div {
                        style: "display: flex; justify-content: center; gap: 8px; margin-bottom: 8px;",
                        for digit in my_code().chars() {
                            div {
                                style: "width: 50px; height: 60px; background: #1e293b; border: 2px solid #3b82f6; border-radius: 8px; display: flex; align-items: center; justify-content: center; font-size: 28px; font-weight: bold; color: #3b82f6;",
                                "{digit}"
                            }
                        }
                    }
                    
                    // Remaining time display with warning indicator
                    {
                        let remaining = remaining_seconds();
                        let minutes = remaining / 60;
                        let seconds = remaining % 60;
                        let is_warning = remaining < 60;
                        let color = if is_warning { "#f59e0b" } else { "#64748b" };
                        let icon = if is_warning { "⚠️ " } else { "" };
                        
                        rsx! {
                            div {
                                style: "color: {color}; font-size: 12px; margin-bottom: 4px;",
                                "{icon}Expires in {minutes}:{seconds:02}"
                            }
                        }
                    }
                    
                    div { style: "color: #64748b; font-size: 12px;", "Share this code to receive files" }
                }
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
                style: "position: relative; width: min(300px, 100%); height: min(300px, 100%); aspect-ratio: 1; margin: 0 auto 20px; background: radial-gradient(circle, #0f172a 0%, #1e293b 100%); border-radius: 50%; border: 2px solid #334155;",
                
                // Radar rings
                div { style: "position: absolute; top: 25%; left: 25%; width: 50%; height: 50%; border: 1px solid #334155; border-radius: 50%;" }
                div { style: "position: absolute; top: 10%; left: 10%; width: 80%; height: 80%; border: 1px solid #334155; border-radius: 50%;" }
                
                // Center point
                div { style: "position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); width: 12px; height: 12px; background: #3b82f6; border-radius: 50%; box-shadow: 0 0 10px #3b82f6;" }
                
                // Device dots
                for (i, device) in devices.iter().enumerate() {
                    {
                        let angle = (i as f64) * (360.0 / devices.len().max(1) as f64);
                        let radius = if device.is_trusted { 60.0 } else { 100.0 };
                        let x = 150.0 + radius * (angle * std::f64::consts::PI / 180.0).cos();
                        let y = 150.0 + radius * (angle * std::f64::consts::PI / 180.0).sin();
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
                style: "margin-bottom: 20px;",
                
                div { style: "color: #94a3b8; font-size: 14px; margin-bottom: 12px; font-weight: 500;", "Discovered Devices" }
                
                div {
                    style: "max-height: 200px; overflow-y: auto;",
                    
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
                            let device_code = device.code.clone();
                            let is_selected = selected_device() == Some(device.id.clone());
                            let bg = if is_selected { "#334155" } else { "#1e293b" };
                            
                            rsx! {
                                div {
                                    key: "{device.id}",
                                    style: "display: flex; align-items: center; justify-content: space-between; padding: 12px; background: {bg}; border-radius: 8px; margin-bottom: 8px; cursor: pointer;",
                                    onclick: move |_| handle_device_click(device_id.clone()),
                                    
                                    div {
                                        style: "display: flex; align-items: center; gap: 12px;",
                                        span { style: "font-size: 24px;", "{device.os_icon}" }
                                        div {
                                            div { style: "color: #fff; font-weight: 500;", "{device.label}" }
                                            div { 
                                                style: "color: #64748b; font-size: 12px;", 
                                                "{device.os} • {device.ip}"
                                                if let Some(code) = &device_code {
                                                    " • Code: {code}"
                                                }
                                            }
                                        }
                                    }
                                    
                                    div {
                                        style: "display: flex; align-items: center; gap: 8px;",
                                        if device.is_trusted {
                                            span { style: "color: #4ade80; font-size: 12px;", "✓ Connected" }
                                        } else if let Some(ref code) = device_code {
                                            // Show code with "Use Code" button
                                            div {
                                                style: "display: flex; flex-direction: column; align-items: flex-end; gap: 4px;",
                                                div {
                                                    style: "background: #3b82f6; color: white; padding: 4px 12px; border-radius: 6px; font-size: 14px; font-weight: bold; letter-spacing: 2px;",
                                                    "{code}"
                                                }
                                                button {
                                                    style: "background: #10b981; color: white; border: none; padding: 4px 12px; border-radius: 6px; cursor: pointer; font-size: 11px;",
                                                    onclick: {
                                                        let code_val = code.clone();
                                                        move |e| {
                                                            e.stop_propagation();
                                                            input_code.set(code_val.clone());
                                                        }
                                                    },
                                                    "Use Code"
                                                }
                                            }
                                        } else {
                                            // No code available, show connect button
                                            button {
                                                style: "background: #3b82f6; color: white; border: none; padding: 6px 12px; border-radius: 6px; cursor: pointer; font-size: 12px;",
                                                onclick: move |e| {
                                                    e.stop_propagation();
                                                    handle_connect(device_id2.clone());
                                                },
                                                "Connect"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Manual Code Connection Section
            div {
                style: "background: #0f172a; border: 1px solid #334155; border-radius: 12px; padding: 16px;",
                
                div { style: "color: #94a3b8; font-size: 14px; margin-bottom: 12px; font-weight: 500;", "Connect to Device" }
                
                div {
                    style: "display: flex; gap: 8px; align-items: flex-start;",
                    
                    div {
                        style: "flex: 1;",
                        input {
                            r#type: "text",
                            placeholder: "Enter 4-digit code",
                            maxlength: 4,
                            value: "{input_code}",
                            style: "width: 100%; padding: 10px; background: #1e293b; border: 2px solid #334155; border-radius: 8px; color: #fff; font-size: 16px; text-align: center; letter-spacing: 8px; font-weight: bold;",
                            oninput: move |e| {
                                let val = e.value();
                                // Only allow numbers, max 4 digits
                                if val.chars().all(|c| c.is_numeric()) && val.len() <= 4 {
                                    input_code.set(val);
                                    // Clear error when user types
                                    if matches!(connection_status(), ConnectionStatus::Error(_)) {
                                        connection_status.set(ConnectionStatus::Idle);
                                    }
                                }
                            }
                        }
                    }
                    
                    button {
                        style: "background: #3b82f6; color: white; border: none; padding: 10px 20px; border-radius: 8px; cursor: pointer; font-size: 14px; font-weight: 500;",
                        disabled: input_code().len() != 4 || matches!(connection_status(), ConnectionStatus::Connecting(_)),
                        onclick: handle_code_connect,
                        if matches!(connection_status(), ConnectionStatus::Connecting(_)) {
                            "Connecting..."
                        } else {
                            "Connect"
                        }
                    }
                }
                
                // Connection status messages
                match connection_status() {
                    ConnectionStatus::Connecting(label) => rsx! {
                        div {
                            style: "margin-top: 8px; color: #3b82f6; font-size: 12px;",
                            "🔄 Connecting to {label}..."
                        }
                    },
                    ConnectionStatus::Connected(label) => rsx! {
                        div {
                            style: "margin-top: 8px; color: #4ade80; font-size: 12px;",
                            "✓ Connected to {label}"
                        }
                    },
                    ConnectionStatus::Error(err) => rsx! {
                        div {
                            style: "margin-top: 8px; color: #ef4444; font-size: 12px;",
                            "⚠️ {err}"
                        }
                    },
                    ConnectionStatus::Idle => rsx! {
                        div {
                            style: "margin-top: 8px; color: #64748b; font-size: 11px;",
                            "Enter the 4-digit code from another device to connect"
                        }
                    },
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

/// Result of a code connection
struct CodeConnectionResult {
    device_label: String,
}

/// Generate a 4-digit code for this device using ConnectionCoordinator
async fn generate_device_code() -> Result<(String, u64), String> {
    // Get the connection coordinator
    let coordinator = get_connection_coordinator()
        .map_err(|e| format!("Failed to get coordinator: {}", e))?;
    
    // Generate code
    let connection_code = coordinator.generate_my_code()
        .map_err(|e| format!("Failed to generate code: {}", e))?;
    
    Ok((connection_code.code, connection_code.remaining_seconds))
}

/// Connect to a device using their 4-digit code via ConnectionCoordinator
async fn connect_via_code(code: &str) -> Result<CodeConnectionResult, String> {
    // Get the connection coordinator
    let coordinator = get_connection_coordinator()
        .map_err(|e| format!("Failed to get coordinator: {}", e))?;
    
    // Connect using the code
    let result = coordinator.connect_with_code(code).await
        .map_err(|e| e.user_message())?;
    
    println!("[Radar] Connected via code to: {} (type: {:?})", 
        result.device.label, result.connection_type);
    
    Ok(CodeConnectionResult {
        device_label: result.device.label,
    })
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
    
    // Create new coordinator
    let relay = Arc::new(crate::file_share::relay::RelayService::new());
    
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
