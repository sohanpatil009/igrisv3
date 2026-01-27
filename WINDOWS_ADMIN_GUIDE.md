# Windows Administrator Privileges Guide

## Why Admin is Needed

IGRIS needs administrator privileges on Windows to:
1. **Create firewall rules** for UDP port 45679 (QUIC connections)
2. **Allow incoming connections** for File Share feature
3. **Automatically configure** network permissions

## Methods to Run as Administrator

### Method 1: Right-Click (Easiest) ⭐
1. Navigate to: `F:\igrisv3\target\release\igrisv3.exe`
2. **Right-click** on `igrisv3.exe`
3. Select **"Run as administrator"**
4. Click **"Yes"** on the UAC prompt
5. Done! App runs with admin privileges

### Method 2: PowerShell as Admin
1. Press `Win + X`
2. Select **"Windows PowerShell (Admin)"** or **"Terminal (Admin)"**
3. Navigate to project:
   ```powershell
   cd F:\igrisv3
   cargo run --release
   ```
4. App runs with admin privileges

### Method 3: Command Prompt as Admin
1. Press `Win + R`
2. Type: `cmd`
3. Press `Ctrl + Shift + Enter` (opens as admin)
4. Navigate and run:
   ```cmd
   cd F:\igrisv3
   cargo run --release
   ```

### Method 4: Create Admin Shortcut (Permanent)
1. Right-click `igrisv3.exe` → **"Create shortcut"**
2. Right-click the shortcut → **"Properties"**
3. Click **"Advanced"** button
4. Check ✅ **"Run as administrator"**
5. Click **OK** → **OK**
6. Move shortcut to Desktop or Start Menu
7. **Double-click shortcut** → Always runs as admin!

### Method 5: Always Run as Admin (Permanent)
1. Right-click `igrisv3.exe` → **"Properties"**
2. Go to **"Compatibility"** tab
3. Check ✅ **"Run this program as an administrator"**
4. Click **Apply** → **OK**
5. Now **every time** you run the app, it requests admin automatically!

## Method 6: Automatic Admin Request (Built-in) 🚀

**NEW**: The app now has an embedded manifest that automatically requests admin privileges!

After the next build, when you run `igrisv3.exe`:
1. Windows shows UAC prompt automatically
2. Click **"Yes"**
3. App runs with admin privileges
4. Firewall rules created automatically

**How it works:**
- `igrisv3.exe.manifest` embedded in the executable
- Windows detects admin requirement
- Shows UAC prompt automatically
- No manual "Run as administrator" needed!

## What Happens When Running as Admin

**First time:**
```
[FileShare] Checking firewall permissions...
[Firewall] Checking Windows firewall permissions...
[Firewall] Firewall rule not found - attempting to create...
[Firewall] ✓ Inbound firewall rule created successfully
[Firewall] ✓ Outbound firewall rule created successfully
[FileShare] Firewall permissions OK
```

**Subsequent runs:**
```
[FileShare] Checking firewall permissions...
[Firewall] Checking Windows firewall permissions...
[Firewall] ✓ Firewall rule already exists
[FileShare] Firewall permissions OK
```

## If You Don't Want to Run as Admin

You can manually create the firewall rule once, then run normally:

**PowerShell (as Admin) - One Time:**
```powershell
New-NetFirewallRule -DisplayName "IGRIS File Share" `
  -Direction Inbound -Protocol UDP -LocalPort 45679 `
  -Action Allow

New-NetFirewallRule -DisplayName "IGRIS File Share" `
  -Direction Outbound -Protocol UDP -LocalPort 45679 `
  -Action Allow
```

After this, you can run the app normally without admin!

## Troubleshooting

### UAC Prompt Doesn't Appear
- Make sure you rebuilt after pulling latest code: `cargo build --release`
- Check if `igrisv3.exe.manifest` exists in project root
- Try Method 5 (Compatibility tab) as fallback

### "Access Denied" Error
- You clicked "No" on UAC prompt
- Run using Method 1 (Right-click → Run as administrator)

### Firewall Rule Creation Failed
- Antivirus might be blocking
- Temporarily disable antivirus
- Or manually create rules using PowerShell above

## Security Note

Admin privileges are only used to:
- Create firewall rules for UDP port 45679
- No system files modified
- No registry changes
- No background services installed
- Safe and transparent

---

**Recommended**: Use Method 5 (Compatibility tab) or wait for Method 6 (automatic) for best experience!

**Last Updated**: January 27, 2026
