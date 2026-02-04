// Connection manager

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ConnectionManager {
    active_connections: Arc<RwLock<HashMap<String, ConnectionInfo>>>,
}

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub device_id: String,
    pub connected_at: u64,
    pub last_activity: u64,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            active_connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_connection(&self, device_id: String) {
        let now = current_timestamp();
        let info = ConnectionInfo {
            device_id: device_id.clone(),
            connected_at: now,
            last_activity: now,
        };
        self.active_connections.write().await.insert(device_id, info);
    }

    pub async fn remove_connection(&self, device_id: &str) {
        self.active_connections.write().await.remove(device_id);
    }

    pub async fn update_activity(&self, device_id: &str) {
        if let Some(conn) = self.active_connections.write().await.get_mut(device_id) {
            conn.last_activity = current_timestamp();
        }
    }

    pub async fn get_active_count(&self) -> usize {
        self.active_connections.read().await.len()
    }

    pub async fn cleanup_stale(&self, timeout_secs: u64) {
        let now = current_timestamp();
        self.active_connections
            .write()
            .await
            .retain(|_, conn| now - conn.last_activity < timeout_secs);
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
