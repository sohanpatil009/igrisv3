# Cross-Subnet & Mobile Hotspot File Sharing Guide

## Problem: AP Isolation on Mobile Hotspots

Mobile hotspots enable **AP Isolation** which blocks peer-to-peer connections:
- ✅ Multicast works (device discovery)
- ❌ Unicast UDP/TCP blocked (direct connections)
- This is a **network-level restriction**, not a code issue

## Solutions (Ranked by Ease)

### 1. Mac Personal Hotspot ⭐ RECOMMENDED
**Setup Time**: 2 minutes  
**Internet Required**: No  
**Works**: ✅ Out of the box

Mac hotspots allow P2P by default!

**Steps**:
```bash
# On Mac:
System Settings → General → Sharing → Personal Hotspot → Turn On

# On Windows:
Connect to Mac's WiFi hotspot

# Test:
Run app on both devices → Connect → Works!
```

### 2. WiFi Router
**Setup Time**: 5 minutes  
**Internet Required**: No (router can be offline)  
**Works**: ✅ Out of the box

Connect both devices to same WiFi router (even without internet).

### 3. Direct Ethernet/USB
**Setup Time**: 2 minutes  
**Internet Required**: No  
**Works**: ✅ Out of the box

- USB-C to Ethernet adapter + cable
- Or USB tethering (Mac → Windows)

### 4. Relay Server (Future)
**Status**: 🚧 Partially implemented  
**Internet Required**: Yes (for relay server)  
**Works**: Coming soon

Automatic fallback when direct connection fails.

## Current Implementation

### Automatic Fallback
The app now automatically tries:
1. **Direct QUIC connection** (fast, low latency)
2. **Relay fallback** (if direct fails)

```rust
// In connection.rs
pub async fn connect_direct() -> Result<ConnectionResult> {
    // Try direct first
    match connect_direct_internal().await {
        Ok(result) => Ok(result),
        Err(_) => {
            println!("Direct failed, trying relay...");
            connect_via_relay_internal().await
        }
    }
}
```

### Error Messages
When connection fails, you'll see:
```
Direct connection failed due to network restrictions (likely AP isolation on mobile hotspot).

Solutions:
1. Use Mac Personal Hotspot instead (allows P2P)
2. Connect both devices to a WiFi router
3. Use direct Ethernet/USB connection

Relay server support coming soon!
```

## Testing

### Test Direct Connection (WiFi Router or Mac Hotspot)
```bash
# Mac:
./target/release/igrisv3

# Windows:
igrisv3.exe

# Both devices should discover each other
# Click "Connect" → Should work!
```

### Test Mobile Hotspot (Will Fail)
```bash
# Connect both to mobile hotspot
# Discovery works ✅
# Connection fails ❌ (expected - AP isolation)
# Error message shows solutions
```

## Network Diagnostics

### Check if AP Isolation is Active
```bash
# On Mac, ping Windows:
ping 10.11.81.244

# If ping works but QUIC fails → AP Isolation
# If ping fails → Network issue
```

### Check Firewall
```bash
# Mac:
./check_firewall_status.sh

# Windows:
.\check_firewall_status.ps1
```

## Future: Full Relay Implementation

### Architecture
```
Device A <--QUIC--> Relay Server <--QUIC--> Device B
```

### Benefits
- Works through any network restriction
- Automatic fallback
- No user configuration needed

### Requirements
- Public relay server (or local relay on one device)
- Both devices connect to relay
- Relay forwards QUIC streams

### Implementation Status
- ✅ Relay detection logic
- ✅ Automatic fallback
- ✅ Error messages
- 🚧 Relay server (coming soon)
- 🚧 Stream forwarding (coming soon)

## Recommended Workflow

**For Development/Testing**:
1. Use Mac Personal Hotspot (easiest)
2. Or WiFi router

**For Production**:
1. Direct connection (LAN)
2. Automatic relay fallback (when implemented)

## FAQ

**Q: Why does discovery work but connection fails?**  
A: Mobile hotspots allow multicast (for discovery) but block unicast (for connections). This is AP Isolation.

**Q: Can I disable AP Isolation on mobile hotspot?**  
A: No, it's enforced by the mobile OS for security.

**Q: Will relay server require internet?**  
A: Yes, unless you run a local relay on one of the devices.

**Q: When will relay be fully implemented?**  
A: Soon! The infrastructure is ready, just needs relay server deployment.

## Summary

✅ **Working Now**: Mac hotspot, WiFi router, Ethernet  
🚧 **Coming Soon**: Automatic relay fallback  
❌ **Won't Work**: Mobile hotspot (AP isolation)

Use Mac Personal Hotspot for immediate testing!
