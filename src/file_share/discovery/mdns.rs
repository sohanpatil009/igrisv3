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
    loop {
        if let Err(e) = send_announcement(&device_info).await {
            eprintln!("Failed to send announcement: {}", e);
        }

        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}

/// Listen loop - receives announcements from other devices
async fn listen_loop(
    registry: Arc<RwLock<DeviceRegistry>>,
    our_device_info: DeviceInfo,
) -> Result<()> {
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", MULTICAST_PORT))?;
    socket.set_read_timeout(Some(Duration::from_secs(1)))?;

    // Join multicast group
    let multicast_addr: IpAddr = MULTICAST_ADDR.parse()?;
    if let IpAddr::V4(addr) = multicast_addr {
        socket.join_multicast_v4(&addr, &"0.0.0.0".parse().unwrap())?;
    }

    let mut buf = [0u8; 4096];

    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, addr)) => {
                if let Ok(msg) = serde_json::from_slice::<AnnouncementMessage>(&buf[..len]) {
                    // Ignore self
                    if msg.fingerprint == our_device_info.fingerprint {
                        continue;
                    }

                    let device_info = DeviceInfo {
                        alias: msg.alias,
                        version: msg.version,
                        device_model: msg.device_model,
                        device_type: msg.device_type,
                        fingerprint: msg.fingerprint,
                        port: msg.port,
                        protocol: msg.protocol,
                        download: msg.download,
                    };

                    let device = Device::from_device_info(device_info, addr.ip());
                    registry.write().await.add_device(device);

                    // Send response if announce is true
                    if msg.announce {
                        if let Err(e) = send_register_response(&our_device_info, addr).await {
                            eprintln!("Failed to send register response: {}", e);
                        }
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Err(e) => {
                eprintln!("Socket error: {}", e);
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
