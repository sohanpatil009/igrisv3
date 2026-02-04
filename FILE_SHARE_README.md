# IGRIS File Share Module

## Quick Start

### 1. Initialize File Share

```rust
use igrisv3::file_share::FileShareManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create file share manager
    let manager = FileShareManager::new(
        "IGRIS".to_string(),  // Device name
        53317                  // Port
    ).await?;
    
    // Start discovery and HTTP server
    manager.start().await?;
    
    println!("File share started on port 53317");
    
    // Keep running
    tokio::signal::ctrl_c().await?;
    
    // Cleanup
    manager.stop().await?;
    Ok(())
}
```

### 2. Discover Devices

```rust
// Get list of discovered devices
let devices = manager.get_devices().await;

for device in devices {
    println!("Found: {} ({})", device.alias, device.ip);
    println!("  ID: {}", device.id);
    println!("  Fingerprint: {}", device.fingerprint);
}
```

### 3. Send Files

```rust
// Send files to a device
let device_id = "192.168.1.100:53317";
let files = vec![
    "document.pdf".to_string(),
    "photo.jpg".to_string(),
];

let session_id = manager.send_files(device_id, files).await?;
println!("Transfer started: {}", session_id);

// Track progress
loop {
    if let Some(progress) = manager.get_progress(&session_id) {
        println!("Progress: {:.1}%", progress.percentage());
        
        if progress.is_complete() {
            println!("Transfer completed!");
            break;
        }
    }
    
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
}
```

### 4. Cancel Transfer

```rust
manager.cancel_transfer(&session_id).await?;
```

## UI Integration (Dioxus 0.7)

```rust
use dioxus::prelude::*;
use igrisv3::ui::FileSharePanel;

#[component]
fn App() -> Element {
    // Provide FileShareManager via context
    let file_share = use_signal(|| Some(Arc::new(RwLock::new(manager))));
    use_context_provider(|| file_share);
    
    rsx! {
        FileSharePanel {}
    }
}
```

## Voice Commands (Future)

```
"Show nearby devices"
"Share file document.pdf with laptop"
"Send photo to phone"
"Accept transfer"
"Reject transfer"
```

## Configuration

Create `pkg/file_share_config.json`:

```json
{
  "enabled": true,
  "port": 53317,
  "download_dir": "./downloads",
  "auto_accept_trusted": true,
  "max_transfer_size": 10737418240,
  "chunk_size": 65536
}
```

## Firewall Setup

### Windows
```powershell
netsh advfirewall firewall add rule name="IGRIS File Share" dir=in action=allow protocol=UDP localport=53317
netsh advfirewall firewall add rule name="IGRIS File Share" dir=in action=allow protocol=TCP localport=53317
```

### Linux
```bash
sudo ufw allow 53317/tcp
sudo ufw allow 53317/udp
```

### macOS
Allow in System Preferences > Security & Privacy > Firewall

## Troubleshooting

### Devices Not Discovered
- Ensure devices are on the same WiFi network
- Check firewall allows UDP port 53317
- Disable AP isolation on router

### Transfer Fails
- Check disk space in download directory
- Verify file permissions
- Ensure network is stable

### Slow Transfer
- Use 5GHz WiFi instead of 2.4GHz
- Reduce distance to router
- Close bandwidth-heavy applications

## Architecture

```
FileShareManager
├── MdnsDiscovery (UDP multicast)
├── DeviceRegistry (discovered devices)
├── TransferOrchestrator
│   ├── FileSender (outgoing)
│   └── FileReceiver (incoming)
└── FileShareApi (HTTP server)
    ├── /api/localsend/v2/info
    ├── /api/localsend/v2/register
    ├── /api/localsend/v2/prepare-upload
    ├── /api/localsend/v2/upload
    └── /api/localsend/v2/cancel
```

## Protocol

Based on **LocalSend Protocol v2.1**

- **Port:** 53317 (TCP/UDP)
- **Multicast:** 224.0.0.167:53317
- **Protocol:** HTTP (HTTPS in future)
- **Discovery:** mDNS (UDP multicast)
- **Transfer:** REST API (HTTP POST)

## Security

### Current
- Device fingerprints (SHA-256)
- Trust management
- File integrity checks (SHA-256)

### Future
- TLS/HTTPS transport
- End-to-end encryption
- Certificate pinning

## Compatibility

Compatible with:
- LocalSend (official app)
- Any device implementing LocalSend Protocol v2.1

Tested on:
- Windows 10+
- macOS 11+
- Linux (Ubuntu 20.04+)

## Performance

- **Discovery:** < 1 second
- **Transfer Speed:** 10-100 MB/s (WiFi dependent)
- **Memory:** ~50MB during transfer
- **CPU:** < 5% during transfer

## Examples

See `examples/file_share_demo.rs` for complete examples.

## Documentation

- [Architecture](FILE_SHARE_ARCHITECTURE.md)
- [Integration Status](FILE_SHARE_INTEGRATION_STATUS.md)
- [LocalSend Protocol](https://github.com/localsend/protocol)

---

**IGRIS File Share** - Secure P2P file transfer for IGRIS voice assistant.
