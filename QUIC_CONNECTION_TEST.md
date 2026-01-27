# QUIC Connection Testing Guide

## Status: Ready for Testing ✅

### What's Fixed:
1. ✅ UI now checks actual QUIC connection status
2. ✅ Shows "Reconnect" button for trusted but disconnected devices
3. ✅ Shows "Connect" button for untrusted devices
4. ✅ Shows "✓ Connected (QUIC)" only when actual QUIC connection exists

---

## Testing Steps

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

### Step 2: Run Apps

**Mac:**
```bash
cargo run --release
```

**Windows:**
```bash
cargo run --release
```

### Step 3: Test QUIC Connection

1. **Open File Share Panel** on both devices
2. **Click "Scan Again"** - devices should discover each other
3. **Check UI:**
   - If device shows "🔗 Reconnect" button → Click it!
   - If device shows "🔗 Connect" button → Click it!
   - If device shows "✓ Connected (QUIC)" → Already connected!

### Step 4: Watch Logs

**Expected logs when connecting:**

```
[QuicBridge] Connecting to SOHAN-PATIL911 at 10.106.46.244:45679
[QuicBridge] QUIC connection established to SOHAN-PATIL911
[ConnectionCoordinator] Handshake sent, waiting for response...
[ConnectionCoordinator] Handshake response received
[ConnectionCoordinator] Trust established with SOHAN-PATIL911
[ConnectionCoordinator] Adding device to QUIC BridgeManager...
```

### Step 5: Test File Transfer

1. **Create test file:**
   ```bash
   echo "QUIC test from Mac" > ~/Desktop/test.txt
   ```

2. **Send file:**
   - Drag file to Windows device in UI
   - Or use voice: "Send file test.txt to Windows"

3. **Watch logs:**
   ```
   [Transfer] Initiated send: test.txt to 291f3ff3
   [QuicBridge] Sending FileTransferRequest
   [Transfer] Transfer started
   [Transfer] Progress: 100%
   [Transfer] Completed: test.txt
   ```

---

## Expected Results

### ✅ Success Indicators:
- UI shows "✓ Connected (QUIC)" after clicking Reconnect
- Logs show QUIC connection established
- File transfer works
- Both devices can send/receive files

### ❌ Failure Indicators:
- Button stays as "Reconnect" after clicking
- No QUIC connection logs
- File transfer fails
- Error messages in logs

---

## Troubleshooting

### Issue: "Reconnect" button doesn't work

**Check:**
1. Windows firewall allows UDP 45679
2. Both devices on same WiFi
3. No VPN blocking UDP traffic

**Fix:**
```bash
# Windows (PowerShell as Admin):
New-NetFirewallRule -DisplayName "IGRIS QUIC" -Direction Inbound -Protocol UDP -LocalPort 45679 -Action Allow

# Mac:
# System Settings → Network → Firewall → Allow IGRIS
```

### Issue: Connection timeout

**Check logs for:**
```
[QuicBridge] Failed to initiate connection: <error>
[QuicBridge] Connection failed: <error>
```

**Common causes:**
- Firewall blocking
- Wrong IP address
- Port already in use
- Network isolation

### Issue: File transfer fails

**Check:**
1. QUIC connection established first
2. Sufficient disk space
3. File permissions
4. Transfer manager initialized

---

## Performance Benchmarks

### Expected Performance:
- **Connection time**: ~100ms (QUIC) vs ~200ms (old TCP+TLS)
- **File transfer**: Same speed as before
- **Multiple files**: Parallel transfers via QUIC streams

### Test Large File:
```bash
# Create 100MB test file
dd if=/dev/zero of=~/Desktop/large_test.bin bs=1M count=100

# Send and measure time
time <send via UI>
```

---

## Next Steps After Testing

1. ✅ Verify QUIC connection works
2. ✅ Test file transfers
3. ⏳ Test multiple simultaneous transfers
4. ⏳ Test connection migration (WiFi switch)
5. ⏳ Performance benchmarks
6. ⏳ Cross-subnet testing with relay

---

## Logs to Share

If issues occur, share these logs:

**Mac:**
```bash
cargo run --release 2>&1 | tee mac_quic_test.log
```

**Windows:**
```bash
cargo run --release 2>&1 | Tee-Object -FilePath windows_quic_test.log
```

---

**Status**: Ready for testing! 🚀
**Last Updated**: January 27, 2026
**Version**: QUIC v1.1 (UI Fixed)
