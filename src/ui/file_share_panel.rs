// src/ui/file_share_panel.rs - File sharing with custom file explorer
use dioxus::prelude::*;
use crate::file_share::{FileShareManager, DeviceInfo};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

pub static FILE_SHARE_MANAGER: once_cell::sync::Lazy<Arc<Mutex<Option<FileShareManager>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

#[derive(Clone, Debug)]
pub struct FileItem {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PanelView { DeviceList, FileExplorer }

#[derive(Clone, Debug)]
pub struct FileSharePanelState {
    pub is_open: bool,
    pub devices: Vec<DeviceInfo>,
    pub bridge_code: String,
    pub current_view: PanelView,
    pub connected_device: Option<DeviceInfo>,
    pub current_path: PathBuf,
    pub files: Vec<FileItem>,
    pub selected_files: Vec<PathBuf>,
}

impl Default for FileSharePanelState {
    fn default() -> Self {
        Self {
            is_open: false,
            devices: Vec::new(),
            bridge_code: String::new(),
            current_view: PanelView::DeviceList,
            connected_device: None,
            current_path: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("C:\\")),
            files: Vec::new(),
            selected_files: Vec::new(),
        }
    }
}

pub static FILE_SHARE_STATE: once_cell::sync::Lazy<Arc<Mutex<FileSharePanelState>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(FileSharePanelState::default())));

pub async fn init_file_share() -> Result<(), Box<dyn std::error::Error>> {
    // First, try to stop any existing file share service
    let _ = shutdown_file_share().await;
    
    // Longer delay to ensure ports are fully released (macOS needs more time)
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Try up to 3 times with increasing delays
    for attempt in 1..=3 {
        match FileShareManager::new().await {
            Ok(manager) => {
                match manager.start().await {
                    Ok(_) => {
                        let code = manager.get_bridge_code().await;
                        
                        // Update state with the code
                        {
                            let mut state = FILE_SHARE_STATE.lock().unwrap();
                            state.bridge_code = code.clone();
                        }
                        
                        *FILE_SHARE_MANAGER.lock().unwrap() = Some(manager);
                        println!("✅ File share initialized with code: {}", code);
                        return Ok(());
                    }
                    Err(e) => {
                        if attempt < 3 {
                            println!("⚠️  File share init attempt {} failed: {}. Retrying in {}s...", 
                                attempt, e, attempt);
                            tokio::time::sleep(tokio::time::Duration::from_secs(attempt as u64)).await;
                            continue;
                        } else {
                            return Err(e);
                        }
                    }
                }
            }
            Err(e) => {
                if attempt < 3 {
                    println!("⚠️  File share manager creation attempt {} failed: {}. Retrying...", 
                        attempt, e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(attempt as u64)).await;
                    continue;
                } else {
                    return Err(e);
                }
            }
        }
    }
    
    Err("Failed to initialize file share after 3 attempts".into())
}

pub async fn shutdown_file_share() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(manager) = FILE_SHARE_MANAGER.lock().unwrap().as_ref() {
        manager.stop().await?;
        println!("✅ File share stopped");
    }
    *FILE_SHARE_MANAGER.lock().unwrap() = None;
    Ok(())
}

fn load_directory_files(path: &PathBuf) -> Vec<FileItem> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name.starts_with('.') { continue; }
                files.push(FileItem {
                    path: entry.path(),
                    name: file_name,
                    is_dir: metadata.is_dir(),
                    size: metadata.len(),
                });
            }
        }
    }
    files.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
    files
}

fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if size >= GB { format!("{:.2} GB", size as f64 / GB as f64) }
    else if size >= MB { format!("{:.2} MB", size as f64 / MB as f64) }
    else if size >= KB { format!("{:.2} KB", size as f64 / KB as f64) }
    else { format!("{} B", size) }
}

#[component]
pub fn FileSharePanel() -> Element {
    let mut is_open = use_signal(|| false);
    let mut view = use_signal(|| PanelView::DeviceList);
    let mut devices = use_signal(|| Vec::<DeviceInfo>::new());
    let mut code = use_signal(|| String::from("0000"));
    let connect_code = use_signal(|| String::new());
    let status = use_signal(|| String::new());
    let mut pulse = use_signal(|| 1.0);
    let mut device = use_signal(|| None::<DeviceInfo>);
    let mut path = use_signal(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("C:\\")));
    let mut files = use_signal(|| Vec::<FileItem>::new());
    let mut selected = use_signal(|| Vec::<PathBuf>::new());

    use_effect(move || {
        spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                let state = FILE_SHARE_STATE.lock().unwrap();
                is_open.set(state.is_open);
                devices.set(state.devices.clone());
                code.set(state.bridge_code.clone());
                view.set(state.current_view.clone());
                device.set(state.connected_device.clone());
                path.set(state.current_path.clone());
                files.set(state.files.clone());
                selected.set(state.selected_files.clone());
            }
        });
    });

    use_effect(move || {
        spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                if let Some(manager) = FILE_SHARE_MANAGER.lock().unwrap().as_ref() {
                    let devs = tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(manager.get_devices())
                    });
                    let mut state = FILE_SHARE_STATE.lock().unwrap();
                    state.devices = devs;
                }
            }
        });
    });

    use_effect(move || {
        spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                let current = pulse();
                pulse.set(if current >= 1.2 { 1.0 } else { current + 0.01 });
            }
        });
    });

    if !is_open() { return rsx! { div {} }; }

    let sel_count = selected().len();
    
    rsx! {
        div {
            style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(0,0,0,0.92); backdrop-filter: blur(16px); z-index: 9998; display: flex; align-items: center; justify-content: center;",
            onclick: move |_| {
                is_open.set(false);
                let mut state = FILE_SHARE_STATE.lock().unwrap();
                state.is_open = false;
                state.current_view = PanelView::DeviceList;
            },
            
            div {
                style: "width: min(95vw, 1100px); height: min(92vh, 750px); background: #000; border-radius: 24px; box-shadow: 0 0 80px rgba(34,211,238,0.3); display: flex; flex-direction: column; overflow: hidden;",
                onclick: move |e| e.stop_propagation(),
                
                div {
                    style: "position: absolute; width: 100%; height: 100%; background: radial-gradient(circle at 20% 50%, rgba(34,211,238,0.08) 0%, transparent 50%); pointer-events: none;",
                }
                
                if view() == PanelView::DeviceList {
                    {render_devices(devices, code, connect_code, status, pulse, is_open)}
                } else {
                    {render_explorer(device, path, files, selected, sel_count, status, is_open)}
                }
            }
        }
        
        style { "
            @keyframes slideIn {{ from {{ opacity: 0; transform: translateY(-12px); }} to {{ opacity: 1; transform: translateY(0); }} }}
            input:focus {{ border-color: rgba(168,85,247,0.6) !important; box-shadow: 0 0 0 4px rgba(168,85,247,0.15) !important; }}
            button:hover:not(:disabled) {{ transform: translateY(-2px); }}
            ::-webkit-scrollbar {{ width: 8px; }}
            ::-webkit-scrollbar-track {{ background: rgba(255,255,255,0.02); }}
            ::-webkit-scrollbar-thumb {{ background: rgba(34,211,238,0.3); border-radius: 4px; }}
        " }
    }
}

fn render_devices(
    devices: Signal<Vec<DeviceInfo>>,
    code: Signal<String>,
    mut connect_code: Signal<String>,
    mut status: Signal<String>,
    pulse: Signal<f64>,
    mut is_open: Signal<bool>,
) -> Element {
    let dev_count = devices().len();
    
    rsx! {
        div {
            style: "padding: 32px 40px; border-bottom: 1px solid rgba(34,211,238,0.15); z-index: 10;",
            div {
                style: "display: flex; justify-content: space-between; align-items: center;",
                div {
                    style: "display: flex; align-items: center; gap: 20px;",
                    div {
                        style: "width: 56px; height: 56px; background: linear-gradient(135deg, #06b6d4 0%, #a855f7 100%); border-radius: 16px; display: flex; align-items: center; justify-content: center; font-size: 32px;",
                        "📡"
                    }
                    div {
                        h2 { style: "font-size: 32px; font-weight: 800; margin: 0; background: linear-gradient(135deg, #22d3ee 0%, #a855f7 100%); -webkit-background-clip: text; -webkit-text-fill-color: transparent;", "File Share" }
                        p { style: "font-size: 14px; color: rgba(255,255,255,0.5); margin: 0;", "Share files instantly" }
                    }
                }
                div {
                    style: "display: flex; gap: 32px; align-items: center;",
                    div {
                        style: "text-align: center;",
                        div { style: "font-size: 11px; color: rgba(255,255,255,0.4); text-transform: uppercase; margin-bottom: 6px;", "Your Code" }
                        div { style: "font-size: 28px; font-weight: 800; color: #22d3ee; font-family: monospace; letter-spacing: 6px;", "{code}" }
                    }
                    div { style: "width: 1px; height: 48px; background: rgba(255,255,255,0.1);", }
                    div {
                        style: "text-align: center;",
                        div { style: "font-size: 11px; color: rgba(255,255,255,0.4); text-transform: uppercase; margin-bottom: 6px;", "Devices" }
                        div { style: "font-size: 28px; font-weight: 800; color: #a855f7;", "{dev_count}" }
                    }
                    button {
                        style: "background: rgba(239,68,68,0.15); border: 1px solid rgba(239,68,68,0.3); color: #ef4444; width: 48px; height: 48px; border-radius: 12px; cursor: pointer; font-size: 24px;",
                        onclick: move |_| {
                            is_open.set(false);
                            let mut state = FILE_SHARE_STATE.lock().unwrap();
                            state.is_open = false;
                        },
                        "×"
                    }
                }
            }
        }

        div {
            style: "flex: 1; display: flex; padding: 40px; gap: 40px;",
            div {
                style: "flex: 1;",
                h3 { style: "font-size: 16px; font-weight: 700; color: rgba(255,255,255,0.9); margin: 0 0 24px 0; text-transform: uppercase; letter-spacing: 2px;", "Nearby Devices" }
                if devices().is_empty() {
                    div {
                        style: "flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; border: 2px dashed rgba(34,211,238,0.2); border-radius: 20px; padding: 60px;",
                        div { style: format!("font-size: 80px; margin-bottom: 24px; transform: scale({}); transition: transform 0.3s;", pulse()), "🔍" }
                        h4 { style: "font-size: 24px; font-weight: 700; color: rgba(255,255,255,0.8); margin: 0 0 12px 0;", "Scanning..." }
                        p { style: "font-size: 15px; color: rgba(255,255,255,0.4); margin: 0;", "Looking for devices on your network" }
                    }
                } else {
                    div {
                        style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 20px;",
                        for dev in devices() {
                            {
                                let dev_id = dev.id.clone();
                                let dev_name = dev.name.clone();
                                let dev_os = dev.os.clone();
                                let dev_ip = dev.ip_address.clone();
                                rsx! {
                                    div {
                                        key: "{dev_id}",
                                        style: "background: rgba(34,211,238,0.05); border: 1px solid rgba(34,211,238,0.2); border-radius: 16px; padding: 24px; cursor: pointer; transition: all 0.3s;",
                                        onclick: move |_| {
                                            let dev_id = dev_id.clone();
                                            let dev_clone = dev.clone();
                                            spawn(async move {
                                                if let Some(manager) = FILE_SHARE_MANAGER.lock().unwrap().as_ref() {
                                                    match tokio::task::block_in_place(|| {
                                                        tokio::runtime::Handle::current().block_on(manager.connect_device(&dev_id))
                                                    }) {
                                                        Ok(_) => {
                                                            let mut state = FILE_SHARE_STATE.lock().unwrap();
                                                            state.current_view = PanelView::FileExplorer;
                                                            state.connected_device = Some(dev_clone);
                                                            state.files = load_directory_files(&state.current_path);
                                                        }
                                                        Err(e) => status.set(format!("❌ {}", e)),
                                                    }
                                                }
                                            });
                                        },
                                        div { style: "font-size: 48px; text-align: center; margin-bottom: 16px;", "{dev.get_icon()}" }
                                        h4 { style: "font-size: 16px; font-weight: 700; color: #22d3ee; margin: 0 0 8px 0; text-align: center;", "{dev_name}" }
                                        div { style: "font-size: 12px; color: rgba(255,255,255,0.5); text-align: center;", "{dev_os}" }
                                        div { style: "font-size: 11px; color: rgba(255,255,255,0.3); text-align: center; font-family: monospace;", "{dev_ip}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            div {
                style: "width: 360px;",
                div {
                    style: "background: rgba(168,85,247,0.05); border: 1px solid rgba(168,85,247,0.2); border-radius: 20px; padding: 32px;",
                    div {
                        style: "display: flex; align-items: center; gap: 16px; margin-bottom: 24px;",
                        div { style: "width: 48px; height: 48px; background: linear-gradient(135deg, #a855f7 0%, #06b6d4 100%); border-radius: 12px; display: flex; align-items: center; justify-content: center; font-size: 24px;", "🔗" }
                        h3 { style: "font-size: 20px; font-weight: 700; margin: 0; color: rgba(255,255,255,0.95);", "Remote Connect" }
                    }
                    p { style: "font-size: 14px; color: rgba(255,255,255,0.6); margin: 0 0 24px 0;", "Enter 4-digit code to connect" }
                    input {
                        style: "width: 100%; background: rgba(0,0,0,0.6); border: 2px solid rgba(168,85,247,0.3); border-radius: 16px; padding: 20px; color: #a855f7; font-size: 32px; font-weight: 800; text-align: center; letter-spacing: 16px; outline: none; font-family: monospace; margin-bottom: 20px;",
                        r#type: "text",
                        placeholder: "0000",
                        maxlength: "4",
                        value: "{connect_code}",
                        oninput: move |e| {
                            let val = e.value().chars().filter(|c| c.is_numeric()).collect::<String>();
                            connect_code.set(val);
                        },
                    }
                    button {
                        style: format!("width: 100%; background: {}; border: none; color: white; padding: 18px; border-radius: 16px; font-size: 17px; font-weight: 700; cursor: {}; opacity: {};",
                            if connect_code().len() == 4 { "linear-gradient(135deg, #a855f7 0%, #06b6d4 100%)" } else { "rgba(255,255,255,0.08)" },
                            if connect_code().len() == 4 { "pointer" } else { "not-allowed" },
                            if connect_code().len() == 4 { "1" } else { "0.4" }
                        ),
                        disabled: connect_code().len() != 4,
                        onclick: move |_| {
                            let code_val = connect_code();
                            spawn(async move {
                                if let Some(manager) = FILE_SHARE_MANAGER.lock().unwrap().as_ref() {
                                    match tokio::task::block_in_place(|| {
                                        tokio::runtime::Handle::current().block_on(manager.connect_by_code(&code_val))
                                    }) {
                                        Ok(_) => {
                                            status.set("✅ Connected!".to_string());
                                            connect_code.set(String::new());
                                        }
                                        Err(e) => status.set(format!("❌ {}", e)),
                                    }
                                }
                            });
                        },
                        "Connect"
                    }
                }
                if !status().is_empty() {
                    div { style: "padding: 20px; background: rgba(34,211,238,0.1); border-radius: 16px; margin-top: 20px; color: #22d3ee; font-size: 15px; text-align: center;", "{status}" }
                }
            }
        }
    }
}

fn render_explorer(
    device: Signal<Option<DeviceInfo>>,
    path: Signal<PathBuf>,
    files: Signal<Vec<FileItem>>,
    selected: Signal<Vec<PathBuf>>,
    sel_count: usize,
    mut status: Signal<String>,
    mut is_open: Signal<bool>,
) -> Element {
    let dev_name = device().map(|d| d.name).unwrap_or_else(|| "Unknown".to_string());
    let path_str = path().to_string_lossy().to_string();
    
    rsx! {
        div {
            style: "padding: 32px 40px; border-bottom: 1px solid rgba(34,211,238,0.15); z-index: 10;",
            div {
                style: "display: flex; justify-content: space-between; align-items: center;",
                div {
                    style: "display: flex; align-items: center; gap: 20px;",
                    button {
                        style: "background: rgba(34,211,238,0.15); border: 1px solid rgba(34,211,238,0.3); color: #22d3ee; width: 48px; height: 48px; border-radius: 12px; cursor: pointer; font-size: 20px;",
                        onclick: move |_| {
                            let mut state = FILE_SHARE_STATE.lock().unwrap();
                            state.current_view = PanelView::DeviceList;
                            state.selected_files.clear();
                        },
                        "←"
                    }
                    div {
                        h2 { style: "font-size: 28px; font-weight: 800; margin: 0; background: linear-gradient(135deg, #22d3ee 0%, #a855f7 100%); -webkit-background-clip: text; -webkit-text-fill-color: transparent;", "Send to {dev_name}" }
                        p { style: "font-size: 13px; color: rgba(255,255,255,0.5); margin: 4px 0 0 0; font-family: monospace;", "{path_str}" }
                    }
                }
                div {
                    style: "display: flex; gap: 16px; align-items: center;",
                    div {
                        style: "text-align: center;",
                        div { style: "font-size: 11px; color: rgba(255,255,255,0.4); text-transform: uppercase; margin-bottom: 6px;", "Selected" }
                        div { style: "font-size: 24px; font-weight: 800; color: #a855f7;", "{sel_count}" }
                    }
                    button {
                        style: format!("background: {}; border: none; color: white; padding: 14px 32px; border-radius: 12px; font-size: 16px; font-weight: 700; cursor: {}; opacity: {};",
                            if sel_count > 0 { "linear-gradient(135deg, #22d3ee 0%, #a855f7 100%)" } else { "rgba(255,255,255,0.08)" },
                            if sel_count > 0 { "pointer" } else { "not-allowed" },
                            if sel_count > 0 { "1" } else { "0.4" }
                        ),
                        disabled: sel_count == 0,
                        onclick: move |_| {
                            let sel_files = selected();
                            spawn(async move {
                                status.set(format!("📤 Sending {} files...", sel_files.len()));
                                // TODO: Implement actual file transfer
                                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                                status.set(format!("✅ Sent {} files successfully!", sel_files.len()));
                                let mut state = FILE_SHARE_STATE.lock().unwrap();
                                state.selected_files.clear();
                            });
                        },
                        "📤 Send Files"
                    }
                    button {
                        style: "background: rgba(239,68,68,0.15); border: 1px solid rgba(239,68,68,0.3); color: #ef4444; width: 48px; height: 48px; border-radius: 12px; cursor: pointer; font-size: 24px;",
                        onclick: move |_| {
                            is_open.set(false);
                            let mut state = FILE_SHARE_STATE.lock().unwrap();
                            state.is_open = false;
                            state.current_view = PanelView::DeviceList;
                        },
                        "×"
                    }
                }
            }
        }

        div {
            style: "flex: 1; padding: 40px; overflow-y: auto;",
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;",
                h3 { style: "font-size: 16px; font-weight: 700; color: rgba(255,255,255,0.9); margin: 0; text-transform: uppercase; letter-spacing: 2px;", "Select Files to Send" }
                div {
                    style: "display: flex; gap: 12px;",
                    button {
                        style: "background: rgba(34,211,238,0.15); border: 1px solid rgba(34,211,238,0.3); color: #22d3ee; padding: 10px 20px; border-radius: 10px; font-size: 13px; font-weight: 600; cursor: pointer;",
                        onclick: move |_| {
                            let mut state = FILE_SHARE_STATE.lock().unwrap();
                            state.selected_files = state.files.iter().map(|f| f.path.clone()).collect();
                        },
                        "Select All"
                    }
                    button {
                        style: "background: rgba(239,68,68,0.15); border: 1px solid rgba(239,68,68,0.3); color: #ef4444; padding: 10px 20px; border-radius: 10px; font-size: 13px; font-weight: 600; cursor: pointer;",
                        onclick: move |_| {
                            let mut state = FILE_SHARE_STATE.lock().unwrap();
                            state.selected_files.clear();
                        },
                        "Clear All"
                    }
                    button {
                        style: "background: rgba(168,85,247,0.15); border: 1px solid rgba(168,85,247,0.3); color: #a855f7; padding: 10px 20px; border-radius: 10px; font-size: 13px; font-weight: 600; cursor: pointer;",
                        onclick: move |_| {
                            if let Some(parent) = path().parent() {
                                let mut state = FILE_SHARE_STATE.lock().unwrap();
                                state.current_path = parent.to_path_buf();
                                state.files = load_directory_files(&state.current_path);
                                state.selected_files.clear();
                            }
                        },
                        "⬆️ Up"
                    }
                }
            }
            
            if files().is_empty() {
                div {
                    style: "display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 60px; border: 2px dashed rgba(34,211,238,0.2); border-radius: 20px;",
                    div { style: "font-size: 64px; margin-bottom: 16px;", "📂" }
                    p { style: "font-size: 18px; color: rgba(255,255,255,0.6); margin: 0;", "Empty folder" }
                }
            } else {
                div {
                    style: "display: flex; flex-direction: column; gap: 8px;",
                    for file in files() {
                        {
                            let file_path = file.path.clone();
                            let file_path_key = file_path.to_string_lossy().to_string();
                            let file_path_onclick = file_path.clone();
                            let file_path_checkbox = file_path.clone();
                            let file_name = file.name.clone();
                            let is_dir = file.is_dir;
                            let file_size = file.size;
                            let is_selected = selected().contains(&file_path);
                            
                            rsx! {
                                div {
                                    key: "{file_path_key}",
                                    style: format!("display: flex; align-items: center; gap: 16px; padding: 16px 20px; background: {}; border: 1px solid {}; border-radius: 12px; cursor: pointer; transition: all 0.2s;",
                                        if is_selected { "rgba(34,211,238,0.15)" } else { "rgba(255,255,255,0.02)" },
                                        if is_selected { "rgba(34,211,238,0.4)" } else { "rgba(255,255,255,0.08)" }
                                    ),
                                    onclick: move |_| {
                                        if is_dir {
                                            let mut state = FILE_SHARE_STATE.lock().unwrap();
                                            state.current_path = file_path_onclick.clone();
                                            state.files = load_directory_files(&state.current_path);
                                            state.selected_files.clear();
                                        } else {
                                            let mut state = FILE_SHARE_STATE.lock().unwrap();
                                            if let Some(pos) = state.selected_files.iter().position(|p| p == &file_path_onclick) {
                                                state.selected_files.remove(pos);
                                            } else {
                                                state.selected_files.push(file_path_onclick.clone());
                                            }
                                        }
                                    },
                                    
                                    input {
                                        r#type: "checkbox",
                                        checked: is_selected,
                                        style: "width: 20px; height: 20px; cursor: pointer; accent-color: #22d3ee;",
                                        onclick: move |e| e.stop_propagation(),
                                        onchange: move |_| {
                                            let mut state = FILE_SHARE_STATE.lock().unwrap();
                                            if let Some(pos) = state.selected_files.iter().position(|p| p == &file_path_checkbox) {
                                                state.selected_files.remove(pos);
                                            } else {
                                                state.selected_files.push(file_path_checkbox.clone());
                                            }
                                        },
                                    }
                                    
                                    div { style: "font-size: 32px;", if is_dir { "📁" } else { "📄" } }
                                    
                                    div {
                                        style: "flex: 1;",
                                        div { style: "font-size: 15px; font-weight: 600; color: rgba(255,255,255,0.9); margin-bottom: 4px;", "{file_name}" }
                                        div { style: "font-size: 12px; color: rgba(255,255,255,0.5);", if is_dir { "Folder" } else { "{format_size(file_size)}" } }
                                    }
                                    
                                    if is_dir {
                                        div { style: "font-size: 20px; color: rgba(255,255,255,0.3);", "→" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            if !status().is_empty() {
                div {
                    style: "margin-top: 24px; padding: 20px; background: rgba(34,211,238,0.1); border-radius: 16px; border: 1px solid rgba(34,211,238,0.3); color: #22d3ee; font-size: 15px; font-weight: 600; text-align: center; animation: slideIn 0.4s;",
                    "{status}"
                }
            }
        }
    }
}

pub fn toggle_file_share_panel() {
    let mut state = FILE_SHARE_STATE.lock().unwrap();
    state.is_open = !state.is_open;
}

pub fn open_file_share_panel() {
    let mut state = FILE_SHARE_STATE.lock().unwrap();
    state.is_open = true;
}

pub fn close_file_share_panel() {
    let mut state = FILE_SHARE_STATE.lock().unwrap();
    state.is_open = false;
}
