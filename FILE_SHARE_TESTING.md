# 🧪 File Sharing Testing Guide

## Current Status

✅ **Code Complete** - All modules implemented:
- Device discovery (multicast UDP)
- Bridge service (4-digit codes)
- Transfer manager
- Trust/crypto systems
- UI panel component

❓ **Not Verified** - Need to test if it actually works

---

## Quick Test Methods

### Method 1: Standalone Test (Recommended)

Run the standalone test to verify the backend works:

```bash
cargo run --bin test_file_share
```

**What it tests:**
- Device info creation
- File share manager initialization
- Service startup
- Bridge code generation
- Device discovery (10 second scan)
- Clean shutdown

**Expected output:**
```
🧪 Testing File Share Module
═══════════════════════════════════════

📱 Test 1: Device Info
✅ Device info created:
   ID: abc-123-def
   Name: Your-PC
   Type: Desktop
   OS: Windows 11 (x86_64)
   IP: 192.168.1.100

🔧 Test 2: File Share Manager
✅ Manager created

🚀 Test 3: Starting Services
🔍 Discovery service started on port 45678
📁 Transfer service started on port 45679
🌉 Bridge service started on port 45680
🚀 File sharing services started
📱 Device: Your-PC (abc-123-def)
🔗 Bridge Code: 1234
✅ Services started

🔑 Test 4: Bridge Code
✅ Your bridge code: 1234
   Share this code with other devices to connect

🔍 Test 5: Device Discovery
   Scanning for 10 seconds...
   No devices found (this is normal if no other devices are running)

🛑 Test 6: Stopping Services
✅ Services stopped cleanly

═══════════════════════════════════════
✅ All tests completed!
```

---

### Method 2: Full App Test

Run the full IGRIS app and look for the file share button:

```bash
dx serve
# or
cargo run
```

**What to look for:**

1. **Console logs** - Check terminal for:
   ```
   [FILE_SHARE_PANEL] Component mounted!
   [FILE_SHARE_PANEL] Initial state - is_open: false
   Initializing file sharing system...
   🔍 Discovery service started on port 45678
   📁 Transfer service started on port 45679
   🌉 Bridge service started on port 45680
   File sharing system ready
   ```

2. **UI Button** - Look for a blue button in the bottom-right corner:
   ```
   📡 File Share
   ```
   - Position: Fixed, bottom-right (20px from edges)
   - Color: Blue (#2563eb)
   - Should be visible on top of everything (z-index: 1000)

3. **Click the button** - Should open a panel showing:
   - Your 4-digit bridge code
   - Device discovery list
   - Manual code entry field

---

## Troubleshooting

### Issue: "No button visible"

**Possible causes:**
1. Component not rendering
2. CSS z-index issue
3. Button behind other elements

**Debug steps:**
```bash
# Check console for mount logs
[FILE_SHARE_PANEL] Component mounted!

# If you see this, component is rendering
# If not, check if FileSharePanel {} is in main.rs App component
```

**Fix:**
- Open browser DevTools (F12)
- Look for button element in DOM
- Check computed styles for `position: fixed` and `z-index: 1000`

---

### Issue: "Button visible but not working"

**Debug steps:**
```bash
# Click button and check console
[FILE_SHARE] Button clicked - opening panel

# If you see this, click handler works
# If not, check browser console for JS errors
```

---

### Issue: "Services not starting"

**Possible causes:**
1. Ports already in use (45678, 45679, 45680)
2. Firewall blocking
3. Network permissions

**Debug steps:**
```bash
# Check if ports are in use
netstat -an | findstr "45678"
netstat -an | findstr "45679"
netstat -an | findstr "45680"

# If ports are in use, kill the process or change ports in FileShareConfig
```

**Fix:**
- Close other apps using these ports
- Or modify `src/file_share/mod.rs` FileShareConfig::default() to use different ports

---

### Issue: "No devices discovered"

**This is NORMAL if:**
- No other devices running IGRIS on same network
- Devices on different subnets
- Firewall blocking multicast UDP

**To test discovery:**
1. Run IGRIS on two devices on same network
2. Wait 15-30 seconds
3. Check if devices appear in list

**Alternative:**
- Use the 4-digit code system instead
- Get code from Device A
- Enter code on Device B
- Should connect even across different networks

---

## Network Requirements

### Same Network (Auto-Discovery)
```
Device A: 192.168.1.100
Device B: 192.168.1.101
Subnet:   192.168.1.0/24 ✅

Result: Should discover automatically via multicast
```

### Different Networks (Manual Code)
```
Device A: 192.168.1.100
Device B: 10.0.0.50
Subnets:  Different ❌

Result: Use 4-digit bridge code to connect
```

---

## Firewall Configuration

### Windows
```powershell
# Allow UDP 45678 (Discovery)
netsh advfirewall firewall add rule name="IGRIS Discovery" dir=in action=allow protocol=UDP localport=45678

# Allow TCP 45679 (Transfer)
netsh advfirewall firewall add rule name="IGRIS Transfer" dir=in action=allow protocol=TCP localport=45679

# Allow TCP 45680 (Bridge)
netsh advfirewall firewall add rule name="IGRIS Bridge" dir=in action=allow protocol=TCP localport=45680
```

### macOS
```bash
# Run the firewall setup script
./setup_macos_firewall.sh
```

### Linux
```bash
sudo ufw allow 45678/udp
sudo ufw allow 45679/tcp
sudo ufw allow 45680/tcp
```

---

## Expected Behavior

### On Startup
1. File share manager initializes
2. Services start (discovery, transfer, bridge)
3. Bridge code generated (4 digits)
4. UI button appears in bottom-right

### When Button Clicked
1. Panel opens
2. Shows your bridge code
3. Starts scanning for devices
4. Updates device list every 5 seconds

### When Device Discovered
1. Device appears in list with:
   - Icon (💻 🖥️ 📱)
   - Name
   - OS
   - IP address
   - Connect button

### When Connect Clicked
1. Establishes connection
2. Shows "Connected!" message
3. Ready to transfer files

---

## Next Steps After Verification

Once you confirm it's working:

1. **Add file picker** - Let users select files to send
2. **Add transfer progress** - Show upload/download progress
3. **Add voice commands** - "Send file to [device]"
4. **Add transfer history** - Show recent transfers
5. **Add drag & drop** - Drag files onto device to send

---

## Quick Checklist

- [ ] Run `cargo run --bin test_file_share`
- [ ] See "All tests completed!"
- [ ] Run full app with `dx serve`
- [ ] See file share button in bottom-right
- [ ] Click button, panel opens
- [ ] See your 4-digit code
- [ ] (Optional) Test on 2 devices to verify discovery

---

## Still Not Working?

If you've tried everything and it's still not working:

1. **Check the logs** - Look for error messages in terminal
2. **Check diagnostics** - Run `cargo check` for compilation errors
3. **Simplify** - Comment out FileSharePanel in main.rs, rebuild, add back
4. **Test backend only** - Use the standalone test to isolate UI issues

The code is there and should work. Most likely it's a visibility/CSS issue or the services aren't starting due to port conflicts.
