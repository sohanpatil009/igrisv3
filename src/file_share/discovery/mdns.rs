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
    println!("[mDNS] Starting broadcast loop for device: {}", device_info.alias);
    loop {
        match send_announcement(&device_info).await {
            Ok(_) => println!("[mDNS] Announcement sent: {}", device_info.alias),
            Err(e) => eprintln!("[mDNS] Failed to send announcement: {}", e),
        }

        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}

/// Listen loop - receives announcements from other devices
async fn listen_loop(
    registry: Arc<RwLock<DeviceRegistry>>,
    our_device_info: DeviceInfo,
) -> Result<()> {
    println!("[mDNS] Starting listen loop on port {}", MULTICAST_PORT);
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", MULTICAST_PORT))?;
    socket.set_read_timeout(Some(Duration::from_secs(1)))?;

    // Join multicast group
    let multicast_addr: IpAddr = MULTICAST_ADDR.parse()?;
    if let IpAddr::V4(addr) = multicast_addr {
        socket.join_multicast_v4(&addr, &"0.0.0.0".parse().unwrap())?;
        println!("[mDNS] Joined multicast group: {}", MULTICAST_ADDR);
    }

    let mut buf = [0u8; 4096];
    println!("[mDNS] Listening for announcements...");

    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, addr)) => {
                println!("[mDNS] Received {} bytes from {}", len, addr);
                if let Ok(msg) = serde_json::from_slice::<AnnouncementMessage>(&buf[..len]) {
                    println!("[mDNS] Parsed announcement from: {} ({})", msg.alias, msg.fingerprint);
                    
                    // Ignore self
                    if msg.fingerprint == our_device_info.fingerprint {
                        println!("[mDNS] Ignoring self announcement");
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
    
    println!("[mDNS] Sending announcement to {}:{} ({} bytes)", MULTICAST_ADDR, MULTICAST_PORT, data.len());
    socket.send_to(&data, addr)?;
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
