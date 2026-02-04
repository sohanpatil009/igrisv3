// mDNS discovery implementation

use super::{Device, DeviceRegistry};
use crate::file_share::protocol::{AnnouncementMessage, DeviceInfo, RegisterMessage, MULTICAST_ADDR};
use anyhow::Result;
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

const MULTICAST_PORT: u16 = 53317;

/// mDNS discovery service
pub struct MdnsDiscovery {
    device_info: DeviceInfo,
    registry: Arc<RwLock<DeviceRegistry>>,
    broadcast_handle: Option<JoinHandle<()>>,
    listen_handle: Option<JoinHandle<()>>,
}

impl MdnsDiscovery {
    pub async fn new(
        device_name: String,
        port: u16,
        registry: Arc<RwLock<DeviceRegistry>>,
    ) -> Result<Self> {
        let fingerprint = generate_fingerprint();
        let device_info = DeviceInfo::new(device_name, fingerprint, port);

        Ok(Self {
            device_info,
            registry,
            broadcast_handle: None,
            listen_handle: None,
        })
    }

    /// Start broadcasting announcements
    pub async fn start_broadcasting(&mut self) -> Result<()> {
        let device_info = self.device_info.clone();
        
        let handle = tokio::spawn(async move {
            if let Err(e) = broadcast_loop(device_info).await {
                eprintln!("Broadcast error: {}", e);
            }
        });

        self.broadcast_handle = Some(handle);
        Ok(())
    }

    /// Start listening for announcements
    pub async fn start_listening(&mut self) -> Result<()> {
        let registry = self.registry.clone();
        let device_info = self.device_info.clone();

        let handle = tokio::spawn(async move {
            if let Err(e) = listen_loop(registry, device_info).await {
                eprintln!("Listen error: {}", e);
            }
        });

        self.listen_handle = Some(handle);
        Ok(())
    }

    /// Stop broadcasting
    pub async fn stop_broadcasting(&mut self) -> Result<()> {
        if let Some(handle) = self.broadcast_handle.take() {
            handle.abort();
        }
        Ok(())
    }

    /// Stop listening
    pub async fn stop_listening(&mut self) -> Result<()> {
        if let Some(handle) = self.listen_handle.take() {
            handle.abort();
        }
        Ok(())
    }

    /// Send announcement once
    pub async fn announce_once(&self) -> Result<()> {
        send_announcement(&self.device_info).await
    }
}

/// Broadcast loop - sends announcements periodically
async fn broadcast_loop(device_info: DeviceInfo) -> Result<()> {
    let local_ip = get_best_local_ip();
    println!("[mDNS] Starting broadcast loop for device: {} (IP: {})", device_info.alias, local_ip);
    
    loop {
        // Send multicast announcement
        match send_announcement(&device_info).await {
            Ok(_) => {}, // Success, no spam
            Err(e) => eprintln!("[mDNS] ⚠ Multicast failed: {}", e),
        }
        
        // Also send broadcast announcement (for mobile hotspots)
        match send_broadcast_announcement(&device_info).await {
            Ok(_) => {}, // Success, no spam
            Err(e) => eprintln!("[mDNS] ⚠ Broadcast failed: {}", e),
        }

        tokio::time::sleep(Duration::from_secs(5)).await; // Announce every 5 seconds
    }
}

/// Listen loop - receives announcements from other devices
async fn listen_loop(
    registry: Arc<RwLock<DeviceRegistry>>,
    our_device_info: DeviceInfo,
) -> Result<()> {
    let local_ip = get_best_local_ip();
    let local_ips = get_all_local_ips(); // Get all local IPs to filter self
    println!("[mDNS] Starting listen loop on port {} (Local IPs: {:?})", MULTICAST_PORT, local_ips);
    
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", MULTICAST_PORT))?;
    socket.set_read_timeout(Some(Duration::from_secs(1)))?;
    socket.set_broadcast(true)?;

    // Join multicast group on all interfaces
    let multicast_addr: IpAddr = MULTICAST_ADDR.parse()?;
    if let IpAddr::V4(addr) = multicast_addr {
        // Join on all interfaces (0.0.0.0)
        match socket.join_multicast_v4(&addr, &"0.0.0.0".parse().unwrap()) {
            Ok(_) => println!("[mDNS] ✓ Joined multicast group {}", MULTICAST_ADDR),
            Err(e) => println!("[mDNS] ⚠ Failed to join multicast: {} (will rely on broadcast)", e),
        }
    }

    let mut buf = [0u8; 4096];
    println!("[mDNS] Listening for announcements...");

    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, addr)) => {
                // Ignore packets from our own IP addresses
                let sender_ip = addr.ip().to_string();
                if local_ips.contains(&sender_ip) {
                    // Silently ignore self packets (no log spam)
                    continue;
                }
                
                println!("[mDNS] Received {} bytes from {}", len, addr);
                if let Ok(msg) = serde_json::from_slice::<AnnouncementMessage>(&buf[..len]) {
                    println!("[mDNS] Parsed announcement from: {} ({})", msg.alias, msg.fingerprint);
                    
                    // Double-check: Ignore self by fingerprint
                    if msg.fingerprint == our_device_info.fingerprint {
                        println!("[mDNS] Ignoring self announcement (same fingerprint)");
                        continue;
                    }

                    let device_info = DeviceInfo {
                        alias: msg.alias.clone(),
                        version: msg.version,
                        device_model: msg.device_model,
                        device_type: msg.device_type,
                        fingerprint: msg.fingerprint,
                        port: msg.port,
                        protocol: msg.protocol,
                        download: msg.download,
                    };

                    let device = Device::from_device_info(device_info, addr.ip());
                    println!("[mDNS] ✓ Discovered device: {} at {}:{}", msg.alias, addr.ip(), msg.port);
                    registry.write().await.add_device(device);

                    // Send response if announce is true
                    if msg.announce {
                        println!("[mDNS] Sending register response to {}", msg.alias);
                        if let Err(e) = send_register_response(&our_device_info, addr).await {
                            eprintln!("[mDNS] Failed to send register response: {}", e);
                        }
                    }
                } else {
                    println!("[mDNS] Failed to parse announcement (invalid JSON)");
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // Timeout is normal, just continue
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Err(e) => {
                // Only log error once per minute to avoid spam
                static LAST_ERROR_TIME: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let last = LAST_ERROR_TIME.load(std::sync::atomic::Ordering::Relaxed);
                
                if now - last > 60 {
                    eprintln!("[mDNS] Socket error (check firewall): {}", e);
                    LAST_ERROR_TIME.store(now, std::sync::atomic::Ordering::Relaxed);
                }
                
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }

        // Cleanup stale devices periodically
        registry.write().await.cleanup_stale();
    }
}

/// Send announcement via multicast
async fn send_announcement(device_info: &DeviceInfo) -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_multicast_loop_v4(true)?;
    socket.set_multicast_ttl_v4(255)?;
    
    let announcement = AnnouncementMessage {
        alias: device_info.alias.clone(),
        version: device_info.version.clone(),
        device_model: device_info.device_model.clone(),
        device_type: device_info.device_type.clone(),
        fingerprint: device_info.fingerprint.clone(),
        port: device_info.port,
        protocol: device_info.protocol.clone(),
        download: device_info.download,
        announce: true,
    };

    let data = serde_json::to_vec(&announcement)?;
    let addr: SocketAddr = format!("{}:{}", MULTICAST_ADDR, MULTICAST_PORT).parse()?;
    
    socket.send_to(&data, addr)?;
    Ok(())
}

/// Send broadcast announcement (for mobile hotspots and networks that block multicast)
async fn send_broadcast_announcement(device_info: &DeviceInfo) -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;
    
    let announcement = AnnouncementMessage {
        alias: device_info.alias.clone(),
        version: device_info.version.clone(),
        device_model: device_info.device_model.clone(),
        device_type: device_info.device_type.clone(),
        fingerprint: device_info.fingerprint.clone(),
        port: device_info.port,
        protocol: device_info.protocol.clone(),
        download: device_info.download,
        announce: true,
    };

    let data = serde_json::to_vec(&announcement)?;
    let broadcast_addr: SocketAddr = format!("255.255.255.255:{}", MULTICAST_PORT).parse()?;
    socket.send_to(&data, broadcast_addr)?;
    
    Ok(())
}

/// Send register response via multicast
async fn send_register_response(device_info: &DeviceInfo, target: SocketAddr) -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    
    let response = AnnouncementMessage {
        alias: device_info.alias.clone(),
        version: device_info.version.clone(),
        device_model: device_info.device_model.clone(),
        device_type: device_info.device_type.clone(),
        fingerprint: device_info.fingerprint.clone(),
        port: device_info.port,
        protocol: device_info.protocol.clone(),
        download: device_info.download,
        announce: false,
    };

    let data = serde_json::to_vec(&response)?;
    socket.send_to(&data, target)?;
    Ok(())
}

/// Generate a random fingerprint
fn generate_fingerprint() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", timestamp)
}

/// Get the best local IP address for multicast/broadcast
/// Prioritizes non-link-local, non-loopback IPv4 addresses
fn get_best_local_ip() -> String {
    use std::net::UdpSocket;
    
    // Try to get local IP by connecting to a public DNS (doesn't actually send data)
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                let ip = addr.ip().to_string();
                // Make sure it's not a link-local address (169.254.x.x)
                if !ip.starts_with("169.254.") && !ip.starts_with("127.") {
                    return ip;
                }
            }
        }
    }
    
    // Fallback: enumerate network interfaces
    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = std::process::Command::new("ipconfig").output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let mut found_wifi = false;
            
            for line in output_str.lines() {
                let line = line.trim();
                
                // Track if we're in WiFi adapter section
                if line.contains("adapter") {
                    found_wifi = line.to_lowercase().contains("wi-fi") || 
                                line.to_lowercase().contains("wireless");
                }
                
                // Look for IPv4 address on WiFi adapter
                if found_wifi && line.contains("IPv4 Address") {
                    if let Some(ip_part) = line.split(':').nth(1) {
                        let ip = ip_part.trim().trim_end_matches("(Preferred)").trim();
                        // Skip link-local addresses
                        if !ip.starts_with("169.254.") && !ip.starts_with("127.") {
                            return ip.to_string();
                        }
                    }
                }
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        // Try to get IP from en0 (WiFi) or en1 (Ethernet)
        for interface in &["en0", "en1"] {
            if let Ok(output) = std::process::Command::new("ifconfig")
                .arg(interface)
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    let line = line.trim();
                    if line.starts_with("inet ") && !line.contains("127.0.0.1") {
                        // Parse: "inet 10.11.81.121 netmask 0xffffff00 broadcast 10.11.81.255"
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let ip = parts[1];
                            if !ip.starts_with("169.254.") && !ip.starts_with("127.") {
                                return ip.to_string();
                            }
                        }
                    }
                }
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("ip")
            .args(&["addr", "show"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                let line = line.trim();
                if line.starts_with("inet ") && !line.contains("127.0.0.1") {
                    // Parse: "inet 192.168.1.100/24 brd 192.168.1.255 scope global dynamic"
                    if let Some(ip_part) = line.split_whitespace().nth(1) {
                        let ip = ip_part.split('/').next().unwrap_or("");
                        if !ip.starts_with("169.254.") && !ip.starts_with("127.") && !ip.is_empty() {
                            return ip.to_string();
                        }
                    }
                }
            }
        }
    }
    
    "0.0.0.0".to_string()
}

/// Get all local IP addresses (to filter out self packets)
fn get_all_local_ips() -> Vec<String> {
    let mut ips = Vec::new();
    
    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = std::process::Command::new("ipconfig").output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            for line in output_str.lines() {
                let line = line.trim();
                if line.contains("IPv4 Address") {
                    if let Some(ip_part) = line.split(':').nth(1) {
                        let ip = ip_part.trim().trim_end_matches("(Preferred)").trim();
                        if !ip.is_empty() {
                            ips.push(ip.to_string());
                        }
                    }
                }
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("ifconfig").output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            for line in output_str.lines() {
                let line = line.trim();
                if line.starts_with("inet ") {
                    // Parse: "inet 10.11.81.121 netmask 0xffffff00 broadcast 10.11.81.255"
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let ip = parts[1];
                        if !ip.is_empty() {
                            ips.push(ip.to_string());
                        }
                    }
                }
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("ip")
            .args(&["addr", "show"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            for line in output_str.lines() {
                let line = line.trim();
                if line.starts_with("inet ") {
                    // Parse: "inet 192.168.1.100/24 brd 192.168.1.255 scope global dynamic"
                    if let Some(ip_part) = line.split_whitespace().nth(1) {
                        let ip = ip_part.split('/').next().unwrap_or("");
                        if !ip.is_empty() {
                            ips.push(ip.to_string());
                        }
                    }
                }
            }
        }
    }
    
    ips
}
