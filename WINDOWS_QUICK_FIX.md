# 🪟 Windows Quick Fix Guide - IGRIS File Sharing

## Current Issue: Mac Can't Discover Windows

**What's Working:** ✅ Windows can see Mac  
**What's Not:** ❌ Mac can't see Windows

---

## 🚀 Quick Solution: Use 4-Digit Code System

Instead of waiting for automatic discovery, use the manual connection feature!

### Step-by-Step:

#### 1. Get Mac's Code

On **Mac terminal**, look for this line in the logs:
```
[Relay] Device registered with code: 9171
```

Or check the **Mac UI** - the code is displayed at the top of the File Share panel:
```
Your Device Code
┌────┬────┬────┬────┐
│ 9  │ 1  │ 7  │ 1  │
└────┴────┴────┴────┘
```

#### 2. Enter Code on Windows

On **Windows IGRIS**:
1. Open File Share panel
2. Scroll to bottom: "Connect to Device" section
3. Enter the 4-digit code: `9171`
4. Click **Connect** button

#### 3. Done! 🎉

Both devices are now connected and can share files!

---

## 🔧 Permanent Fix: Update Windows Code

To enable automatic discovery (Mac → Windows), update the Windows code:

### Option 1: Pull Latest Code (Easiest)

```powershell
# In PowerShell
cd F:\igrisv3
git pull origin main
cargo build --release
```

### Option 2: Manual Code Update

Update `src/file_share/discovery.rs` in the `run_broadcaster()` function:

**Add this code after line 265 (after `socket.bind()`):**

```rust
// Set multicast interface to all available interfaces
// This is critical for Windows to broadcast on the correct interface
if let Ok(interfaces) = get_if_addrs::get_if_addrs() {
    for iface in interfaces {
        if let get_if_addrs::IfAddr::V4(ref addr) = iface.addr {
            if !addr.ip.is_loopback() {
                match socket.set_multicast_if_v4(&addr.ip) {
                    Ok(_) => {
                        println!("[Discovery] Set multicast interface to {} ({})", iface.name, addr.ip);
                        break; // Use first non-loopback interface
                    }
                    Err(e) => {
                        println!("[Discovery] Failed to set multicast interface on {}: {}", iface.name, e);
                    }
                }
            }
        }
    }
}
```

Then rebuild:
```powershell
cargo build --release
```

---

## 🔍 Troubleshooting

### Issue 1: "Code not found or expired"

**Cause:** Code expires after 10 minutes

**Solution:** 
- Get a fresh code from Mac
- Codes refresh automatically every 10 minutes

### Issue 2: "Connection failed"

**Cause:** Network connectivity issue

**Solution:**
1. Check both devices are on same network
2. Ping Mac from Windows:
   ```cmd
   ping 10.106.46.121
   ```
3. Check Windows Firewall (see below)

### Issue 3: Windows Firewall Blocking

**Check Firewall:**
```powershell
# Check if IGRIS is allowed
Get-NetFirewallApplicationFilter | Where-Object {$_.Program -like "*igris*"}
```

**Allow IGRIS:**
```powershell
# Run as Administrator
New-NetFirewallRule -DisplayName "IGRIS Discovery" -Direction Inbound -Protocol UDP -LocalPort 45678 -Action Allow
New-NetFirewallRule -DisplayName "IGRIS File Transfer" -Direction Inbound -Protocol TCP -LocalPort 45679 -Action Allow
```

Or use GUI:
1. Open **Windows Defender Firewall**
2. Click **Advanced settings**
3. Click **Inbound Rules** → **New Rule**
4. Select **Port** → **Next**
5. Select **UDP** → Port: `45678` → **Next**
6. Select **Allow the connection** → **Next**
7. Check all profiles → **Next**
8. Name: "IGRIS Discovery" → **Finish**
9. Repeat for TCP port `45679`

### Issue 4: Can't See File Share Panel

**Solution:**
- Say: "Open file share" or "Share files"
- Or click the File Share button in UI

---

## 📊 Network Diagnostics

### Check Your IP Address
```cmd
ipconfig
```

Look for:
```
Wireless LAN adapter Wi-Fi:
   IPv4 Address. . . . . . . . . . . : 10.106.46.244
```

### Check if Ports are Listening
```powershell
netstat -an | findstr "45678"
netstat -an | findstr "45679"
```

Should show:
```
UDP    0.0.0.0:45678          *:*
TCP    0.0.0.0:45679          0.0.0.0:0              LISTENING
```

### Test Multicast Reception
```powershell
# Terminal 1: Listen
Test-NetConnection -ComputerName 239.255.45.67 -Port 45678

# Terminal 2: Check if receiving
# (Run IGRIS and check logs)
```

---

## 🎯 Expected Behavior After Fix

### Before Fix:
```
Windows Logs:
✅ [Discovery] Found device: Rohits-Laptop.local (macOS)

Mac Logs:
❌ [FileShare] Retrieved 0 devices
```

### After Fix:
```
Windows Logs:
✅ [Discovery] Found device: Rohits-Laptop.local (macOS)
✅ [Discovery] Set multicast interface to Wi-Fi (10.106.46.244)

Mac Logs:
✅ [Discovery] Received from 10.106.46.244
✅ [Discovery] Found device: SOHAN-PATIL911 (Windows)
✅ [FileShare] Retrieved 1 devices
```

---

## 📝 Quick Reference

### Your Network Info:
- **Windows IP:** `10.106.46.244`
- **Mac IP:** `10.106.46.121`
- **Subnet:** `10.106.46.x` ✅ (Same network)
- **Multicast Group:** `239.255.45.67`
- **Discovery Port:** `45678` (UDP)
- **Transfer Port:** `45679` (TCP)

### Current Codes:
- **Mac Code:** Check logs or UI (changes every 10 min)
- **Windows Code:** Check logs or UI (changes every 10 min)

### File Locations:
- **Windows Code:** `F:\igrisv3\`
- **Discovery Module:** `src/file_share/discovery.rs`
- **Logs:** Terminal output when running `dx serve`

---

## 🆘 Still Not Working?

### Collect Debug Info:

1. **Windows Logs:**
   ```powershell
   dx serve > windows_logs.txt 2>&1
   ```

2. **Mac Logs:**
   ```bash
   dx serve > mac_logs.txt 2>&1
   ```

3. **Network Test:**
   ```cmd
   # From Windows
   ping 10.106.46.121
   tracert 10.106.46.121
   ```

4. **Firewall Status:**
   ```powershell
   Get-NetFirewallProfile | Select-Object Name, Enabled
   ```

### Share These Files:
- `windows_logs.txt`
- `mac_logs.txt`
- Network test results
- Firewall status

---

## ✅ Success Checklist

- [ ] Both devices on same network (10.106.46.x)
- [ ] Windows can see Mac in device list
- [ ] 4-digit code system works
- [ ] Can send files from Windows to Mac
- [ ] Windows Firewall allows IGRIS
- [ ] Ports 45678 and 45679 are open

---

## 🎉 Working Configuration

Once everything is set up:

1. **Open File Share** on both devices
2. **Wait 15 seconds** for scanning
3. **See each other** in device list
4. **Click Connect** on any device
5. **Select file** and send
6. **Enjoy fast local file sharing!** 🚀

No internet required! No cloud! Pure P2P! 💪

---

## 📚 Additional Resources

- **Main README:** `README.md`
- **Mac Firewall Setup:** `FIREWALL_SETUP_README.md`
- **Architecture:** `ARCHITECTURE.md`
- **File Share Spec:** `FILE_SHARE_UI_SPEC.md`

---

## 💡 Pro Tips

1. **Keep codes handy:** Screenshot the codes for quick sharing
2. **Same network:** Always ensure both devices are on same Wi-Fi
3. **Firewall once:** Configure firewall once, works forever
4. **Update regularly:** Pull latest code for bug fixes
5. **Check logs:** Logs tell you exactly what's happening

---

**Remember: The 4-digit code system works RIGHT NOW, even without the permanent fix!** 🎯

Just enter Mac's code on Windows, and you're good to go! 🚀
