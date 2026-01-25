# 🔐 macOS Firewall Setup for IGRIS - Safe & Secure

## Why Firewall Configuration is Needed

IGRIS needs to accept incoming network connections for:
- **Device Discovery** (UDP port 45678) - Finding other IGRIS devices on your network
- **File Transfers** (TCP port 45679) - Sending and receiving files

## ✅ Safe Approach: Add IGRIS to Firewall Allow List

**We DO NOT turn off your firewall!** Instead, we add IGRIS to the allowed apps list. Your Mac stays secure! 🔒

---

## Option 1: Automatic Setup (Recommended) ⚡

Run the setup script (one-time):

```bash
sudo ./setup_macos_firewall.sh
```

This will:
1. ✅ Add IGRIS to firewall allow list
2. ✅ Keep your firewall ON and active
3. ✅ Only allow IGRIS ports (45678, 45679)
4. ✅ Maintain full system security

---

## Option 2: Manual Setup (GUI) 🖱️

1. Open **System Settings**
2. Go to **Network** → **Firewall**
3. Click **Options** button
4. Click **+** (plus) button
5. Navigate to IGRIS app location:
   - Development: `./target/debug/igrisv3`
   - Release: `./target/release/igrisv3`
6. Select **Allow incoming connections**
7. Click **OK**

---

## Option 3: Command Line (Manual) 💻

```bash
# Get IGRIS path
IGRIS_PATH="$(pwd)/target/debug/igrisv3"

# Add to firewall
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --add "$IGRIS_PATH"

# Unblock the app
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --unblock "$IGRIS_PATH"

# Verify
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --listapps | grep igris
```

---

## What Happens on First Run

When you start IGRIS, it will:

1. **Check firewall status**
2. **Check if IGRIS is allowed**
3. If NOT allowed:
   ```
   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   ⚠️  FIREWALL SETUP REQUIRED
   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   
   IGRIS needs firewall permission for file sharing.
   
   Run: sudo ./setup_macos_firewall.sh
   
   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   ```
4. **App continues to start** (doesn't block)
5. File sharing won't work until firewall is configured

---

## Security Benefits

### ✅ What We DO:
- Add IGRIS to firewall allow list
- Allow only specific ports (45678, 45679)
- Keep firewall ON and active
- Maintain system security

### ❌ What We DON'T DO:
- Turn off firewall
- Disable system protection
- Open all ports
- Compromise security

---

## Verification

Check if IGRIS is allowed:

```bash
# Check firewall status
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate

# Check if IGRIS is in allow list
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --listapps | grep igris

# Check if ports are listening
lsof -i :45678  # Discovery port
lsof -i :45679  # Bridge port
```

Expected output:
```
Firewall is enabled. (State = 1)
...
/path/to/igrisv3 ( Allow incoming connections )
...
igrisv3   12345  user   UDP *:45678
igrisv3   12345  user   TCP *:45679 (LISTEN)
```

---

## Troubleshooting

### Issue: "File sharing not working"

**Check 1: Is firewall configured?**
```bash
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --listapps | grep igris
```

**Check 2: Are ports listening?**
```bash
lsof -i :45678
lsof -i :45679
```

**Check 3: Can other devices reach you?**
```bash
# From another device, test connection
telnet YOUR_MAC_IP 45679
```

### Issue: "Setup script fails"

**Solution:** Run with sudo:
```bash
sudo ./setup_macos_firewall.sh
```

### Issue: "Permission denied"

**Solution:** Make script executable:
```bash
chmod +x setup_macos_firewall.sh
sudo ./setup_macos_firewall.sh
```

---

## For Windows Users

Windows Firewall is handled differently. On first connection, Windows will show a prompt:

```
Windows Security Alert
Windows Defender Firewall has blocked some features of this app.

Name: igrisv3.exe
Publisher: Unknown

[x] Private networks (home or work)
[x] Public networks

[ Allow access ]  [ Cancel ]
```

Click **Allow access** with both checkboxes selected.

---

## For Linux Users

Linux typically doesn't block local network traffic by default. If using `ufw`:

```bash
# Allow IGRIS ports
sudo ufw allow 45678/udp
sudo ufw allow 45679/tcp

# Check status
sudo ufw status
```

---

## Summary

✅ **Safe**: Firewall stays ON  
✅ **Secure**: Only IGRIS ports allowed  
✅ **Simple**: One-time setup  
✅ **Professional**: Industry-standard approach  

**Your Mac remains fully protected!** 🔐🚀

---

## Need Help?

If you're still having issues:

1. Check the logs when starting IGRIS
2. Look for `[Firewall]` messages
3. Run the verification commands above
4. Check that both devices are on the same network

**Remember: This is a ONE-TIME setup. Once configured, it works forever!** 💪
