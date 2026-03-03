# IGRIS File Share - Go Backend

High-performance P2P file sharing backend for IGRIS, designed to work over mobile hotspot connections.

## Features

- **mDNS Discovery**: Automatic peer discovery on local network (mobile hotspot)
- **LocalSend Protocol v2.1**: Compatible with LocalSend apps
- **REST API**: Easy integration with Rust frontend
- **WebSocket**: Real-time transfer progress updates
- **Concurrent Transfers**: Handle multiple file transfers simultaneously
- **Resume Support**: Resume interrupted transfers (planned)
- **SHA-256 Verification**: Ensure file integrity

## Quick Start

### Build

```bash
cd go-fileshare
go mod download
go build -o fileshare
```

### Run

```bash
# Default settings
./fileshare

# Custom settings
./fileshare -name "My Desktop" -port 53317 -config config.json
```

### Configuration

Create `config.json`:

```json
{
  "device_name": "IGRIS",
  "port": 53317,
  "download_dir": "./downloads",
  "auto_accept_trusted": false,
  "max_transfer_size": 10737418240,
  "chunk_size": 65536,
  "enabled": true
}
```

## API Endpoints

### LocalSend Protocol (Compatible)

- `GET /api/localsend/v2/info` - Device information
- `POST /api/localsend/v2/register` - Register device
- `POST /api/localsend/v2/prepare-upload` - Prepare file transfer
- `POST /api/localsend/v2/upload` - Upload file
- `POST /api/localsend/v2/cancel` - Cancel transfer

### IGRIS Custom API

- `GET /api/igris/devices` - List discovered devices
- `GET /api/igris/transfers` - List all transfers
- `GET /api/igris/transfer/:id` - Get transfer details
- `POST /api/igris/send` - Send files to device
- `DELETE /api/igris/transfer/:id` - Cancel transfer
- `GET /api/igris/ws` - WebSocket for real-time updates

## Mobile Hotspot Setup

### Windows

1. Open Settings → Network & Internet → Mobile hotspot
2. Turn on "Share my Internet connection"
3. Connect both desktops to the hotspot
4. Run fileshare on both machines

### macOS

1. System Preferences → Sharing → Internet Sharing
2. Share from: Wi-Fi, To: iPhone USB
3. Connect devices and run fileshare

### Linux

```bash
# Create hotspot
nmcli dev wifi hotspot ssid IGRIS password igris123

# Connect other device and run
./fileshare
```

## Integration with Rust

### HTTP Client Example

```rust
use reqwest;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    // Get discovered devices
    let devices: Value = client
        .get("http://localhost:53317/api/igris/devices")
        .send()
        .await?
        .json()
        .await?;
    
    println!("Devices: {:#?}", devices);
    Ok(())
}
```

### WebSocket Example

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::StreamExt;

#[tokio::main]
async fn main() {
    let (ws_stream, _) = connect_async("ws://localhost:53317/api/igris/ws")
        .await
        .expect("Failed to connect");
    
    let (_, mut read) = ws_stream.split();
    
    while let Some(msg) = read.next().await {
        if let Ok(Message::Text(text)) = msg {
            println!("Transfer update: {}", text);
        }
    }
}
```

## Architecture

```
┌─────────────────────────────────────┐
│         Go Backend                  │
├─────────────────────────────────────┤
│  HTTP Server (Gin)                  │
│  ├─ LocalSend API                   │
│  ├─ IGRIS Custom API                │
│  └─ WebSocket                       │
├─────────────────────────────────────┤
│  mDNS Discovery (zeroconf)          │
│  ├─ Broadcasting                    │
│  └─ Peer Discovery                  │
├─────────────────────────────────────┤
│  Transfer Manager                   │
│  ├─ Session Management              │
│  ├─ Progress Tracking               │
│  └─ File I/O                        │
└─────────────────────────────────────┘
```

## Testing

```bash
# Run tests
go test ./...

# Test discovery
curl http://localhost:53317/api/localsend/v2/info

# Test device list
curl http://localhost:53317/api/igris/devices

# Health check
curl http://localhost:53317/health
```

## Performance

- **Discovery**: < 1 second on local network
- **Transfer Speed**: 10-100 MB/s (WiFi dependent)
- **Memory**: ~20MB idle, ~50MB during transfer
- **CPU**: < 2% idle, < 10% during transfer

## Troubleshooting

### Devices Not Discovered

- Ensure both devices on same network (mobile hotspot)
- Check firewall allows UDP port 53317
- Disable AP isolation if using router

### Transfer Fails

- Check disk space in download directory
- Verify file permissions
- Ensure network is stable

### Slow Transfer

- Use 5GHz hotspot instead of 2.4GHz
- Reduce distance between devices
- Close bandwidth-heavy applications

## Dependencies

- `gin-gonic/gin` - HTTP framework
- `gorilla/websocket` - WebSocket support
- `grandcat/zeroconf` - mDNS discovery
- `google/uuid` - UUID generation

## License

MIT License - Same as IGRIS v3

## Contributing

1. Fork the repository
2. Create feature branch
3. Commit changes
4. Open Pull Request

---

**IGRIS File Share Backend** - Fast, reliable P2P file sharing over mobile hotspot.
