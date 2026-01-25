# Mac Multicast Discovery Fix

## Problem
Mac can't discover Windows device because multicast packets are not being received.

## Diagnosis

Run the app on Mac and check logs for:
```
[Discovery] Joined multicast on en0 (10.106.46.121)
[Discovery] Listening for devices...
```

If you see:
```
[Discovery] Failed to join multicast on en0: Permission denied
```

Then it's a permission issue.

## Solutions

### Solution 1: Disable Mac Firewall (Temporary)
```bash
# In Terminal:
sudo pfctl -d
```

Then restart the app.

### Solution 2: Allow App Through Firewall
1. System Preferences → Security & Privacy → Firewall
2. Click "Firewall Options"
3. Click "+" and add your app
4. Set to "Allow incoming connections"

### Solution 3: Check Network Interface
```bash
# List all interfaces
ifconfig

# Check if multicast route exists
netstat -rn | grep 224.0.0

# Add multicast route if missing
sudo route add -net 239.255.45.0/24 -interface en0
```

### Solution 4: Test Multicast Manually
```bash
# Terminal 1 (Listener)
nc -u -l 45678

# Terminal 2 (Sender - from Windows)
echo "test" | nc -u 239.255.45.67 45678
```

If Terminal 1 receives "test", multicast is working.

## Code-Level Fix

If none of the above work, we can add Mac-specific multicast options:

```rust
#[cfg(target_os = "macos")]
{
    // Mac needs SO_REUSEPORT
    socket.set_reuse_port(true)?;
    
    // Explicitly set multicast interface
    socket.set_multicast_if_v4(&local_addr)?;
}
```

## Workaround: Direct Connection

Until multicast works, use direct IP connection:

**On Mac:**
1. Note Windows IP: `10.106.46.244`
2. Note Windows code: `5101`
3. Enter code manually: `5101`
4. Connect

The connection will work even without discovery!

## Testing

After applying fix:

**On Mac, check logs:**
```
[Discovery] Joined multicast on en0 (10.106.46.121) ✅
[Discovery] Listening for devices... ✅
[Discovery] Received from 10.106.46.244 ✅
[Discovery] Found device: SOHAN-PATIL911 (Windows) ✅
```

## Common Mac Issues

1. **Little Snitch** - Blocks multicast by default
2. **VPN** - Can interfere with local multicast
3. **Multiple Networks** - Mac might be on different subnet
4. **WiFi vs Ethernet** - Make sure both devices on same network type

## Quick Check

**Same subnet?**
```
Windows: 10.106.46.244
Mac:     10.106.46.121
         ^^^^^^^^^^^ Should match!
```

If first 3 numbers don't match, devices are on different subnets!
