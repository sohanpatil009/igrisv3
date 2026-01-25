# Connection String UI - Easy Cross-Platform Sharing! 🚀

## Solution: Simple Copy-Paste Connection String

Instead of complex code lookup, use a simple shareable string!

## Format

```
igris://10.106.46.121:45679
```

**Benefits:**
- ✅ No code lookup needed
- ✅ Direct IP:Port in string
- ✅ Easy to copy-paste
- ✅ Works cross-platform (WhatsApp, SMS, Email)
- ✅ No internet required

## How It Works

### Device A (Mac - 10.106.46.121):
```
1. Get my IP address
2. Generate connection string: "igris://10.106.46.121:45679"
3. Show in UI (big, copyable)
4. User copies and shares via WhatsApp/SMS
```

### Device B (Windows - 192.168.1.20):
```
1. Receive connection string via WhatsApp/SMS
2. Paste in IGRIS: "igris://10.106.46.121:45679"
3. Click "Connect"
4. Auto-connect to 10.106.46.121:45679 ✅
```

## UI Implementation

### Add to `src/ui/file_share/radar.rs`:

```rust
#[component]
pub fn Radar(/* ... */) -> Element {
    // State for connection string
    let mut my_connection_string = use_signal(|| String::new());
    let mut paste_connection_string = use_signal(|| String::new());
    let mut show_share_section = use_signal(|| false);
    
    // Generate connection string on mount
    use_effect(move || {
        spawn(async move {
            if let Some(ip) = get_primary_ip() {
                let conn_str = file_share::generate_connection_string(ip, 45679);
                my_connection_string.set(conn_str);
            }
        });
    });
    
    rsx! {
        div {
            class: "radar-panel",
            
            // Header with share button
            div {
                class: "radar-header",
                h2 { "Nearby Devices" }
                button {
                    class: "share-btn",
                    onclick: move |_| show_share_section.set(!show_share_section()),
                    "📤 Share My Device"
                }
            }
            
            // Share section (collapsible)
            if show_share_section() {
                div {
                    class: "share-section",
                    h3 { "Share This Connection String" }
                    
                    div {
                        class: "connection-string-display",
                        input {
                            r#type: "text",
                            readonly: true,
                            value: "{my_connection_string}",
                            onclick: move |_| {
                                // Auto-select text on click
                            }
                        }
                        button {
                            class: "copy-btn",
                            onclick: move |_| {
                                // Copy to clipboard
                                let conn_str = my_connection_string();
                                #[cfg(target_arch = "wasm32")]
                                {
                                    use wasm_bindgen::prelude::*;
                                    #[wasm_bindgen]
                                    extern "C" {
                                        #[wasm_bindgen(js_namespace = navigator, js_name = clipboard)]
                                        static CLIPBOARD: web_sys::Clipboard;
                                    }
                                    let _ = CLIPBOARD.write_text(&conn_str);
                                }
                                println!("Copied: {}", conn_str);
                            },
                            "📋 Copy"
                        }
                    }
                    
                    p {
                        class: "share-instructions",
                        "Share this string via WhatsApp, SMS, or Email"
                    }
                }
            }
            
            // Connect via string section
            div {
                class: "connect-section",
                h3 { "Connect to Device" }
                
                div {
                    class: "connection-string-input",
                    input {
                        r#type: "text",
                        placeholder: "Paste connection string (igris://IP:PORT)",
                        value: "{paste_connection_string}",
                        oninput: move |e| paste_connection_string.set(e.value())
                    }
                    button {
                        class: "connect-btn",
                        onclick: move |_| {
                            let conn_str = paste_connection_string();
                            spawn(async move {
                                match file_share::parse_connection_string(&conn_str) {
                                    Ok((ip, port)) => {
                                        println!("Connecting to {}:{}", ip, port);
                                        match file_share::discovery::add_manual_device(&ip, port).await {
                                            Ok(device) => {
                                                println!("Device added: {}", device.label);
                                                paste_connection_string.set(String::new());
                                            }
                                            Err(e) => println!("Error: {}", e)
                                        }
                                    }
                                    Err(e) => println!("Invalid connection string: {}", e)
                                }
                            });
                        },
                        "🔗 Connect"
                    }
                }
                
                p {
                    class: "connect-instructions",
                    "Paste the connection string you received"
                }
            }
            
            // Divider
            hr {}
            
            // Discovered devices list
            h3 { "Discovered Devices" }
            // ... existing radar device list ...
        }
    }
}

// Helper function to get primary IP
fn get_primary_ip() -> Option<String> {
    use get_if_addrs::get_if_addrs;
    
    let interfaces = get_if_addrs().ok()?;
    for iface in interfaces {
        if let get_if_addrs::IfAddr::V4(ref addr) = iface.addr {
            if !addr.ip.is_loopback() && !addr.ip.is_link_local() {
                return Some(addr.ip.to_string());
            }
        }
    }
    None
}
```

### CSS Styling (add to `assets/main.css`):

```css
/* Share Section */
.share-section {
    background: #f0f8ff;
    border: 2px dashed #4a90e2;
    border-radius: 8px;
    padding: 20px;
    margin: 15px 0;
}

.connection-string-display {
    display: flex;
    gap: 10px;
    margin: 15px 0;
}

.connection-string-display input {
    flex: 1;
    padding: 12px;
    font-family: 'Courier New', monospace;
    font-size: 14px;
    border: 2px solid #4a90e2;
    border-radius: 6px;
    background: white;
}

.copy-btn {
    padding: 12px 24px;
    background: #4a90e2;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-weight: bold;
}

.copy-btn:hover {
    background: #357abd;
}

/* Connect Section */
.connect-section {
    background: #fff8f0;
    border: 2px dashed #ff9800;
    border-radius: 8px;
    padding: 20px;
    margin: 15px 0;
}

.connection-string-input {
    display: flex;
    gap: 10px;
    margin: 15px 0;
}

.connection-string-input input {
    flex: 1;
    padding: 12px;
    font-family: 'Courier New', monospace;
    font-size: 14px;
    border: 2px solid #ff9800;
    border-radius: 6px;
}

.connect-btn {
    padding: 12px 24px;
    background: #ff9800;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-weight: bold;
}

.connect-btn:hover {
    background: #e68900;
}

.share-instructions,
.connect-instructions {
    color: #666;
    font-size: 13px;
    margin-top: 10px;
    font-style: italic;
}

.share-btn {
    padding: 8px 16px;
    background: #4caf50;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
}

.share-btn:hover {
    background: #45a049;
}
```

## Voice Command Integration

### Add to `src/commands/file_share.rs`:

```rust
"share" | "file_share_share" => {
    handle_share_connection().await
}

"connect_string" | "file_share_connect_string" => {
    let conn_str = params.get("string").map(|s| s.as_str()).unwrap_or("");
    handle_connect_via_string(conn_str).await
}

async fn handle_share_connection() -> Result<String, Box<dyn Error>> {
    let ip = get_primary_ip().ok_or("Could not get IP address")?;
    let conn_str = generate_connection_string(ip, 45679);
    
    // Open share panel with connection string
    if let Ok(mut state) = FILE_SHARE_PANEL_STATE.lock() {
        *state = FileSharePanelState::Radar;
    }
    
    Ok(format!("Your connection string is: {}", conn_str))
}

async fn handle_connect_via_string(conn_str: &str) -> Result<String, Box<dyn Error>> {
    if conn_str.is_empty() {
        return Err("Please provide a connection string".into());
    }
    
    let (ip, port) = parse_connection_string(conn_str)?;
    
    // Add to discovered devices
    add_manual_device(&ip, port).await?;
    
    Ok(format!("Connected to {}:{}", ip, port))
}
```

## Usage Examples

### Example 1: WhatsApp Share

**Device A:**
```
1. Open IGRIS
2. Say: "file share scan"
3. Click "📤 Share My Device"
4. Click "📋 Copy"
5. Open WhatsApp
6. Paste: "igris://10.106.46.121:45679"
7. Send to friend
```

**Device B:**
```
1. Receive WhatsApp message
2. Copy connection string
3. Open IGRIS
4. Say: "file share scan"
5. Paste in "Connect to Device" box
6. Click "🔗 Connect"
7. ✅ Connected!
```

### Example 2: Voice Command

**Device A:**
```
User: "file share share"
IGRIS: "Your connection string is: igris://10.106.46.121:45679"
```

**Device B:**
```
User: "file share connect string igris://10.106.46.121:45679"
IGRIS: "Connected to 10.106.46.121:45679"
```

## Supported Formats

The parser accepts multiple formats:

```rust
// Full format
"igris://10.106.46.121:45679" ✅

// Without protocol
"10.106.46.121:45679" ✅

// IP only (uses default port 45679)
"10.106.46.121" ✅

// Invalid formats
"invalid" ❌
"10.106.46.121:invalid" ❌
```

## Testing

```rust
// Test connection string generation
let conn_str = generate_connection_string("10.106.46.121".to_string(), 45679);
assert_eq!(conn_str, "igris://10.106.46.121:45679");

// Test parsing
let (ip, port) = parse_connection_string("igris://10.106.46.121:45679").unwrap();
assert_eq!(ip, "10.106.46.121");
assert_eq!(port, 45679);

// Test without protocol
let (ip, port) = parse_connection_string("192.168.1.20:45679").unwrap();
assert_eq!(ip, "192.168.1.20");
assert_eq!(port, 45679);

// Test IP only
let (ip, port) = parse_connection_string("10.106.46.121").unwrap();
assert_eq!(ip, "10.106.46.121");
assert_eq!(port, 45679); // Default
```

## Summary

✅ **Simple copy-paste solution**
✅ **No code lookup needed**
✅ **Works cross-platform (WhatsApp, SMS, Email)**
✅ **No internet required**
✅ **Easy to implement in UI**

**User Flow:**
1. Device A: Generate → Copy → Share (WhatsApp/SMS)
2. Device B: Receive → Paste → Connect
3. ✅ Done!

Bahut simple aur practical solution! 🎉
