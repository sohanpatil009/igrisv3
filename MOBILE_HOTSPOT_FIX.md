# Mobile Hotspot Discovery Fix

## What Changed

Fixed device discovery to work properly on mobile hotspots by:

1. **Smart IP Detection**: Automatically detects the correct WiFi IP (10.11.81.x) instead of link-local (169.254.x.x)
2. **Dual Discovery**: Uses both multicast AND broadcast for maximum compatibility
3. **Faster Announcements**: Reduced from 30s to 5s intervals for quicker discovery
4. **Better Logging**: Shows local IP being used for debugging

## How It Works

### Windows
- Detects WiFi adapter IP via `ipconfig`
- Skips link-local addresses (169.254.x.x)
- Uses proper hotspot IP (10.11.81.244)

### macOS
- Detects network interface via `ifconfig`
- Uses proper hotspot IP (10.11.81.121)

### Discovery Methods
1. **Multicast** (224.0.0.167:53317) - Standard LocalSend
2. **Broadcast** (255.255.255.255:53317) - Mobile hotspot fallback

## Testing

### 1. Check Your IP
```powershell
# Windows
ipconfig | findstr "IPv4 Wireless"

# macOS
ifconfig | grep "inet "
```

Both devices MUST be on same subnet (e.g., 10.11.81.x)

### 2. Run IGRIS
```powershell
cargo run --release
```

### 3. Check Logs

**Good logs (working):**
```
[mDNS] Starting broadcast loop for device: IGRIS (IP: 10.11.81.244)
[mDNS] Starting listen loop on port 53317 (Local IP: 10.11.81.244)
[mDNS] ✓ Joined multicast group 224.0.0.167
[mDNS] Listening for announcements...
[mDNS] Received 234 bytes from 10.11.81.121:53317
[mDNS] ✓ Discovered device: IGRIS at 10.11.81.121:53317
```

**Bad logs (link-local IP):**
```
[mDNS] Starting broadcast loop for device: IGRIS (IP: 169.254.111.90)
```

If you see 169.254.x.x, your Windows is NOT connected to hotspot properly!

## Troubleshooting

### Windows Not Getting Proper IP

1. **Disconnect from hotspot**
   - Settings → Network & Internet → WiFi
   - Click your hotspot → Disconnect

2. **Forget network**
   - Click "Manage known networks"
   - Find your hotspot → Forget

3. **Reconnect**
   - Turn off mobile hotspot
   - Wait 10 seconds
   - Turn on mobile hotspot
   - Connect from Windows

4. **Verify IP**
   ```powershell
   ipconfig | findstr "IPv4 Wireless"
   ```
   Should show: `10.11.81.x` (NOT 169.254.x.x)

### Still Not Working?

1. **Check firewall** (should auto-configure)
   ```powershell
   netsh advfirewall firewall show rule name="IGRIS File Share"
   ```

2. **Test ping**
   ```powershell
   # From Windows
   ping 10.11.81.121
   
   # From Mac
   ping 10.11.81.244
   ```

3. **Check port**
   ```powershell
   netstat -an | findstr 53317
   ```
   Should show: `UDP 0.0.0.0:53317`

### Mac Firewall

If Mac is blocking:
```bash
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --add /path/to/igrisv3
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --unblock /path/to/igrisv3
```

## Expected Behavior

1. **Startup**: Both devices announce every 5 seconds
2. **Discovery**: Devices appear within 5-10 seconds
3. **UI**: Device cards show in File Share panel
4. **Transfer**: Click "Send" to transfer files

## Success Indicators

✅ Local IP is 10.11.81.x (NOT 169.254.x.x)
✅ Multicast group joined successfully
✅ Devices discovered within 10 seconds
✅ No socket timeout errors
✅ Device cards appear in UI

## Notes

- Both devices must be on same mobile hotspot
- Firewall auto-configures on first run
- Discovery happens every 5 seconds
- Works offline (no internet needed)
- Compatible with LocalSend protocol v2.1
