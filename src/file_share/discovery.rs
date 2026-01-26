// src/file_share/discovery.rs - LAN Device Discovery via UDP Multicast

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::broadcast;
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;

use super::config::{DeviceIdentity, OperatingSystem, load_config};

// Discovery constants
const MULTICAST_GROUP: Ipv4Addr = Ipv4Addr::new(239, 255, 45, 67);
const DISCOVERY_PORT: u16 = 45678;
const BROADCAST_INTERVAL_SECS: u64 = 10;
const OFFLINE_THRESHOLD_SECS: u64 = 30;
const MAGIC_BYTES: &[u8; 4] = b"IGRS";
const PROTOCOL_VERSION: u8 = 1;

/// Discovery message sent over UDP multicast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryMessage {
    pub magic: [u8; 4],
    pub version: u8,
    pub device_id: String,
    pub hostname: String,
    pub label: String,
    pub os: OperatingSystem,
    pub bridge_port: u16,
    pub code: Option<String>, // 4-digit pairing code (optional)
}

impl DiscoveryMessage {
    pub fn new(identity: &DeviceIdentity, bridge_port: u16) -> Self {
        DiscoveryMessage {
            magic: *MAGIC_BYTES,
            version: PROTOCOL_VERSION,
            device_id: identity.id.clone(),
            hostname: identity.hostname.clone(),
            label: identity.label.clone(),
            os: identity.os.clone(),
            bridge_port,
            code: None, // Code will be set separately
        }
    }
    
    pub fn with_code(mut self, code: Option<String>) -> Self {
        self.code = code;
        self
    }
    
    pub fn is_valid(&self) -> bool {
        self.magic == *MAGIC_BYTES && self.version == PROTOCOL_VERSION
    }
    
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| format!("Serialize error: {}", e))
    }
    
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(data).map_err(|e| format!("Deserialize error: {}", e))
    }
}

/// A discovered device on the network
#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    pub id: String,
    pub hostname: String,
    pub label: String,
    pub os: OperatingSystem,
    pub ip_address: IpAddr,
    pub bridge_port: u16,
    pub last_seen: Instant,
    pub is_trusted: bool,
    pub code: Option<String>, // Pairing code broadcasted by the device
}

impl DiscoveredDevice {
    pub fn is_online(&self) -> bool {
        self.last_seen.elapsed().as_secs() < OFFLINE_THRESHOLD_SECS
    }
    
    pub fn seconds_since_seen(&self) -> u64 {
        self.last_seen.elapsed().as_secs()
    }
}

/// Events emitted by the discovery service
#[derive(Debug, Clone)]
pub enum DiscoveryEvent {
    DeviceFound(DiscoveredDevice),
    DeviceUpdated(DiscoveredDevice),
    DeviceOffline(String), // device_id
}

/// Discovery service state
pub struct DiscoveryService {
    devices: Arc<Mutex<HashMap<String, DiscoveredDevice>>>,
    my_device_id: String,
    running: Arc<Mutex<bool>>,
    event_sender: broadcast::Sender<DiscoveryEvent>,
}

impl DiscoveryService {
    pub fn new() -> Result<Self, String> {
        let config = load_config()?;
        let (event_sender, _) = broadcast::channel(100);
        
        Ok(DiscoveryService {
            devices: Arc::new(Mutex::new(HashMap::new())),
            my_device_id: config.identity.id,
            running: Arc::new(Mutex::new(false)),
            event_sender,
        })
    }
    
    /// Subscribe to discovery events
    pub fn subscribe(&self) -> broadcast::Receiver<DiscoveryEvent> {
        self.event_sender.subscribe()
    }
    
    /// Get all discovered devices
    pub fn get_devices(&self) -> Vec<DiscoveredDevice> {
        let devices = self.devices.lock().unwrap();
        devices.values().cloned().collect()
    }
    
    /// Get only online devices
    pub fn get_online_devices(&self) -> Vec<DiscoveredDevice> {
        let devices = self.devices.lock().unwrap();
        devices.values()
            .filter(|d| d.is_online())
            .cloned()
            .collect()
    }
    
    /// Get a specific device by ID
    pub fn get_device(&self, device_id: &str) -> Option<DiscoveredDevice> {
        let devices = self.devices.lock().unwrap();
        devices.get(device_id).cloned()
    }
    
    /// Check if discovery is running
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }
    
    /// Start the discovery service
    pub async fn start(&self) -> Result<(), String> {
        {
            let mut running = self.running.lock().unwrap();
            if *running {
                return Ok(()); // Already running
            }
            *running = true;
        }
        
        println!("[Discovery] Starting LAN discovery service...");
        
        // Start broadcaster
        let devices_clone = self.devices.clone();
        let my_id = self.my_device_id.clone();
        let running_clone = self.running.clone();
        let sender_clone = self.event_sender.clone();
        
        tokio::spawn(async move {
            if let Err(e) = run_broadcaster(running_clone.clone()).await {
                println!("[Discovery] Broadcaster error: {}", e);
            }
        });
        
        // Start listener
        let devices_clone2 = self.devices.clone();
        let my_id2 = self.my_device_id.clone();
        let running_clone2 = self.running.clone();
        let sender_clone2 = self.event_sender.clone();
        
        tokio::spawn(async move {
            if let Err(e) = run_listener(devices_clone2, my_id2, running_clone2, sender_clone2).await {
                println!("[Discovery] Listener error: {}", e);
            }
        });
        
        // Start offline checker
        let devices_clone3 = self.devices.clone();
        let running_clone3 = self.running.clone();
        let sender_clone3 = self.event_sender.clone();
        
        tokio::spawn(async move {
            run_offline_checker(devices_clone3, running_clone3, sender_clone3).await;
        });
        
        println!("[Discovery] Service started on {}:{}", MULTICAST_GROUP, DISCOVERY_PORT);
        
        Ok(())
    }
    
    /// Stop the discovery service
    pub fn stop(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
        println!("[Discovery] Service stopped");
    }
    
    /// Manually add a device by IP address (for cross-subnet connections)
    /// This is useful when devices are on different networks and multicast doesn't work
    pub async fn add_manual_device(&self, ip_address: &str, bridge_port: u16) -> Result<DiscoveredDevice, String> {
        println!("[Discovery] Manually adding device at {}:{}", ip_address, bridge_port);
        
        // Parse IP address
        let ip: IpAddr = ip_address.parse()
            .map_err(|e| format!("Invalid IP address: {}", e))?;
        
        // Try to connect and get device info
        // For now, create a placeholder device - in real implementation,
        // we would query the device for its identity
        let device_id = format!("manual_{}", ip_address.replace(".", "_"));
        let device = DiscoveredDevice {
            id: device_id.clone(),
            hostname: format!("Device at {}", ip_address),
            label: format!("Manual Device ({})", ip_address),
            os: OperatingSystem::Unknown,
            ip_address: ip,
            bridge_port,
            last_seen: Instant::now(),
            is_trusted: false,
            code: None, // Manual devices don't have codes initially
        };
        
        // Add to devices list
        {
            let mut devices = self.devices.lock().unwrap();
            devices.insert(device_id.clone(), device.clone());
        }
        
        // Emit event
        let _ = self.event_sender.send(DiscoveryEvent::DeviceFound(device.clone()));
        
        println!("[Discovery] Manual device added: {} at {}", device.label, ip_address);
        Ok(device)
    }
    
    /// Remove a manually added device
    pub fn remove_device(&self, device_id: &str) -> Result<(), String> {
        let mut devices = self.devices.lock().unwrap();
        if devices.remove(device_id).is_some() {
            println!("[Discovery] Removed device: {}", device_id);
            Ok(())
        } else {
            Err(format!("Device not found: {}", device_id))
        }
    }
}

/// Broadcast our presence on the network
async fn run_broadcaster(running: Arc<Mutex<bool>>) -> Result<(), String> {
    let config = load_config()?;
    
    // Create socket with proper multicast settings
    use socket2::{Domain, Protocol, Socket, Type};
    
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
        .map_err(|e| format!("Failed to create broadcast socket: {}", e))?;
    
    // Bind to any available port for sending
    let addr: std::net::SocketAddr = "0.0.0.0:0".parse().unwrap();
    socket.bind(&addr.into())
        .map_err(|e| format!("Failed to bind broadcast socket: {}", e))?;
    
    // Enable multicast loopback (so we can see our own broadcasts for testing)
    socket.set_multicast_loop_v4(true)
        .map_err(|e| format!("Failed to set multicast loopback: {}", e))?;
    
    // Set multicast TTL to 1 (local network only)
    socket.set_multicast_ttl_v4(1)
        .map_err(|e| format!("Failed to set multicast TTL: {}", e))?;
    
    // Set non-blocking for tokio
    socket.set_nonblocking(true)
        .map_err(|e| format!("Failed to set non-blocking: {}", e))?;
    
    // Convert to tokio UdpSocket
    let std_socket: std::net::UdpSocket = socket.into();
    let socket = UdpSocket::from_std(std_socket)
        .map_err(|e| format!("Failed to convert to tokio socket: {}", e))?;
    
    let multicast_addr = std::net::SocketAddr::new(std::net::IpAddr::V4(MULTICAST_GROUP), DISCOVERY_PORT);
    
    println!("[Discovery] Broadcasting as: {} ({})", config.identity.label, &config.identity.id[..8]);
    println!("[Discovery] Multicast address: {}", multicast_addr);
    
    // Wait 5 seconds before generating code to allow device discovery to complete
    println!("[Discovery] Waiting 5 seconds before generating code...");
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    // Check if code already exists (UI might have generated it)
    let existing_code = super::relay::get_my_device_code(&config.identity.id);
    
    if existing_code.is_none() {
        // Generate initial code only if it doesn't exist
        let initial_code = super::relay::generate_my_code(
            config.identity.id.clone(),
            "0.0.0.0".to_string(),
            config.bridge_port,
            config.identity.hostname.clone(),
            config.identity.label.clone(),
            config.identity.os.clone(),
        )?;
        println!("[Discovery] Generated initial broadcast code: {}", initial_code);
    } else {
        println!("[Discovery] Using existing code: {}", existing_code.unwrap());
    }
    
    loop {
        {
            if !*running.lock().unwrap() {
                break;
            }
        }
        
        // Get current code from relay service (should always exist now)
        let my_code = super::relay::get_my_device_code(&config.identity.id);
        
        if my_code.is_none() {
            println!("[Discovery] WARNING: No code found for device {}, broadcasts will not include code", 
                &config.identity.id[..8]);
        }
        
        // Create message with current code
        let message = DiscoveryMessage::new(&config.identity, config.bridge_port)
            .with_code(my_code);
        let message_bytes = message.to_bytes()?;
        
        // Send discovery message
        match socket.send_to(&message_bytes, multicast_addr).await {
            Ok(bytes) => {
                println!("[Discovery] Sent {} bytes to multicast group", bytes);
            }
            Err(e) => {
                println!("[Discovery] Broadcast error: {}", e);
            }
        }
        
        tokio::time::sleep(Duration::from_secs(BROADCAST_INTERVAL_SECS)).await;
    }
    
    Ok(())
}

/// Listen for other devices on the network
async fn run_listener(
    devices: Arc<Mutex<HashMap<String, DiscoveredDevice>>>,
    my_device_id: String,
    running: Arc<Mutex<bool>>,
    event_sender: broadcast::Sender<DiscoveryEvent>,
) -> Result<(), String> {
    // Create socket and join multicast group
    let socket = create_multicast_socket().await?;
    
    let mut buf = [0u8; 2048];
    
    println!("[Discovery] Listening for devices...");
    
    loop {
        {
            if !*running.lock().unwrap() {
                break;
            }
        }
        
        // Set a timeout so we can check the running flag
        let recv_result = tokio::time::timeout(
            Duration::from_secs(5),
            socket.recv_from(&mut buf)
        ).await;
        
        match recv_result {
            Ok(Ok((len, src_addr))) => {
                // Parse the message
                if let Ok(message) = DiscoveryMessage::from_bytes(&buf[..len]) {
                    println!("[Discovery] Received from {}: device_id={}, my_id={}", 
                        src_addr.ip(), &message.device_id[..8.min(message.device_id.len())], &my_device_id[..8.min(my_device_id.len())]);
                    
                    if message.is_valid() && message.device_id != my_device_id {
                        // Check if trusted
                        let is_trusted = {
                            let config = load_config().unwrap_or_default();
                            config.is_trusted(&message.device_id)
                        };
                        
                        let device = DiscoveredDevice {
                            id: message.device_id.clone(),
                            hostname: message.hostname,
                            label: message.label,
                            os: message.os,
                            ip_address: src_addr.ip(),
                            bridge_port: message.bridge_port,
                            last_seen: Instant::now(),
                            is_trusted,
                            code: message.code, // Store the broadcasted code
                        };
                        
                        // Update device list
                        let event = {
                            let mut devices_lock = devices.lock().unwrap();
                            if devices_lock.contains_key(&device.id) {
                                devices_lock.insert(device.id.clone(), device.clone());
                                DiscoveryEvent::DeviceUpdated(device)
                            } else {
                                println!("[Discovery] Found device: {} ({}) at {}", 
                                    device.label, device.os.as_str(), src_addr.ip());
                                devices_lock.insert(device.id.clone(), device.clone());
                                DiscoveryEvent::DeviceFound(device)
                            }
                        };
                        
                        // Send event (ignore if no receivers)
                        let _ = event_sender.send(event);
                    } else if message.device_id == my_device_id {
                        println!("[Discovery] Ignoring own broadcast");
                    }
                }
            }
            Ok(Err(e)) => {
                println!("[Discovery] Receive error: {}", e);
            }
            Err(_) => {
                // Timeout - continue loop to check running flag
            }
        }
    }
    
    Ok(())
}

/// Check for offline devices periodically
async fn run_offline_checker(
    devices: Arc<Mutex<HashMap<String, DiscoveredDevice>>>,
    running: Arc<Mutex<bool>>,
    event_sender: broadcast::Sender<DiscoveryEvent>,
) {
    loop {
        {
            if !*running.lock().unwrap() {
                break;
            }
        }
        
        tokio::time::sleep(Duration::from_secs(10)).await;
        
        // Find offline devices
        let offline_ids: Vec<String> = {
            let devices_lock = devices.lock().unwrap();
            devices_lock.iter()
                .filter(|(_, d)| !d.is_online())
                .map(|(id, _)| id.clone())
                .collect()
        };
        
        // Remove and notify
        for id in offline_ids {
            {
                let mut devices_lock = devices.lock().unwrap();
                if let Some(device) = devices_lock.remove(&id) {
                    println!("[Discovery] Device offline: {}", device.label);
                }
            }
            let _ = event_sender.send(DiscoveryEvent::DeviceOffline(id));
        }
    }
}

/// Create a UDP socket configured for multicast
async fn create_multicast_socket() -> Result<UdpSocket, String> {
    use socket2::{Domain, Protocol, Socket, Type};
    
    // Create socket with socket2 for multicast options
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
        .map_err(|e| format!("Failed to create socket: {}", e))?;
    
    // Allow address reuse
    socket.set_reuse_address(true)
        .map_err(|e| format!("Failed to set reuse address: {}", e))?;
    
    #[cfg(unix)]
    socket.set_reuse_port(true).ok(); // Not available on all platforms
    
    // Bind to the multicast port on all interfaces
    let addr: std::net::SocketAddr = format!("0.0.0.0:{}", DISCOVERY_PORT).parse().unwrap();
    socket.bind(&addr.into())
        .map_err(|e| format!("Failed to bind to port {}: {}", DISCOVERY_PORT, e))?;
    
    println!("[Discovery] Socket bound to 0.0.0.0:{}", DISCOVERY_PORT);
    
    // Join multicast group on all available interfaces
    let joined = join_multicast_on_all_interfaces(&socket);
    if joined == 0 {
        println!("[Discovery] WARNING: Failed to join multicast on any interface, trying default...");
        // Fallback: join on default interface
        socket.join_multicast_v4(&MULTICAST_GROUP, &std::net::Ipv4Addr::UNSPECIFIED)
            .map_err(|e| format!("Failed to join multicast group: {}", e))?;
        println!("[Discovery] Joined multicast on default interface");
    } else {
        println!("[Discovery] Joined multicast on {} interface(s)", joined);
    }
    
    // Enable multicast loopback
    socket.set_multicast_loop_v4(true)
        .map_err(|e| format!("Failed to set multicast loopback: {}", e))?;
    
    // Set non-blocking for tokio
    socket.set_nonblocking(true)
        .map_err(|e| format!("Failed to set non-blocking: {}", e))?;
    
    // Convert to tokio UdpSocket
    let std_socket: std::net::UdpSocket = socket.into();
    UdpSocket::from_std(std_socket)
        .map_err(|e| format!("Failed to convert to tokio socket: {}", e))
}

/// Join multicast group on all available network interfaces
fn join_multicast_on_all_interfaces(socket: &socket2::Socket) -> usize {
    use std::net::Ipv4Addr;
    
    let mut joined_count = 0;
    
    // Get all network interfaces
    if let Ok(interfaces) = get_if_addrs::get_if_addrs() {
        for iface in interfaces {
            // Only process IPv4 non-loopback interfaces
            if let get_if_addrs::IfAddr::V4(ref addr) = iface.addr {
                if !addr.ip.is_loopback() {
                    match socket.join_multicast_v4(&MULTICAST_GROUP, &addr.ip) {
                        Ok(_) => {
                            println!("[Discovery] Joined multicast on {} ({})", iface.name, addr.ip);
                            joined_count += 1;
                        }
                        Err(e) => {
                            println!("[Discovery] Failed to join multicast on {} ({}): {}", 
                                iface.name, addr.ip, e);
                        }
                    }
                }
            }
        }
    }
    
    joined_count
}

// Global discovery service instance
static DISCOVERY_SERVICE: Lazy<Arc<Mutex<Option<DiscoveryService>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(None)));

/// Initialize and get the discovery service
pub fn get_discovery_service() -> Result<Arc<Mutex<Option<DiscoveryService>>>, String> {
    let mut service = DISCOVERY_SERVICE.lock().map_err(|e| format!("Lock error: {}", e))?;
    if service.is_none() {
        *service = Some(DiscoveryService::new()?);
    }
    Ok(DISCOVERY_SERVICE.clone())
}

/// Start discovery (convenience function)
pub async fn start_discovery() -> Result<(), String> {
    let service_lock = get_discovery_service()?;
    let service = service_lock.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(ref svc) = *service {
        svc.start().await
    } else {
        Err("Discovery service not initialized".to_string())
    }
}

/// Stop discovery (convenience function)
pub fn stop_discovery() -> Result<(), String> {
    let service_lock = get_discovery_service()?;
    let service = service_lock.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(ref svc) = *service {
        svc.stop();
    }
    Ok(())
}

/// Get discovered devices (convenience function)
pub fn get_discovered_devices() -> Result<Vec<DiscoveredDevice>, String> {
    let service_lock = get_discovery_service()?;
    let service = service_lock.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(ref svc) = *service {
        Ok(svc.get_online_devices())
    } else {
        Ok(Vec::new())
    }
}

/// Manually add a device by IP address (for cross-subnet connections)
pub async fn add_manual_device(ip_address: &str, bridge_port: u16) -> Result<DiscoveredDevice, String> {
    let service_lock = get_discovery_service()?;
    let service = service_lock.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    if let Some(ref svc) = *service {
        svc.add_manual_device(ip_address, bridge_port).await
    } else {
        Err("Discovery service not initialized".to_string())
    }
}

/// Remove a manually added device
pub fn remove_manual_device(device_id: &str) -> Result<(), String> {
    let service_lock = get_discovery_service()?;
    let service = service_lock.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    if let Some(ref svc) = *service {
        svc.remove_device(device_id)
    } else {
        Err("Discovery service not initialized".to_string())
    }
}
