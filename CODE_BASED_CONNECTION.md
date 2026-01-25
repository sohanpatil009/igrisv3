# Code-Based Device Connection - Easy Cross-Subnet File Sharing! 🎯

## Problem Solved!
IP address manually likhna bahut hectic hai! Ab sirf **6-digit code** se connect karo!

## How It Works

### Simple Flow:
```
Device A (Mac)                    Device B (Windows)
─────────────                     ──────────────────
1. Generate Code                  
   "123456" ←─────────────────→  2. Enter Code "123456"
                                  
3. Code maps to:                  4. Gets IP automatically:
   IP: 10.106.46.121                 IP: 10.106.46.121
   Port: 45679                       Port: 45679
                                  
5. Direct Connection Established! ✅
```

## Implementation

### New Module: `src/file_share/relay.rs`

**Features:**
- ✅ Generate random 6-digit codes
- ✅ Register device with code
- ✅ Lookup device by code
- ✅ Auto-expire codes after 10 minutes
- ✅ Local storage (no internet needed!)

### API Functions:

```rust
// Generate code for this device
let code = generate_connection_code(
    device_id,
    ip_address,
    bridge_port,
    hostname,
    label
)?;
// Returns: "123456"

// Connect using code
let device_info = connect_via_code("123456")?;
// Returns: DeviceRegistration with IP, port, etc.

// Remove code after connection
invalidate_code("123456")?;
```

## Usage Examples

### Example 1: Voice Command (To Be Implemented)

**Device A (Mac):**
```
User: "file share generate code"
IGRIS: "Your code is 1-2-3-4-5-6. Valid for 10 minutes."
```

**Device B (Windows):**
```
User: "file share connect code 1-2-3-4-5-6"
IGRIS: "Connected to Rohit's Mac!"
```

### Example 2: UI (To Be Implemented)

**Device A:**
1. Open file share panel
2. Click "Generate Connection Code"
3. Shows: **"123456"** (big, easy to read)
4. Share code with other person

**Device B:**
1. Open file share panel
2. Enter code: **123456**
3. Click "Connect"
4. Automatically connects!

### Example 3: Programmatic (Current - For Testing)

```rust
// Device A - Generate code
let my_ip = "10.106.46.121"; // Get from network interface
let code = file_share::generate_connection_code(
    device_id.clone(),
    my_ip.to_string(),
    45679,
    hostname.clone(),
    label.clone(),
)?;
println!("Share this code: {}", code);

// Device B - Connect using code
let device_info = file_share::connect_via_code(&code)?;
println!("Connecting to: {} at {}", device_info.label, device_info.ip_address);

// Add to discovered devices
file_share::discovery::add_manual_device(
    &device_info.ip_address,
    device_info.bridge_port
).await?;
```

## Advantages Over Manual IP Entry

| Manual IP Entry | Code-Based |
|----------------|------------|
| Type: `10.106.46.121` | Type: `123456` |
| 15 characters | 6 digits |
| Easy to mistype | Easy to remember |
| Need to know IP | Just share code |
| ❌ Hectic | ✅ Simple! |

## How to Get Your IP Address (For Code Generation)

### macOS:
```rust
use std::net::UdpSocket;

fn get_local_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}
```

### Or use existing network interface detection:
```rust
use get_if_addrs::get_if_addrs;

fn get_primary_ip() -> Option<String> {
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

## Next Steps: Integration

### Step 1: Add Voice Commands

Add to `src/commands/file_share.rs`:

```rust
"generate_code" | "file_share_generate_code" => {
    handle_generate_code().await
}

"connect_code" | "file_share_connect_code" => {
    let code = params.get("code").map(|s| s.as_str()).unwrap_or("");
    handle_connect_via_code(code).await
}

async fn handle_generate_code() -> Result<String, Box<dyn Error>> {
    let config = load_config()?;
    let ip = get_primary_ip().ok_or("Could not get IP address")?;
    
    let code = generate_connection_code(
        config.identity.id,
        ip,
        45679,
        config.identity.hostname,
        config.identity.label,
    )?;
    
    Ok(format!("Your connection code is: {}. Valid for 10 minutes.", code))
}

async fn handle_connect_via_code(code: &str) -> Result<String, Box<dyn Error>> {
    if code.is_empty() {
        return Err("Please provide a code".into());
    }
    
    let device_info = connect_via_code(code)?;
    
    // Add to discovered devices
    add_manual_device(&device_info.ip_address, device_info.bridge_port).await?;
    
    // Invalidate code after successful connection
    invalidate_code(code)?;
    
    Ok(format!("Connected to {}", device_info.label))
}
```

### Step 2: Add UI Components

Add to `src/ui/file_share/radar.rs`:

```rust
// Generate Code Section
div {
    class: "code-generation-section",
    h3 { "Share Your Device" }
    
    if let Some(code) = generated_code() {
        div {
            class: "connection-code",
            h1 { "{code}" }
            p { "Share this code with the other device" }
            p { class: "expiry", "Expires in 10 minutes" }
        }
    } else {
        button {
            onclick: move |_| {
                spawn(async move {
                    match generate_my_code().await {
                        Ok(code) => generated_code.set(Some(code)),
                        Err(e) => println!("Error: {}", e)
                    }
                });
            },
            "Generate Connection Code"
        }
    }
}

// Connect via Code Section
div {
    class: "code-connect-section",
    h3 { "Connect to Device" }
    input {
        r#type: "text",
        placeholder: "Enter 6-digit code",
        maxlength: 6,
        value: "{connection_code}",
        oninput: move |e| connection_code.set(e.value())
    }
    button {
        onclick: move |_| {
            let code = connection_code();
            spawn(async move {
                match connect_via_code(&code) {
                    Ok(device) => {
                        // Add to devices and connect
                        add_manual_device(&device.ip_address, device.bridge_port).await.ok();
                    }
                    Err(e) => println!("Error: {}", e)
                }
            });
        },
        "Connect"
    }
}
```

### Step 3: Add Plugin Command

Add to `src/plugins/builtin/file_share.rs`:

```rust
cmd!("generate connection code", "Generates a code to share your device", 
     &["generate connection code", "share code", "get code"], 
     ActionType::FileShare, "generate_code"),

cmd!("connect with code", "Connect to device using code", 
     &["connect with code", "enter code", "use code"], 
     ActionType::FileShare, "connect_code"),
```

## Testing

### Test 1: Generate Code
```rust
let code = generate_connection_code(
    "device123".to_string(),
    "10.106.46.121".to_string(),
    45679,
    "Rohits-Laptop".to_string(),
    "Rohit's Mac".to_string(),
)?;
println!("Generated code: {}", code);
```

### Test 2: Lookup Code
```rust
let device = connect_via_code(&code)?;
println!("Found device: {} at {}", device.label, device.ip_address);
```

### Test 3: Code Expiry
```rust
// Wait 11 minutes
std::thread::sleep(Duration::from_secs(660));

// Should fail
match connect_via_code(&code) {
    Err(e) => println!("Expected error: {}", e), // "Code expired"
    Ok(_) => println!("Unexpected success!"),
}
```

## Security Considerations

### Current Implementation (Local):
- ✅ Codes stored locally on each device
- ✅ 10-minute expiry
- ✅ Codes removed after use
- ⚠️ Only works if devices can communicate directly

### Future Enhancement (Optional - Remote Server):
If you want devices on completely different networks (e.g., different cities):

```rust
// POST to relay server
async fn register_with_server(code: &str, ip: &str) -> Result<(), Error> {
    let client = reqwest::Client::new();
    client.post("https://relay.igris.app/register")
        .json(&json!({
            "code": code,
            "ip": ip,
            "port": 45679
        }))
        .send()
        .await?;
    Ok(())
}

// GET from relay server
async fn lookup_from_server(code: &str) -> Result<DeviceInfo, Error> {
    let client = reqwest::Client::new();
    let response = client.get(&format!("https://relay.igris.app/lookup/{}", code))
        .send()
        .await?
        .json()
        .await?;
    Ok(response)
}
```

## Summary

✅ **Code-based connection implemented!**
✅ **6-digit codes instead of IP addresses**
✅ **Auto-expiry after 10 minutes**
✅ **Local storage (no internet needed)**
🔄 **Voice commands & UI need to be added**

**Benefits:**
- 🎯 Super easy to use
- 🔢 Just 6 digits to share
- ⏱️ Auto-expires for security
- 🚀 Works cross-subnet
- 💪 No manual IP typing!

**Next:** Add voice commands and UI to make it user-friendly! 🎉
