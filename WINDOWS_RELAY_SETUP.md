# Windows Relay Server Setup Guide

## Quick Setup (3 Options)

### Option 1: Mac as Relay Server (Easiest) ⭐

**On Mac:**
```bash
# Build and run relay server
cargo build --bin relay_server --release
./target/release/relay_server

# Note Mac's IP address (shown in logs or use ifconfig)
# Example: 10.11.81.121
```

**On Windows:**
```powershell
# Pull latest code
git pull

# Update relay address in code
# Edit src/file_share/quic_relay.rs line 147:
# Change "127.0.0.1:45680" to Mac's IP
# Example: "10.11.81.121:45680"

# Build
cargo build --release

# Run
.\target\release\igrisv3.exe
```

**Connect**: Both devices will automatically use Mac as relay!

---

### Option 2: Windows as Relay Server

**On Windows (Terminal 1 - PowerShell as Admin):**
```powershell
# Build relay server
cargo build --bin relay_server --release

# Allow through firewall
.\setup_windows_firewall.ps1

# Run relay server
.\target\release\relay_server.exe

# Note Windows IP address
# Example: 10.11.81.244
```

**On Windows (Terminal 2 - Regular PowerShell):**
```powershell
# Update relay address to localhost
# src/file_share/quic_relay.rs line 147:
# "127.0.0.1:45680"

# Build and run main app
cargo build --release
.\target\release\igrisv3.exe
```

**On Mac:**
```bash
# Update relay address to Windows IP
# src/file_share/quic_relay.rs line 147:
# "10.11.81.244:45680"

# Build and run
cargo build --release
./target/release/igrisv3
```

---

### Option 3: No Relay - Use Mac Hotspot (Simplest!)

**On Mac:**
```bash
# Create Personal Hotspot
System Settings → General → Sharing → Personal Hotspot → Turn On
```

**On Windows:**
```powershell
# Connect to Mac's WiFi hotspot
# Then run app normally
.\target\release\igrisv3.exe
```

**No relay needed!** Mac hotspot allows P2P by default.

---

## Detailed Windows Instructions

### Step 1: Pull Latest Code

```powershell
# Open PowerShell in project directory
cd C:\path\to\igrisv3

# Pull latest changes
git pull
```

### Step 2: Choose Your Setup

#### If Mac is Relay Server:

1. **Get Mac's IP Address** (from Mac terminal):
   ```bash
   ifconfig | grep "inet " | grep -v 127.0.0.1
   # Example output: inet 10.11.81.121
   ```

2. **Update Windows Code**:
   - Open `src/file_share/quic_relay.rs`
   - Find line 147 (in `get_default_relay_address()`)
   - Change:
     ```rust
     pub fn get_default_relay_address() -> String {
         "10.11.81.121:45680".to_string()  // Mac's IP
     }
     ```

3. **Build Windows App**:
   ```powershell
   cargo build --release
   ```

4. **Run Windows App**:
   ```powershell
   .\target\release\igrisv3.exe
   ```

#### If Windows is Relay Server:

1. **Build Relay Server**:
   ```powershell
   cargo build --bin relay_server --release
   ```

2. **Setup Firewall** (PowerShell as Admin):
   ```powershell
   .\setup_windows_firewall.ps1
   ```

3. **Run Relay Server** (PowerShell as Admin):
   ```powershell
   .\target\release\relay_server.exe
   ```

4. **Open Second PowerShell** (Regular):
   ```powershell
   # Build and run main app
   cargo build --release
   .\target\release\igrisv3.exe
   ```

5. **Update Mac Code** (on Mac):
   ```rust
   // src/file_share/quic_relay.rs line 147
   pub fn get_default_relay_address() -> String {
       "10.11.81.244:45680".to_string()  // Windows IP
   }
   ```

### Step 3: Test Connection

1. **Start Relay Server** (if using relay)
2. **Run App on Both Devices**
3. **Click "Connect"** in UI
4. **Check Logs**:
   - Should see "Direct connection failed"
   - Then "Attempting relay connection..."
   - Then "✓ Handshake complete via relay"

---

## Windows Commands Cheat Sheet

```powershell
# Pull code
git pull

# Build main app
cargo build --release

# Build relay server
cargo build --bin relay_server --release

# Run main app
.\target\release\igrisv3.exe

# Run relay server (as Admin)
.\target\release\relay_server.exe

# Check firewall
.\check_firewall_status.ps1

# Setup firewall (as Admin)
.\setup_windows_firewall.ps1

# Get Windows IP
ipconfig | findstr IPv4
```

---

## Troubleshooting

### "cargo: command not found"

```powershell
# Install Rust
# Download from: https://rustup.rs/
# Or use:
winget install Rustlang.Rustup
```

### "Permission denied" when running relay server

```powershell
# Run PowerShell as Administrator
# Right-click PowerShell → Run as Administrator
```

### Firewall blocks relay server

```powershell
# Run as Admin
.\setup_windows_firewall.ps1

# Or manually:
netsh advfirewall firewall add rule name="IGRIS Relay" dir=in action=allow protocol=UDP localport=45680
```

### Can't find Windows IP

```powershell
# Method 1
ipconfig

# Method 2
Get-NetIPAddress -AddressFamily IPv4 | Where-Object {$_.IPAddress -notlike "127.*"}
```

### Relay connection fails

```powershell
# Check if relay server is running
Get-Process | Where-Object {$_.ProcessName -like "*relay*"}

# Check if port is open
netstat -an | findstr :45680

# Test connectivity (from Mac)
# On Mac: nc -u WINDOWS_IP 45680
```

---

## Recommended Setup for Testing

**Easiest Way** (No relay needed):
1. Mac: Create Personal Hotspot
2. Windows: Connect to Mac's hotspot
3. Both: Run apps normally
4. ✓ Works immediately!

**With Relay** (For learning/production):
1. Mac: Run relay server
2. Windows: Update code with Mac's IP
3. Windows: Build and run
4. ✓ Works through relay!

---

## File Locations

```
C:\Users\YourName\ai\igrisv3\
├── src\
│   ├── bin\
│   │   └── relay_server.rs          # Relay server code
│   └── file_share\
│       ├── quic_relay.rs             # Edit line 147 for relay IP
│       └── relay_server.rs
├── target\
│   └── release\
│       ├── igrisv3.exe               # Main app
│       └── relay_server.exe          # Relay server
├── setup_windows_firewall.ps1        # Firewall setup
└── check_firewall_status.ps1         # Check firewall
```

---

## Quick Test Script

Save as `test_relay.ps1`:

```powershell
# Test Relay Setup
Write-Host "=== IGRIS Relay Test ===" -ForegroundColor Green

# Check if relay server exists
if (Test-Path ".\target\release\relay_server.exe") {
    Write-Host "✓ Relay server binary found" -ForegroundColor Green
} else {
    Write-Host "✗ Relay server not built. Run: cargo build --bin relay_server --release" -ForegroundColor Red
    exit
}

# Check if main app exists
if (Test-Path ".\target\release\igrisv3.exe") {
    Write-Host "✓ Main app binary found" -ForegroundColor Green
} else {
    Write-Host "✗ Main app not built. Run: cargo build --release" -ForegroundColor Red
    exit
}

# Get Windows IP
$ip = (Get-NetIPAddress -AddressFamily IPv4 | Where-Object {$_.IPAddress -notlike "127.*"} | Select-Object -First 1).IPAddress
Write-Host "✓ Windows IP: $ip" -ForegroundColor Green

Write-Host "`nReady to test!" -ForegroundColor Green
Write-Host "1. Run relay server: .\target\release\relay_server.exe" -ForegroundColor Yellow
Write-Host "2. Run main app: .\target\release\igrisv3.exe" -ForegroundColor Yellow
```

Run with:
```powershell
.\test_relay.ps1
```

---

## Summary

**Simplest**: Use Mac Personal Hotspot (no relay needed)  
**For Testing**: Mac as relay server  
**For Production**: Cloud relay server  

Choose based on your needs!
