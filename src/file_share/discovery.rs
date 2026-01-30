// src/file_share/discovery.rs
// Device discovery service using multicast UDP

use super::*;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{interval, Duration};
use serde::{Serialize, Deserialize};

/// Discovery service for finding devices on local network
pub struct DiscoveryService {
    device_info: DeviceInfo,
    devices: Arc<RwLock<HashMap<String, DeviceInfo>>>,
    socket: Option<Arc<UdpSocket>>,
    event_tx: mpsc::UnboundedSender<FileShareEvent>,
    running: Arc<RwLock<bool>>,
    config: FileShareConfig,
}

/// Discovery message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMessage {
    Announce(DeviceInfo),
    Query,
    Response(DeviceInfo),
    Goodbye(String), // device_id
}

impl DiscoveryService {
    /// Create new discovery service
    pub async fn new(
        device_info: DeviceInfo,
        event_tx: mpsc::UnboundedSender<FileShareEvent>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            device_info,
            devices: Arc::new(RwLock::new(HashMap::new())),
            socket: None,
            event_tx,
            running: Arc::new(RwLock::new(false)),
            config: FileShareConfig::default(),
        })
    }

    /// Start discovery service
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }

        // Bind to multicast address
        let multicast_addr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(224, 0, 0, 251)), // mDNS multicast
            self.config.discovery_port,
        );

        // Create socket with SO_REUSEADDR/SO_REUSEPORT BEFORE binding
        let socket = {
            #[cfg(unix)]
            {
                use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd};
                use std::net::UdpSocket as StdUdpSocket;
                
                // Create unbound socket
                let domain = libc::AF_INET;
                let socket_type = libc::SOCK_DGRAM;
                let protocol = 0;
                
                let fd = unsafe { libc::socket(domain, socket_type, protocol) };
                if fd < 0 {
                    return Err("Failed to create socket".into());
                }
                
                unsafe {
                    let optval: libc::c_int = 1;
                    
                    // SO_REUSEADDR - must be set before bind
                    if libc::setsockopt(
                        fd,
                        libc::SOL_SOCKET,
                        libc::SO_REUSEADDR,
                        &optval as *const _ as *const libc::c_void,
                        std::mem::size_of_val(&optval) as libc::socklen_t,
                    ) < 0 {
                        libc::close(fd);
                        return Err("Failed to set SO_REUSEADDR".into());
                    }
                    
                    // SO_REUSEPORT - must be set before bind (macOS/BSD)
                    if libc::setsockopt(
                        fd,
                        libc::SOL_SOCKET,
                        libc::SO_REUSEPORT,
                        &optval as *const _ as *const libc::c_void,
                        std::mem::size_of_val(&optval) as libc::socklen_t,
                    ) < 0 {
                        libc::close(fd);
                        return Err("Failed to set SO_REUSEPORT".into());
                    }
                    
                    // Now bind the socket
                    let addr: SocketAddr = format!("0.0.0.0:{}", self.config.discovery_port).parse()?;
                    let (addr_ptr, addr_len) = match addr {
                        SocketAddr::V4(addr) => {
                            // BSD-based systems (macOS, iOS, FreeBSD) have sin_len field
                            #[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
                            let sin = libc::sockaddr_in {
                                sin_len: std::mem::size_of::<libc::sockaddr_in>() as u8,
                                sin_family: libc::AF_INET as _,
                                sin_port: addr.port().to_be(),
                                sin_addr: libc::in_addr {
                                    s_addr: u32::from_ne_bytes(addr.ip().octets()),
                                },
                                sin_zero: [0; 8],
                            };
                            // Linux and other Unix systems don't have sin_len
                            #[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd")))]
                            let sin = libc::sockaddr_in {
                                sin_family: libc::AF_INET as _,
                                sin_port: addr.port().to_be(),
                                sin_addr: libc::in_addr {
                                    s_addr: u32::from_ne_bytes(addr.ip().octets()),
                                },
                                sin_zero: [0; 8],
                            };
                            (
                                &sin as *const _ as *const libc::sockaddr,
                                std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t,
                            )
                        }
                        _ => {
                            libc::close(fd);
                            return Err("IPv6 not supported".into());
                        }
                    };
                    
                    if libc::bind(fd, addr_ptr, addr_len) < 0 {
                        libc::close(fd);
                        return Err(format!("Failed to bind to port {} (error 48: address already in use)", self.config.discovery_port).into());
                    }
                    
                    // Set non-blocking
                    let flags = libc::fcntl(fd, libc::F_GETFL, 0);
                    libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
                    
                    // Convert to std socket
                    let std_socket = StdUdpSocket::from_raw_fd(fd);
                    UdpSocket::from_std(std_socket)?
                }
            }
            #[cfg(windows)]
            {
                use std::os::windows::io::{AsRawSocket, FromRawSocket};
                let std_socket = std::net::UdpSocket::bind(format!("0.0.0.0:{}", self.config.discovery_port))?;
                let socket = std_socket.as_raw_socket();
                unsafe {
                    let optval: i32 = 1;
                    winapi::um::winsock2::setsockopt(
                        socket as _,
                        winapi::um::winsock2::SOL_SOCKET,
                        winapi::um::winsock2::SO_REUSEADDR,
                        &optval as *const _ as *const i8,
                        std::mem::size_of_val(&optval) as i32,
                    );
                }
                std_socket.set_nonblocking(true)?;
                UdpSocket::from_std(std_socket)?
            }
        };
        
        // Join multicast group
        socket.join_multicast_v4(
            Ipv4Addr::new(224, 0, 0, 251),
            Ipv4Addr::new(0, 0, 0, 0),
        )?;

        let socket = Arc::new(socket);
        
        // Start announcement task
        let announce_socket = socket.clone();
        let announce_device = self.device_info.clone();
        let announce_running = self.running.clone();
        tokio::spawn(async move {
            Self::announcement_task(announce_socket, announce_device, announce_running, multicast_addr).await;
        });

        // Start listening task
        let listen_socket = socket.clone();
        let listen_devices = self.devices.clone();
        let listen_event_tx = self.event_tx.clone();
        let listen_running = self.running.clone();
        let listen_device_info = self.device_info.clone();
        tokio::spawn(async move {
            Self::listening_task(
                listen_socket,
                listen_devices,
                listen_event_tx,
                listen_running,
                listen_device_info,
                multicast_addr,
            ).await;
        });

        // Start cleanup task
        let cleanup_devices = self.devices.clone();
        let cleanup_event_tx = self.event_tx.clone();
        let cleanup_running = self.running.clone();
        tokio::spawn(async move {
            Self::cleanup_task(cleanup_devices, cleanup_event_tx, cleanup_running).await;
        });

        *running = true;
        println!("🔍 Discovery service started on port {}", self.config.discovery_port);
        
        Ok(())
    }

    /// Stop discovery service
    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut running = self.running.write().await;
        if !*running {
            return Ok(());
        }

        // Send goodbye message
        if let Some(socket) = &self.socket {
            let goodbye = DiscoveryMessage::Goodbye(self.device_info.id.clone());
            let message = serde_json::to_vec(&goodbye)?;
            let multicast_addr = SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(224, 0, 0, 251)),
                self.config.discovery_port,
            );
            let _ = socket.send_to(&message, multicast_addr).await;
        }

        *running = false;
        println!("🛑 Discovery service stopped");
        
        Ok(())
    }

    /// Get all discovered devices
    pub async fn get_devices(&self) -> Vec<DeviceInfo> {
        self.devices.read().await.values().cloned().collect()
    }

    /// Get specific device by ID
    pub async fn get_device(&self, device_id: &str) -> Option<DeviceInfo> {
        self.devices.read().await.get(device_id).cloned()
    }

    /// Announcement task - periodically announce presence
    async fn announcement_task(
        socket: Arc<UdpSocket>,
        mut device_info: DeviceInfo,
        running: Arc<RwLock<bool>>,
        multicast_addr: SocketAddr,
    ) {
        let mut interval = interval(Duration::from_secs(15)); // Announce every 15 seconds

        while *running.read().await {
            interval.tick().await;

            // Update timestamp
            device_info.update_last_seen();

            // Send announcement
            let announce = DiscoveryMessage::Announce(device_info.clone());
            if let Ok(message) = serde_json::to_vec(&announce) {
                if let Err(e) = socket.send_to(&message, multicast_addr).await {
                    eprintln!("Failed to send announcement: {}", e);
                }
            }
        }
    }

    /// Listening task - handle incoming messages
    async fn listening_task(
        socket: Arc<UdpSocket>,
        devices: Arc<RwLock<HashMap<String, DeviceInfo>>>,
        event_tx: mpsc::UnboundedSender<FileShareEvent>,
        running: Arc<RwLock<bool>>,
        own_device: DeviceInfo,
        multicast_addr: SocketAddr,
    ) {
        let mut buffer = [0u8; 4096];

        while *running.read().await {
            match socket.recv_from(&mut buffer).await {
                Ok((len, addr)) => {
                    if let Ok(message) = serde_json::from_slice::<DiscoveryMessage>(&buffer[..len]) {
                        match message {
                            DiscoveryMessage::Announce(device) => {
                                // Ignore our own announcements
                                if device.id == own_device.id {
                                    continue;
                                }

                                let mut devices_lock = devices.write().await;
                                let is_new = !devices_lock.contains_key(&device.id);
                                
                                devices_lock.insert(device.id.clone(), device.clone());
                                
                                if is_new {
                                    let _ = event_tx.send(FileShareEvent::DeviceDiscovered(device));
                                }
                            }
                            DiscoveryMessage::Query => {
                                // Respond to query with our device info
                                let response = DiscoveryMessage::Response(own_device.clone());
                                if let Ok(response_data) = serde_json::to_vec(&response) {
                                    let _ = socket.send_to(&response_data, addr).await;
                                }
                            }
                            DiscoveryMessage::Response(device) => {
                                // Handle response to our query
                                if device.id != own_device.id {
                                    let mut devices_lock = devices.write().await;
                                    let is_new = !devices_lock.contains_key(&device.id);
                                    
                                    devices_lock.insert(device.id.clone(), device.clone());
                                    
                                    if is_new {
                                        let _ = event_tx.send(FileShareEvent::DeviceDiscovered(device));
                                    }
                                }
                            }
                            DiscoveryMessage::Goodbye(device_id) => {
                                // Remove device
                                let mut devices_lock = devices.write().await;
                                if devices_lock.remove(&device_id).is_some() {
                                    let _ = event_tx.send(FileShareEvent::DeviceLost(device_id));
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Discovery listen error: {}", e);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }

    /// Cleanup task - remove stale devices
    async fn cleanup_task(
        devices: Arc<RwLock<HashMap<String, DeviceInfo>>>,
        event_tx: mpsc::UnboundedSender<FileShareEvent>,
        running: Arc<RwLock<bool>>,
    ) {
        let mut interval = interval(Duration::from_secs(30)); // Cleanup every 30 seconds

        while *running.read().await {
            interval.tick().await;

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let mut devices_lock = devices.write().await;
            let mut to_remove = Vec::new();

            for (id, device) in devices_lock.iter() {
                // Remove devices not seen for 60 seconds
                if now - device.last_seen > 60 {
                    to_remove.push(id.clone());
                }
            }

            for id in to_remove {
                devices_lock.remove(&id);
                let _ = event_tx.send(FileShareEvent::DeviceLost(id));
            }
        }
    }

    /// Send query to discover devices immediately
    pub async fn query_devices(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(socket) = &self.socket {
            let query = DiscoveryMessage::Query;
            let message = serde_json::to_vec(&query)?;
            let multicast_addr = SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(224, 0, 0, 251)),
                self.config.discovery_port,
            );
            socket.send_to(&message, multicast_addr).await?;
        }
        Ok(())
    }

    /// Get device count
    pub async fn device_count(&self) -> usize {
        self.devices.read().await.len()
    }

    /// Check if device is online
    pub async fn is_device_online(&self, device_id: &str) -> bool {
        if let Some(device) = self.devices.read().await.get(device_id) {
            device.is_online()
        } else {
            false
        }
    }
}