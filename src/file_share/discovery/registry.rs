// Device registry for managing discovered devices

use super::Device;
use std::collections::HashMap;

/// Registry of discovered devices
pub struct DeviceRegistry {
    devices: HashMap<String, Device>,
    timeout_secs: u64,
}

impl DeviceRegistry {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
            timeout_secs: 300, // 5 minutes
        }
    }

    /// Add or update a device
    pub fn add_device(&mut self, device: Device) {
        if let Some(existing) = self.devices.get_mut(&device.id) {
            existing.update_last_seen();
            existing.alias = device.alias;
            existing.device_type = device.device_type;
            existing.device_model = device.device_model;
        } else {
            self.devices.insert(device.id.clone(), device);
        }
    }

    /// Get a device by ID
    pub fn get_device(&self, id: &str) -> Option<Device> {
        self.devices.get(id).cloned()
    }

    /// Get device by alias (case-insensitive)
    pub fn get_device_by_alias(&self, alias: &str) -> Option<Device> {
        let alias_lower = alias.to_lowercase();
        self.devices
            .values()
            .find(|d| d.alias.to_lowercase() == alias_lower)
            .cloned()
    }

    /// Get all devices
    pub fn get_all_devices(&self) -> Vec<Device> {
        self.devices.values().cloned().collect()
    }

    /// Remove a device
    pub fn remove_device(&mut self, id: &str) {
        self.devices.remove(id);
    }

    /// Remove stale devices
    pub fn cleanup_stale(&mut self) {
        self.devices.retain(|_, device| !device.is_stale(self.timeout_secs));
    }

    /// Get device count
    pub fn count(&self) -> usize {
        self.devices.len()
    }

    /// Clear all devices
    pub fn clear(&mut self) {
        self.devices.clear();
    }
}

impl Default for DeviceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
