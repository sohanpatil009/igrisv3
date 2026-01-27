# Automatic Firewall Permission System

## What It Does

The app now automatically checks and requests firewall permissions when File Share initializes. This ensures QUIC connections work without manual firewall configuration.

## How It Works

### macOS
1. **Automatic Detection**: Checks if macOS firewall is enabled
2. **Permission Dialog**: When you first use File Share, macOS shows a native permission dialog
3. **User Action**: Click "Allow" to enable incoming connections
4. **No Admin Required**: Works without administrator privileges

**What you'll see:**
```
[FileShare] Checking firewall permissions...
[Firewall] Checking macOS firewall permissions...
[Firewall] macOS firewall is enabled
[Firewall] Requesting firewall permission...
[Firewall] macOS will show a permission dialog when you first use File Share
[Firewall] Please click 'Allow' when prompted
[FileShare] Firewall permissions OK
```

### Windows
1. **Automatic Detection**: Checks if firewall rule exists
2. **Auto-Create Rule**: Attempts to create firewall rule automatically
3. **Admin Check**: If not admin, shows manual instructions
4. **Fallback**: Provides PowerShell commands if auto-creation fails

**What you'll see (as Admin):**
```
[FileShare] Checking firewall permissions...
[Firewall] Checking Windows firewall permissions...
[Firewall] Firewall rule not found - attempting to create...
[Firewall] ✓ Inbound firewall rule created successfully
[Firewall] ✓ Outbound firewall rule created successfully
[FileShare] Firewall permissions OK
```

**What you'll see (not Admin):**
```
[Firewall] ⚠️  Need administrator privileges to add firewall rule
[Firewall] Please run as administrator or manually add firewall rule:
[Firewall]   1. Open Windows Defender Firewall
[Firewall]   2. Click 'Advanced settings'
[Firewall]   3. Add Inbound Rule for UDP port 45679
```

## Manual Setup (If Needed)

### macOS
If the automatic dialog doesn't appear:
1. Open **System Settings** → **Network** → **Firewall**
2. Click **Options**
3. Click **+** to add an application
4. Navigate to IGRIS binary (e.g., `~/ai/igrisv3/target/release/igrisv3`)
5. Select it and click **Add**
6. Set to **Allow incoming connections**

### Windows (PowerShell as Admin)
```powershell
New-NetFirewallRule -DisplayName "IGRIS File Share" `
  -Direction Inbound -Protocol UDP -LocalPort 45679 `
  -Action Allow

New-NetFirewallRule -DisplayName "IGRIS File Share" `
  -Direction Outbound -Protocol UDP -LocalPort 45679 `
  -Action Allow
```

### Windows (GUI)
1. Open **Windows Defender Firewall with Advanced Security**
2. Click **Inbound Rules** → **New Rule**
3. Rule Type: **Port**
4. Protocol: **UDP**, Port: **45679**
5. Action: **Allow the connection**
6. Apply to all profiles
7. Name: **IGRIS File Share**
8. Repeat for **Outbound Rules**

## Testing

After updating:

**On both Mac and Windows:**
```bash
git pull origin main
cargo build --release
cargo run --release
```

**Expected behavior:**
1. App starts
2. Firewall check runs automatically
3. macOS: Permission dialog appears (click "Allow")
4. Windows: Firewall rule created automatically (if admin)
5. File Share works without connection timeouts

**Logs to watch for:**
```
[FileShare] Checking firewall permissions...
[Firewall] Checking [OS] firewall permissions...
[Firewall] ✓ [Success message]
[FileShare] Firewall permissions OK
```

## Benefits

✅ **No Manual Configuration**: Firewall rules created automatically  
✅ **User-Friendly**: Native OS permission dialogs  
✅ **Cross-Platform**: Works on macOS and Windows  
✅ **Graceful Fallback**: Shows instructions if auto-setup fails  
✅ **One-Time Setup**: Permission persists across app restarts  

## Technical Details

### Files Changed
- `src/platform/firewall.rs` - New firewall permission module
- `src/platform/mod.rs` - Export firewall functions
- `src/file_share/manager.rs` - Call firewall check during initialization

### Firewall Rules Created
- **Protocol**: UDP
- **Port**: 45679 (QUIC)
- **Direction**: Inbound + Outbound
- **Action**: Allow
- **Scope**: All profiles (Domain, Private, Public)

### Security
- Only allows UDP port 45679 (QUIC)
- Specific to IGRIS application
- User must approve (macOS) or run as admin (Windows)
- No system-wide firewall changes

---

**Status**: Ready for testing  
**Commit**: `0b07b3a`  
**Last Updated**: January 27, 2026
