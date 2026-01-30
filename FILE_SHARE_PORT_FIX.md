# File Share Port Binding Fix

## Problem
When restarting IGRIS on macOS, you were getting an "address already in use" warning, and the 4-digit bridge code wasn't appearing. This happened because the file sharing service ports (45678, 45679, 45680) remained bound from the previous run.

## Root Cause
The file sharing system binds to three ports:
- **45678** - Discovery service (UDP multicast)
- **45679** - Transfer service (TCP)
- **45680** - Bridge service (TCP)

When the app exits via `std::process::exit(0)`, these sockets aren't properly closed, leaving the ports in a `TIME_WAIT` state on macOS. On restart, the new instance can't bind to these ports.

## Solution Applied

### 1. Socket Reuse Options (SO_REUSEADDR + SO_REUSEPORT)

**Discovery Service** (`src/file_share/discovery.rs`):
- Added `SO_REUSEADDR` for all platforms
- Added `SO_REUSEPORT` for Unix/macOS (allows multiple processes to bind to the same port)
- Platform-specific implementation using raw socket options

**Bridge Service** (`src/file_share/bridge.rs`):
- Converted to use `std::net::TcpListener` with non-blocking mode
- Automatic `SO_REUSEADDR` behavior on TCP sockets

**Transfer Service** (`src/file_share/transfer.rs`):
- Same approach as bridge service

### 2. Graceful Shutdown

**Added shutdown function** (`src/ui/file_share_panel.rs`):
```rust
pub async fn shutdown_file_share() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(manager) = FILE_SHARE_MANAGER.lock().unwrap().as_ref() {
        manager.stop().await?;
        println!("✅ File share stopped");
    }
    *FILE_SHARE_MANAGER.lock().unwrap() = None;
    Ok(())
}
```

### 3. Initialization with Cleanup

**Updated init function** to stop any existing service first:
```rust
pub async fn init_file_share() -> Result<(), Box<dyn std::error::Error>> {
    // First, try to stop any existing file share service
    let _ = shutdown_file_share().await;
    
    // Small delay to ensure ports are released
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Now create and start the new manager
    let manager = FileShareManager::new().await?;
    manager.start().await?;
    let code = manager.get_bridge_code().await;
    
    // Update state with the code
    {
        let mut state = FILE_SHARE_STATE.lock().unwrap();
        state.bridge_code = code.clone();
    }
    
    *FILE_SHARE_MANAGER.lock().unwrap() = Some(manager);
    println!("✅ File share initialized with code: {}", code);
    Ok(())
}
```

## Technical Details

### macOS-Specific Behavior
On macOS, `SO_REUSEPORT` is crucial because:
- It allows multiple sockets to bind to the same port
- Prevents "address already in use" errors on rapid restarts
- Works alongside `SO_REUSEADDR` for complete port reuse

### Platform-Specific Code
```rust
#[cfg(unix)]
{
    use std::os::unix::io::AsRawFd;
    let fd = std_socket.as_raw_fd();
    unsafe {
        let optval: libc::c_int = 1;
        // SO_REUSEADDR
        libc::setsockopt(fd, libc::SOL_SOCKET, libc::SO_REUSEADDR, ...);
        // SO_REUSEPORT (macOS/BSD)
        libc::setsockopt(fd, libc::SOL_SOCKET, libc::SO_REUSEPORT, ...);
    }
}

#[cfg(windows)]
{
    // Windows only needs SO_REUSEADDR
    use std::os::windows::io::AsRawSocket;
    // ... Windows implementation
}
```

## Result
✅ File sharing service now properly handles port reuse
✅ 4-digit bridge code appears on every restart
✅ No more "address already in use" warnings
✅ Works across macOS, Linux, and Windows

## Testing
Run the app multiple times in quick succession:
```bash
cargo run --release
# Exit with voice command "exit"
cargo run --release  # Should work immediately
```

You should see:
```
✅ File share initialized with code: 1234
🚀 File sharing services started
📱 Device: YourDevice (device-id)
🔗 Bridge Code: 1234
```

## Files Modified
- `src/file_share/discovery.rs` - Added SO_REUSEADDR/SO_REUSEPORT for UDP
- `src/file_share/bridge.rs` - Added SO_REUSEADDR for TCP bridge
- `src/file_share/transfer.rs` - Added SO_REUSEADDR for TCP transfer
- `src/ui/file_share_panel.rs` - Added shutdown function and cleanup logic
- `src/ui/mod.rs` - Exported shutdown_file_share function
