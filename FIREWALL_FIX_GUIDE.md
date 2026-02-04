# Windows Firewall Fix for File Share

## Problem
Socket error 10060 (Connection timeout) - Windows Firewall is blocking UDP port 53317

## Automatic Fix (Recommended)

IGRIS now automatically configures firewall on startup. Just run as Administrator:

```powershell
# Run PowerShell as Administrator
Right-click PowerShell → Run as Administrator

# Navigate to IGRIS directory
cd F:\rust\igrisv3

# Run IGRIS
cargo run --release
```

The app will automatically create firewall rules for port 53317.

## Manual Fix (If Automatic Fails)

### Option 1: PowerShell Command (Run as Admin)

```powershell
# Allow UDP port 53317 inbound
New-NetFirewallRule -DisplayName "IGRIS File Share" -Direction Inbound -Protocol UDP -LocalPort 53317 -Action Allow

# Allow UDP port 53317 outbound
New-NetFirewallRule -DisplayName "IGRIS File Share Out" -Direction Outbound -Protocol UDP -LocalPort 53317 -Action Allow
```

### Option 2: Windows Firewall GUI

1. Open **Windows Defender Firewall with Advanced Security**
   - Press `Win + R`
   - Type `wf.msc`
   - Press Enter

2. Click **Inbound Rules** → **New Rule**

3. Select **Port** → Next

4. Select **UDP** → Specific local ports: `53317` → Next

5. Select **Allow the connection** → Next

6. Check all profiles (Domain, Private, Public) → Next

7. Name: `IGRIS File Share` → Finish

8. Repeat for **Outbound Rules**

### Option 3: Command Prompt (Run as Admin)

```cmd
netsh advfirewall firewall add rule name="IGRIS File Share" dir=in action=allow protocol=UDP localport=53317

netsh advfirewall firewall add rule name="IGRIS File Share Out" dir=out action=allow protocol=UDP localport=53317
```

## Verify Firewall Rules

```powershell
# Check if rule exists
netsh advfirewall firewall show rule name="IGRIS File Share"
```

## Test Port

```powershell
# Check if port is listening
netstat -an | findstr 53317
```

You should see:
```
UDP    0.0.0.0:53317          *:*
```

## Troubleshooting

### Still Getting Errors?

1. **Check Antivirus**
   - Some antivirus software blocks UDP multicast
   - Temporarily disable to test

2. **Check Network Type**
   - Go to Settings → Network & Internet
   - Make sure network is set to "Private" not "Public"

3. **Check Windows Firewall Status**
   ```powershell
   Get-NetFirewallProfile | Select-Object Name, Enabled
   ```

4. **Disable Firewall Temporarily (Testing Only)**
   ```powershell
   Set-NetFirewallProfile -Profile Domain,Public,Private -Enabled False
   ```
   
   **Remember to re-enable:**
   ```powershell
   Set-NetFirewallProfile -Profile Domain,Public,Private -Enabled True
   ```

## macOS Firewall

If using macOS, run:

```bash
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --add /path/to/igrisv3
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --unblock /path/to/igrisv3
```

## Linux Firewall

### UFW (Ubuntu/Debian)
```bash
sudo ufw allow 53317/udp
```

### firewalld (Fedora/RHEL)
```bash
sudo firewall-cmd --add-port=53317/udp --permanent
sudo firewall-cmd --reload
```

### iptables
```bash
sudo iptables -A INPUT -p udp --dport 53317 -j ACCEPT
sudo iptables -A OUTPUT -p udp --sport 53317 -j ACCEPT
```

## After Fixing

1. Restart IGRIS
2. Open File Share panel
3. Devices should now appear in the list
4. No more socket timeout errors

## Notes

- Port 53317 is the standard LocalSend protocol port
- UDP multicast address: 224.0.0.167:53317
- Both devices must be on the same network
- Both devices must have firewall configured

## Success Indicators

When working correctly, you'll see:
```
[FILE_SHARE] Configuring firewall for port 53317...
[FILE_SHARE] Firewall configured successfully
[FILE_SHARE] mDNS broadcasting and listening started
✓ File Share service started on port 53317
```

No socket errors should appear!
