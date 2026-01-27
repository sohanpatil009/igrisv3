# Trusted Devices Management - Complete Guide

## Overview

IGRIS File Sharing includes a comprehensive trusted devices management system that allows users to establish, view, and revoke trust relationships with other devices on the network.

---

## 🎯 Features

### ✅ Currently Implemented:

1. **Automatic Trust Establishment**
   - Trust created during first successful connection
   - Bidirectional trust (both devices trust each other)
   - Certificate fingerprint verification
   - Persistent storage in JSON file

2. **Trust Verification**
   - Check if device is trusted before connection
   - Certificate fingerprint validation
   - Expiry checking (30-day timeout)
   - Rate limiting (3 failed attempts = 5-minute block)

3. **Trusted Devices List**
   - View all trusted devices
   - See device details (name, OS, IP)
   - Show trust date and last connection
   - Expired device warnings

4. **Untrust/Remove Device**
   - Two-step confirmation to prevent accidents
   - Removes device from trusted list
   - Updates storage immediately
   - Requires manual reconnection after removal

5. **Device Rename**
   - Change device display name
   - Persists across sessions
   - Updates in real-time

---

## 📁 Storage Location

### Configuration File: `file_share.json`

**Windows**: `C:\Users\<username>\AppData\Roaming\IGRIS\file_share.json`
**macOS**: `~/Library/Application Support/IGRIS/file_share.json`
**Linux**: `~/.config/IGRIS/file_share.json`

### File Structure:
```json
{
  "identity": {
    "id": "291f3ff3a1b2c3d4e5f6...",
    "label": "SOHAN-PATIL911",
    "os": "Windows",
    "hostname": "SOHAN-PATIL911",
    "salt": "randomBase64String==",
    "created_at": "2026-01-27T10:30:00Z"
  },
  "trusted_devices": [
    {
      "id": "fd5da15073f27d94c017...",
      "label": "Rohits-Laptop.local",
      "os": "MacOS",
      "cert_fingerprint": "052a8a597ad1414b414be1c2f88ecd9b2cd2ed5a678ea2a220a676c00d9e65e5",
      "trusted_at": "2026-01-27T10:35:00Z",
      "last_connected": "2026-01-27T11:20:00Z"
    },
    {
      "id": "a1b2c3d4e5f6g7h8i9j0...",
      "label": "Work-Desktop",
      "os": "Windows",
      "cert_fingerprint": "1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0t1u2v3w4x5y6z7a8b9c0d1e2f",
      "trusted_at": "2026-01-20T09:15:00Z",
      "last_connected": "2026-01-25T16:45:00Z"
    }
  ],
  "bridge_port": 45679,
  "discovery_port": 45678
}
```

---

## 🔐 Security Features

### 1. Certificate Fingerprint Verification
- Each device has a unique TLS certificate
- SHA-256 fingerprint stored for each trusted device
- Verified on every connection attempt
- Prevents man-in-the-middle attacks

### 2. Trust Expiry
- **30-day timeout** without connection
- Automatic expiry checking
- Visual warning for expired devices (⚠ badge)
- Requires re-establishment after expiry

### 3. Rate Limiting
- **Maximum 3 failed connection attempts**
- **5-minute block** after exceeding limit
- Automatic cleanup after block expires
- Prevents brute force attacks

### 4. Bidirectional Trust
- Both devices must trust each other
- Trust established simultaneously during handshake
- Symmetric security model

---

## 🎨 User Interface

### My Devices Panel

Access via: **File Share Panel → My Devices Tab (📱 icon)**

#### Empty State:
```
┌─────────────────────────────────────────┐
│  📱 My Devices                     [X]  │
├─────────────────────────────────────────┤
│                                         │
│              🔗                         │
│       No trusted devices yet            │
│                                         │
└─────────────────────────────────────────┘
```

#### Device List:
```
┌─────────────────────────────────────────┐
│  📱 My Devices                     [X]  │
├─────────────────────────────────────────┤
│                                         │
│  💻 Rohits-Laptop.local                │
│     macOS                               │
│                                         │
│     Trusted since    Jan 27, 2026       │
│     Last connected   2 minutes ago      │
│                                         │
│     [✏️ Rename]  [🗑️ Remove]           │
│                                         │
├─────────────────────────────────────────┤
│                                         │
│  🖥️ Work-Desktop              ⚠ Expired│
│     Windows                             │
│                                         │
│     Trusted since    Jan 20, 2026       │
│     Last connected   2 days ago         │
│                                         │
│     [✏️ Rename]  [🗑️ Remove]           │
│                                         │
└─────────────────────────────────────────┘
```

#### Rename Mode:
```
┌─────────────────────────────────────────┐
│  💻 [Rohits-MacBook-Pro    ] [✓] [✕]   │
│     macOS                               │
│                                         │
│     Trusted since    Jan 27, 2026       │
│     Last connected   2 minutes ago      │
└─────────────────────────────────────────┘
```

#### Remove Confirmation:
```
┌─────────────────────────────────────────┐
│  💻 Rohits-Laptop.local                │
│     macOS                               │
│                                         │
│     ⚠ Remove this device?               │
│     [Remove]  [Cancel]                  │
└─────────────────────────────────────────┘
```

---

## 🔄 Trust Workflow

### First Connection (Trust Establishment):

```
Device A (Windows)                    Device B (Mac)
─────────────────                    ──────────────
1. Discover Device B
2. Click "Connect"
3. TLS Handshake ──────────────────> Accept Connection
4. Exchange Certificates <─────────> Exchange Certificates
5. Verify Fingerprint                Verify Fingerprint
6. Save to trusted_devices           Save to trusted_devices
7. Update file_share.json            Update file_share.json
8. Show "✓ Connected"                Show "✓ Connected"
```

### Subsequent Connections (Auto-Connect):

```
Device A (Windows)                    Device B (Mac)
─────────────────                    ──────────────
1. Discover Device B
2. Check is_trusted(device_b_id)
3. ✅ Trusted → Auto-connect ──────> Accept (already trusted)
4. Verify Certificate <───────────> Verify Certificate
5. Update last_connected             Update last_connected
6. Show "✓ Connected"                Show "✓ Connected"
```

### After Untrust:

```
Device A (Windows)                    Device B (Mac)
─────────────────                    ──────────────
1. Discover Device B
2. Check is_trusted(device_b_id)
3. ❌ Not Trusted → Show "Connect"
4. User must manually connect again
5. Trust re-established if connected
```

---

## 🛠️ Implementation Details

### Core Files:

1. **`src/file_share/config.rs`** (350 lines)
   - `TrustedDevice` struct definition
   - `DeviceConfig` with trusted_devices array
   - Storage functions (load/save)
   - Config file path management

2. **`src/file_share/trust.rs`** (450 lines)
   - `TrustManager` struct
   - Trust establishment logic
   - Rate limiting implementation
   - Certificate verification
   - Expiry checking

3. **`src/ui/file_share/my_devices.rs`** (200 lines)
   - My Devices panel UI
   - Device list rendering
   - Rename functionality
   - Remove confirmation dialog

4. **`src/ui/file_share/panel.rs`** (500+ lines)
   - Main file share panel
   - Tab navigation
   - Device list integration
   - Event handlers

### Key Functions:

#### Trust Management:
```rust
// Establish trust with a device
pub fn establish_trust(
    device_info: &DeviceInfo, 
    cert_fingerprint: &str
) -> Result<(), String>

// Check if device is trusted
pub fn is_device_trusted(device_id: &str) -> Result<bool, String>

// Get all trusted devices
pub fn get_all_trusted() -> Result<Vec<TrustedDevice>, String>

// Remove trust
pub fn remove_trusted(device_id: &str) -> Result<bool, String>

// Rename device
pub fn rename_trusted_device(
    device_id: &str, 
    new_label: &str
) -> Result<bool, String>
```

#### Rate Limiting:
```rust
// Check if device is rate limited
pub fn check_rate_limit(device_id: &str) -> Result<(), u64>

// Record failed attempt
pub fn record_failed_attempt(device_id: &str) -> Result<(), String>
```

#### Storage:
```rust
// Load configuration from disk
pub fn load_config() -> Result<DeviceConfig, String>

// Save configuration to disk
pub fn save_config(config: &DeviceConfig) -> Result<(), String>

// Get config file path
pub fn get_config_file_path() -> PathBuf
```

---

## 📊 Data Structures

### TrustedDevice:
```rust
pub struct TrustedDevice {
    /// Device fingerprint ID (SHA-256)
    pub id: String,
    
    /// User-defined label
    pub label: String,
    
    /// Operating system (Windows/macOS/Linux)
    pub os: OperatingSystem,
    
    /// TLS certificate fingerprint (SHA-256)
    pub cert_fingerprint: String,
    
    /// When trust was established
    pub trusted_at: DateTime<Utc>,
    
    /// Last successful connection (None if never connected)
    pub last_connected: Option<DateTime<Utc>>,
}

impl TrustedDevice {
    /// Check if trust has expired (30 days without connection)
    pub fn is_expired(&self) -> bool {
        match self.last_connected {
            Some(last) => (Utc::now() - last).num_days() > 30,
            None => (Utc::now() - self.trusted_at).num_days() > 30,
        }
    }
}
```

### DeviceConfig:
```rust
pub struct DeviceConfig {
    /// This device's identity
    pub identity: DeviceIdentity,
    
    /// List of trusted devices
    pub trusted_devices: Vec<TrustedDevice>,
    
    /// Bridge port for secure connections
    pub bridge_port: u16,
    
    /// Discovery port for UDP multicast
    pub discovery_port: u16,
}
```

---

## 🎯 User Actions

### 1. View Trusted Devices
**Steps:**
1. Open File Share Panel (📡 icon)
2. Click "My Devices" tab (📱 icon)
3. View list of all trusted devices

**What You See:**
- Device name and OS icon
- Trust date and last connection time
- Expired warning if applicable
- Rename and Remove buttons

### 2. Rename a Device
**Steps:**
1. Open My Devices panel
2. Click "✏️ Rename" button
3. Edit the device name
4. Click "✓" to save or "✕" to cancel

**Result:**
- Device name updated immediately
- Saved to file_share.json
- Visible across all UI components

### 3. Remove/Untrust a Device
**Steps:**
1. Open My Devices panel
2. Click "🗑️ Remove" button
3. Confirmation dialog appears
4. Click "Remove" to confirm or "Cancel" to abort

**Result:**
- Device removed from trusted list
- file_share.json updated
- Next connection requires manual approval
- No automatic reconnection

### 4. Reconnect After Untrust
**Steps:**
1. Device appears in discovery with "Connect" button
2. Click "Connect" button
3. Trust re-established automatically
4. Device added back to trusted list

---

## 🔍 Visual Indicators

### Device Status:
- **✓ Connected** - Green checkmark, device is trusted and online
- **Connect** - Orange button, device not trusted
- **⚠ Expired** - Orange badge, trust expired (30 days)
- **Offline** - Gray, device not responding

### Radar Display:
- **Green dot (inner circle)** - Trusted device (40px radius)
- **Orange dot (outer circle)** - Untrusted device (70px radius)

### Connection State:
- **Solid color** - Connected
- **Pulsing animation** - Connecting
- **Faded** - Disconnected

---

## 🧪 Testing Scenarios

### Scenario 1: First Connection
1. Start IGRIS on Device A and Device B
2. Device A discovers Device B
3. Click "Connect" on Device A
4. Verify trust established on both devices
5. Check file_share.json on both devices

### Scenario 2: Auto-Reconnect
1. Connect Device A to Device B (establish trust)
2. Close IGRIS on both devices
3. Restart IGRIS on both devices
4. Verify automatic connection without user prompt

### Scenario 3: Untrust and Reconnect
1. Connect Device A to Device B
2. Open My Devices on Device A
3. Remove Device B
4. Verify "Connect" button appears
5. Click "Connect" to re-establish trust

### Scenario 4: Trust Expiry
1. Connect Device A to Device B
2. Manually edit file_share.json
3. Set last_connected to 31 days ago
4. Restart IGRIS
5. Verify "⚠ Expired" badge appears

### Scenario 5: Rate Limiting
1. Attempt to connect to Device B
2. Fail connection 3 times
3. Verify 5-minute block is enforced
4. Wait 5 minutes
5. Verify connection allowed again

---

## 🐛 Troubleshooting

### Issue: Device Not Showing in My Devices
**Cause:** Device not trusted yet
**Solution:** Connect to device first to establish trust

### Issue: Trust Expired Warning
**Cause:** No connection for 30+ days
**Solution:** Reconnect to device to refresh trust

### Issue: Cannot Remove Device
**Cause:** File permission error
**Solution:** Check file_share.json permissions

### Issue: Device Auto-Connects After Removal
**Cause:** Trust not removed on remote device
**Solution:** Remove trust on both devices

### Issue: Rate Limited Error
**Cause:** 3+ failed connection attempts
**Solution:** Wait 5 minutes before retrying

---

## 📈 Future Enhancements

### Potential Features:
- [ ] Export/Import trusted devices
- [ ] Trust groups (home, work, etc.)
- [ ] Custom trust expiry periods
- [ ] Trust history/audit log
- [ ] Bulk device management
- [ ] Trust notifications
- [ ] Device notes/descriptions
- [ ] Last IP address tracking
- [ ] Connection statistics

---

## 🔑 Key Takeaways

1. **Automatic Trust** - Established during first connection
2. **Bidirectional** - Both devices trust each other
3. **Secure** - Certificate fingerprint verification
4. **Persistent** - Stored in JSON file
5. **Expiring** - 30-day timeout without connection
6. **Rate Limited** - 3 attempts, 5-minute block
7. **User Controlled** - Easy to view, rename, and remove
8. **Visual Feedback** - Clear UI indicators

---

## 📚 Related Documentation

- `README.md` - Main project documentation
- `FILE_SHARING_COMPLETE_GUIDE.md` - File sharing overview
- `ARCHITECTURE.md` - System architecture
- `CONNECTION_POOL_IMPLEMENTATION.md` - Connection management

---

**Last Updated:** January 27, 2026
**Version:** 1.0
**Status:** ✅ Fully Implemented
