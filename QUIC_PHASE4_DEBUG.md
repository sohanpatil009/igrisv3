# QUIC Phase 4: Connection Testing & Debugging

## What We Just Did

### Added Comprehensive Logging
Enhanced logging throughout the QUIC connection flow to identify exactly where connections fail:

1. **`src/file_share/quic_bridge.rs`** - Added 10+ log points in `connect()` method
2. **`src/file_share/connection.rs`** - Added 15+ log points in `establish_quic_connection_with_handshake()`
3. **`src/ui/file_share/panel.rs`** - Added logging in `on_connect` handler and `connect_direct_async()`

### Created Debugging Tools

1. **`QUIC_DEBUG_GUIDE.md`** - Comprehensive debugging guide with:
   - Expected log flow
   - Common issues & fixes
   - Network verification commands
   - Step-by-step debugging process

2. **`test_quic_debug.sh`** - Quick test script to run IGRIS with filtered logs

3. **`QUIC_CONNECTION_TEST.md`** - Testing guide (already existed, now referenced)

## How to Use

### On Mac:
```bash
cd ~/ai/igrisv3
git pull origin main
cargo build --release
cargo run --release 2>&1 | tee mac_debug.log
```

### On Windows:
```bash
cd F:\igrisv3
git pull origin main
cargo build --release
cargo run --release 2>&1 | Tee-Object -FilePath windows_debug.log
```

### Test Connection:
1. Open File Share panel on Mac
2. Click "Scan Again"
3. Click "Reconnect" button on Windows device
4. Watch terminal logs

### Share Results:
Send the **last log message** you see before it stops, plus any error messages.

## Expected Behavior

### ✅ Success:
```
[QuicBridge] QUIC connection established to SOHAN-PATIL911
[ConnectionCoordinator] Handshake response received successfully
[ConnectionCoordinator] Trust established with SOHAN-PATIL911
[FileShare] Connected to SOHAN-PATIL911
```
UI shows: **"✓ Connected (QUIC)"**

### ❌ Failure Points:

1. **No logs** → Button not triggering
2. **Stops at "Initiating QUIC connection"** → Network/firewall issue
3. **Stops at "Waiting for connection"** → Timeout (Windows not responding)
4. **"Connection failed"** → QUIC handshake failed
5. **"Connection not found in map"** → Bug in connection storage

## What to Check

### 1. QUIC Endpoint Running?
Look for this in startup logs:
```
[QuicBridge] Endpoint initialized on UDP port 45679
```

### 2. Firewall Allowing UDP?
**Windows (PowerShell as Admin):**
```powershell
New-NetFirewallRule -DisplayName "IGRIS QUIC" -Direction Inbound -Protocol UDP -LocalPort 45679 -Action Allow
```

**Mac:**
System Settings → Network → Firewall → Allow IGRIS

### 3. Same WiFi Network?
Both devices should be on `10.106.46.x` network.

### 4. Can Ping Each Other?
**From Mac:**
```bash
ping 10.106.46.244
```

**From Windows:**
```bash
ping 10.106.46.121
```

## Next Steps

1. ✅ Pull latest code on both devices
2. ✅ Run with logging
3. ⏳ Click "Reconnect" and watch logs
4. ⏳ Share the last log message you see
5. ⏳ We'll fix the specific issue based on logs

## Files Changed

- `src/file_share/quic_bridge.rs` - Enhanced logging in `connect()`
- `src/file_share/connection.rs` - Enhanced logging in handshake
- `src/ui/file_share/panel.rs` - Enhanced logging in UI handlers
- `QUIC_DEBUG_GUIDE.md` - New debugging guide
- `test_quic_debug.sh` - New test script
- `QUIC_CONNECTION_TEST.md` - Testing guide

## Commits

- `8ecf198` - Add detailed QUIC connection logging for debugging
- `ec875b4` - Add QUIC debugging tools and comprehensive guide

---

**Status:** Ready for testing with detailed logging  
**Last Updated:** January 27, 2026  
**Next:** Test connection and share logs
