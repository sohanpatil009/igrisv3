# QUIC Connection Debugging Guide

## Current Status
- ✅ Phase 1: QUIC implementation complete
- ✅ Phase 2: TCP+TLS code removed
- ✅ Phase 3: UI fixed to show actual QUIC connection status
- ⏳ Phase 4: Testing connection - **DEBUGGING IN PROGRESS**

## Issue
Mac shows "Reconnect" button for Windows device, but clicking it doesn't establish QUIC connection.

## Expected Log Flow

### When "Reconnect" button is clicked:

```
[FileShare] on_connect() called for device: SOHAN-PATIL911 at 10.106.46.244
[FileShare] Direct connecting to SOHAN-PATIL911 at 10.106.46.244
[FileShare] Spawned async task for connection
[FileShare] connect_direct_async() called for SOHAN-PATIL911 at 10.106.46.244:45679
[FileShare] Got connection coordinator, calling connect_direct()...
[ConnectionCoordinator] Direct connection to SOHAN-PATIL911 at 10.106.46.244:45679
[ConnectionCoordinator] establish_quic_connection_with_handshake() called for 10.106.46.244:45679
[ConnectionCoordinator] Created temp device: 10.106.46.244 at 10.106.46.244:45679
[ConnectionCoordinator] Calling mgr.connect()...
[QuicBridge] connect() called for device: 10.106.46.244 (temp_10_106_46_244)
[QuicBridge] Connecting to 10.106.46.244 at 10.106.46.244:45679
[QuicBridge] Parsed address: 10.106.46.244:45679
[QuicBridge] Initiating QUIC connection...
[QuicBridge] Waiting for connection to establish...
[QuicBridge] QUIC connection established to 10.106.46.244
[QuicBridge] Device 10.106.46.244 added to connections map
[ConnectionCoordinator] QUIC connection established, getting connection from map...
[ConnectionCoordinator] Opening bidirectional stream...
[ConnectionCoordinator] Sending handshake (XXX bytes)...
[ConnectionCoordinator] Handshake sent, waiting for response...
[ConnectionCoordinator] Reading response (XXX bytes)...
[ConnectionCoordinator] Handshake response received successfully
[ConnectionCoordinator] Received device_id from remote: 291f3ff3
[ConnectionCoordinator] Trust established with SOHAN-PATIL911
[ConnectionCoordinator] Adding device to QUIC BridgeManager...
[FileShare] Direct connection successful to: SOHAN-PATIL911 (type: NewConnection)
[FileShare] Connected to SOHAN-PATIL911
```

## Debugging Steps

### Step 1: Pull Latest Code (Both Devices)

**Mac:**
```bash
cd ~/ai/igrisv3
git pull origin main
cargo build --release
```

**Windows:**
```bash
cd F:\igrisv3
git pull origin main
cargo build --release
```

### Step 2: Run with Full Logging

**Mac:**
```bash
cargo run --release 2>&1 | tee mac_debug.log
```

**Windows:**
```bash
cargo run --release 2>&1 | Tee-Object -FilePath windows_debug.log
```

### Step 3: Test Connection

1. Open File Share panel on Mac
2. Click "Scan Again" - should see Windows device
3. Click "Reconnect" button
4. Watch terminal logs carefully

### Step 4: Identify Where It Fails

Check which log message is the **LAST** one you see:

#### If you see:
- ❌ **No logs at all** → Button click not triggering handler
- ❌ **Only `[FileShare] on_connect() called`** → Async spawn failing
- ❌ **Only `[FileShare] connect_direct_async() called`** → Coordinator creation failing
- ❌ **Only `[ConnectionCoordinator] establish_quic_connection_with_handshake()`** → QUIC bridge not initialized
- ❌ **Only `[QuicBridge] Initiating QUIC connection...`** → Network/firewall blocking
- ❌ **Only `[QuicBridge] Waiting for connection to establish...`** → Connection timeout
- ✅ **`[QuicBridge] QUIC connection established`** → Success!

## Common Issues & Fixes

### Issue 1: No logs when clicking button
**Cause:** UI not triggering handler  
**Fix:** Check if device is in `devices()` list

### Issue 2: "QUIC bridge not initialized"
**Cause:** QUIC endpoint not started  
**Fix:** Check `[QuicBridge] Endpoint initialized on UDP port 45679` in startup logs

### Issue 3: "Connection failed" after "Initiating QUIC connection"
**Cause:** Firewall blocking UDP 45679 or Windows not listening  
**Fix:**
```bash
# Windows (PowerShell as Admin):
New-NetFirewallRule -DisplayName "IGRIS QUIC" -Direction Inbound -Protocol UDP -LocalPort 45679 -Action Allow

# Mac:
# System Settings → Network → Firewall → Allow IGRIS
```

### Issue 4: Connection timeout
**Cause:** Network isolation or wrong IP  
**Fix:** Verify both devices on same WiFi, ping each other

### Issue 5: "Connection not found in map after connect"
**Cause:** QUIC connection succeeded but not added to map  
**Fix:** Bug in `quic_bridge.rs` - check `connections.insert()` call

## Network Verification

### Check UDP Port 45679

**Mac:**
```bash
lsof -i UDP:45679
```

**Windows:**
```bash
netstat -an | findstr 45679
```

Should show IGRIS process listening.

### Check Connectivity

**From Mac to Windows:**
```bash
nc -u -v 10.106.46.244 45679
```

**From Windows to Mac:**
```bash
Test-NetConnection -ComputerName 10.106.46.121 -Port 45679
```

## Windows Side Logs

Windows should show these logs when Mac connects:

```
[QuicBridge] Accepted connection from 10.106.46.121:XXXXX
[ConnectionCoordinator] Received connection from Rohits-Laptop.local (fd5da150)
[ConnectionCoordinator] Trust established with initiator Rohits-Laptop.local
[ConnectionCoordinator] Added initiator to discovery cache
[ConnectionCoordinator] Adding initiator to QUIC BridgeManager...
[ConnectionCoordinator] Sending ResponderAck to Rohits-Laptop.local
```

If Windows shows nothing, the QUIC connection isn't reaching it.

## Next Steps After Debugging

Once you identify where it fails, share:
1. The **last log message** you see
2. Any **error messages**
3. Logs from **both Mac and Windows**

Then we can fix the specific issue!

---

**Last Updated:** January 27, 2026  
**Commit:** 8ecf198
