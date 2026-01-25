# File Share UI Specification - Complete Design 🎨

## Overview

Simple, intuitive file sharing UI with 4-digit codes for cross-platform connections.

## UI Flow

```
Radar Panel → Connect → File Explorer → Confirmation → Transfer
```

---

## 1. Radar Panel (Main Screen)

### Layout:

```
┌─────────────────────────────────────────┐
│  📡 File Share                     [X]  │
├─────────────────────────────────────────┤
│                                         │
│  Your Device: Rohit's Mac               │
│  Code: 1234                             │
│  (Share this code to receive files)     │
│                                         │
├─────────────────────────────────────────┤
│                                         │
│  🔍 Scanning for devices...             │
│                                         │
│     ●  ●  ●  ●  ●                       │
│   ●           ●                         │
│     ●  [ME]  ●    (Radar animation)    │
│   ●           ●                         │
│     ●  ●  ●  ●  ●                       │
│                                         │
│  Found Devices:                         │
│  • Windows PC (192.168.1.20) [Connect] │
│  • iPhone (192.168.1.30)     [Connect] │
│                                         │
├─────────────────────────────────────────┤
│                                         │
│  Connect to Device:                     │
│  ┌─────────────────┐  ┌──────────────┐ │
│  │ Enter 4-digit   │  │   Connect    │ │
│  │ code: [____]    │  │              │ │
│  └─────────────────┘  └──────────────┘ │
│                                         │
└─────────────────────────────────────────┘
```

### Components:

1. **Header**
   - Device name: "Rohit's Mac"
   - 4-digit code: "1234" (large, bold)
   - Subtitle: "Share this code to receive files"

2. **Radar Animation**
   - Circular scanning animation
   - Shows "ME" in center
   - Discovered devices appear as dots

3. **Discovered Devices List**
   - Device name + IP
   - "Connect" button for each

4. **Manual Connect Section**
   - 4-digit code input (numeric only, max 4 digits)
   - "Connect" button

---

## 2. File Explorer UI (After Connection)

### Layout:

```
┌─────────────────────────────────────────┐
│  📁 Select File to Share           [X]  │
├─────────────────────────────────────────┤
│  Connected to: Windows PC               │
│                                         │
│  ┌───────────────────────────────────┐ │
│  │ 🔍 Search files...                │ │
│  └───────────────────────────────────┘ │
│                                         │
│  📍 Current: /Users/rohit/Documents     │
│  ┌─ ← Back                             │
│                                         │
│  📂 Folders:                            │
│  ├─ 📁 Projects                         │
│  ├─ 📁 Photos                           │
│  ├─ 📁 Videos                           │
│  └─ 📁 Downloads                        │
│                                         │
│  📄 Files:                              │
│  ├─ 📄 Report.pdf          (2.5 MB)    │
│  ├─ 🖼️  Photo.jpg          (1.2 MB)    │
│  ├─ 🎵 Song.mp3            (4.8 MB)    │
│  └─ 📹 Video.mp4           (45 MB)     │
│                                         │
│  Quick Access:                          │
│  💾 Desktop  📁 Documents  ⬇️ Downloads │
│                                         │
└─────────────────────────────────────────┘
```

### Features:

1. **Search Bar**
   - Real-time file search
   - Searches in current folder and subfolders

2. **Breadcrumb Navigation**
   - Shows current path
   - Click to navigate up

3. **Folder List**
   - Click to enter folder
   - Shows folder icon

4. **File List**
   - File name + size
   - File type icon
   - Click to select

5. **Quick Access**
   - Desktop, Documents, Downloads
   - One-click navigation

---

## 3. Confirmation Dialog

### Layout:

```
┌─────────────────────────────────────────┐
│  ⚠️  Confirm File Share                 │
├─────────────────────────────────────────┤
│                                         │
│  Do you want to share this file?        │
│                                         │
│  📄 Report.pdf                          │
│  Size: 2.5 MB                           │
│  To: Windows PC (192.168.1.20)          │
│                                         │
│  ┌──────────────┐  ┌──────────────┐    │
│  │   Cancel     │  │  Share File  │    │
│  └──────────────┘  └──────────────┘    │
│                                         │
└─────────────────────────────────────────┘
```

---

## 4. Transfer Progress

### Layout:

```
┌─────────────────────────────────────────┐
│  📤 Sending File...                     │
├─────────────────────────────────────────┤
│                                         │
│  Report.pdf → Windows PC                │
│                                         │
│  ████████████░░░░░░░░░░░░  45%         │
│                                         │
│  1.1 MB / 2.5 MB                        │
│  Speed: 2.3 MB/s                        │
│  Time remaining: 3 seconds              │
│                                         │
│  ┌──────────────┐                       │
│  │   Cancel     │                       │
│  └──────────────┘                       │
│                                         │
└─────────────────────────────────────────┘
```

---

## Implementation Guide

### Step 1: Update Radar Panel

File: `src/ui/file_share/radar.rs`

```rust
#[component]
pub fn Radar() -> Element {
    // State
    let mut my_code = use_signal(|| String::new());
    let mut input_code = use_signal(|| String::new());
    let mut devices = use_signal(|| Vec::new());
    let mut scanning = use_signal(|| false);
    
    // Generate my code on mount
    use_effect(move || {
        spawn(async move {
            if let Some(ip) = get_primary_ip() {
                let config = load_config().ok()?;
                match generate_my_code(
                    config.identity.id,
                    ip,
                    45679,
                    config.identity.hostname,
                    config.identity.label,
                ) {
                    Ok(code) => my_code.set(code),
                    Err(e) => println!("Error generating code: {}", e)
                }
            }
        });
    });
    
    rsx! {
        div {
            class: "radar-panel",
            
            // Header with my code
            div {
                class: "my-device-section",
                h3 { "Your Device: {get_device_name()}" }
                div {
                    class: "my-code",
                    "Code: "
                    span { class: "code-display", "{my_code}" }
                }
                p { class: "code-hint", "Share this code to receive files" }
            }
            
            // Radar animation
            div {
                class: "radar-container",
                if scanning() {
                    div { class: "radar-animation",
                        // Radar SVG animation
                    }
                }
                
                // Discovered devices
                div {
                    class: "discovered-devices",
                    h4 { "Found Devices:" }
                    for device in devices() {
                        div {
                            class: "device-item",
                            span { "{device.label} ({device.ip_address})" }
                            button {
                                onclick: move |_| {
                                    // Connect to device
                                },
                                "Connect"
                            }
                        }
                    }
                }
            }
            
            // Manual connect section
            div {
                class: "connect-section",
                h4 { "Connect to Device:" }
                div {
                    class: "code-input-group",
                    input {
                        r#type: "text",
                        placeholder: "Enter 4-digit code",
                        maxlength: 4,
                        pattern: "[0-9]*",
                        value: "{input_code}",
                        oninput: move |e| {
                            let val = e.value();
                            // Only allow numbers
                            if val.chars().all(|c| c.is_numeric()) {
                                input_code.set(val);
                            }
                        }
                    }
                    button {
                        disabled: input_code().len() != 4,
                        onclick: move |_| {
                            let code = input_code();
                            spawn(async move {
                                match connect_with_code(&code) {
                                    Ok(device_info) => {
                                        // Add to devices and open file explorer
                                        add_manual_device(&device_info.ip_address, device_info.bridge_port).await.ok();
                                        // Open file explorer UI
                                    }
                                    Err(e) => println!("Connection error: {}", e)
                                }
                            });
                        },
                        "Connect"
                    }
                }
            }
        }
    }
}
```

### Step 2: Create File Explorer Component

File: `src/ui/file_share/file_explorer.rs`

```rust
#[component]
pub fn FileExplorer(device_id: String, device_label: String) -> Element {
    let mut current_path = use_signal(|| PathBuf::from(dirs::home_dir().unwrap()));
    let mut search_query = use_signal(|| String::new());
    let mut files = use_signal(|| Vec::new());
    let mut folders = use_signal(|| Vec::new());
    let mut selected_file = use_signal(|| None::<PathBuf>);
    
    // Load directory contents
    use_effect(move || {
        let path = current_path();
        spawn(async move {
            if let Ok(entries) = std::fs::read_dir(&path) {
                let mut f = Vec::new();
                let mut d = Vec::new();
                
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_dir() {
                            d.push(entry.path());
                        } else {
                            f.push(entry.path());
                        }
                    }
                }
                
                folders.set(d);
                files.set(f);
            }
        });
    });
    
    rsx! {
        div {
            class: "file-explorer",
            
            // Header
            div {
                class: "explorer-header",
                h3 { "Select File to Share" }
                p { "Connected to: {device_label}" }
            }
            
            // Search bar
            div {
                class: "search-bar",
                input {
                    r#type: "text",
                    placeholder: "🔍 Search files...",
                    value: "{search_query}",
                    oninput: move |e| search_query.set(e.value())
                }
            }
            
            // Breadcrumb
            div {
                class: "breadcrumb",
                "📍 Current: {current_path().display()}"
                button {
                    onclick: move |_| {
                        if let Some(parent) = current_path().parent() {
                            current_path.set(parent.to_path_buf());
                        }
                    },
                    "← Back"
                }
            }
            
            // Folders
            div {
                class: "folders-section",
                h4 { "📂 Folders:" }
                for folder in folders() {
                    div {
                        class: "folder-item",
                        onclick: move |_| {
                            current_path.set(folder.clone());
                        },
                        "📁 {folder.file_name().unwrap().to_string_lossy()}"
                    }
                }
            }
            
            // Files
            div {
                class: "files-section",
                h4 { "📄 Files:" }
                for file in files() {
                    div {
                        class: "file-item",
                        onclick: move |_| {
                            selected_file.set(Some(file.clone()));
                            // Show confirmation dialog
                        },
                        "{get_file_icon(&file)} {file.file_name().unwrap().to_string_lossy()}"
                        span { class: "file-size", "({format_file_size(file.metadata().unwrap().len())})" }
                    }
                }
            }
            
            // Quick access
            div {
                class: "quick-access",
                button {
                    onclick: move |_| current_path.set(dirs::desktop_dir().unwrap()),
                    "💾 Desktop"
                }
                button {
                    onclick: move |_| current_path.set(dirs::document_dir().unwrap()),
                    "📁 Documents"
                }
                button {
                    onclick: move |_| current_path.set(dirs::download_dir().unwrap()),
                    "⬇️ Downloads"
                }
            }
        }
    }
}

fn get_file_icon(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("pdf") => "📄",
        Some("jpg") | Some("jpeg") | Some("png") => "🖼️",
        Some("mp3") | Some("wav") => "🎵",
        Some("mp4") | Some("avi") | Some("mkv") => "📹",
        Some("zip") | Some("rar") => "📦",
        _ => "📄"
    }
}
```

### Step 3: Confirmation Dialog

```rust
#[component]
pub fn ShareConfirmation(
    file_path: PathBuf,
    device_label: String,
    device_ip: String,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let file_name = file_path.file_name().unwrap().to_string_lossy().to_string();
    let file_size = file_path.metadata().ok().map(|m| format_file_size(m.len())).unwrap_or_default();
    
    rsx! {
        div {
            class: "confirmation-dialog-overlay",
            div {
                class: "confirmation-dialog",
                h3 { "⚠️ Confirm File Share" }
                
                p { "Do you want to share this file?" }
                
                div {
                    class: "file-info",
                    p { "📄 {file_name}" }
                    p { "Size: {file_size}" }
                    p { "To: {device_label} ({device_ip})" }
                }
                
                div {
                    class: "dialog-buttons",
                    button {
                        class: "cancel-btn",
                        onclick: move |_| on_cancel.call(()),
                        "Cancel"
                    }
                    button {
                        class: "confirm-btn",
                        onclick: move |_| on_confirm.call(()),
                        "Share File"
                    }
                }
            }
        }
    }
}
```

## Summary

✅ **4-digit code system** - Easy to type
✅ **Radar panel** - Shows my code + discovered devices
✅ **Code input** - Connect to other devices
✅ **File explorer** - Browse folders/drives
✅ **Search bar** - Find files quickly
✅ **Confirmation dialog** - Before sending
✅ **Progress tracking** - Real-time transfer status

Yeh complete UI specification hai! Ab implement karna easy hoga! 🚀
