use crate::fastswap::models::Device;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

const SCAN_PORT: u16 = 53317;

pub struct DiscoveryService {
    devices: Arc<RwLock<Vec<Device>>>,
}

impl DiscoveryService {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn scan_network(&self, local_ip: &str) -> Result<Vec<Device>> {
        tracing::info!("Starting network scan from {}", local_ip);
        
        let mut discovered = Vec::new();
        
        // Parse local IP to get subnet
        let parts: Vec<&str> = local_ip.split('.').collect();
        if parts.len() != 4 {
            return Ok(discovered);
        }
        
        let subnet = format!("{}.{}.{}", parts[0], parts[1], parts[2]);
        
        // Scan subnet (full range 1-254)
        let mut tasks = Vec::new();
        
        for i in 1..=254 {
            let ip = format!("{}.{}", subnet, i);
            if ip == local_ip {
                continue; // Skip self
            }
            
            let task = tokio::spawn(async move {
                Self::probe_device(&ip, SCAN_PORT).await
            });
            
            tasks.push(task);
            
            // Limit concurrent tasks to avoid overwhelming the system
            if tasks.len() >= 50 {
                // Wait for batch to complete
                for task in tasks.drain(..) {
                    if let Ok(Some(device)) = task.await {
                        discovered.push(device);
                    }
                }
            }
        }
        
        // Collect remaining results
        for task in tasks {
            if let Ok(Some(device)) = task.await {
                discovered.push(device);
            }
        }
        
        // Update stored devices
        let mut devices = self.devices.write().await;
        *devices = discovered.clone();
        
        tracing::info!("Scan complete. Found {} devices", discovered.len());
        Ok(discovered)
    }

    async fn probe_device(ip: &str, port: u16) -> Option<Device> {
        let url = format!("http://{}:{}/api/localsend/v2/info", ip, port);
        
        match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()
        {
            Ok(client) => {
                match client.get(&url).send().await {
                    Ok(response) => {
                        if response.status().is_success() {
                            if let Ok(device) = response.json::<Device>().await {
                                tracing::info!("Found device: {} at {}", device.alias, ip);
                                return Some(device);
                            }
                        }
                    }
                    Err(_) => {}
                }
            }
            Err(_) => {}
        }
        
        None
    }

    pub async fn get_devices(&self) -> Vec<Device> {
        self.devices.read().await.clone()
    }
}
