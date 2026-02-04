// Transfer approval system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub session_id: String,
    pub from_device: String,
    pub from_alias: String,
    pub files: Vec<String>,
    pub total_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalResponse {
    Accepted,
    Rejected,
    Pending,
}

pub struct ApprovalManager {
    pending_requests: Arc<RwLock<HashMap<String, ApprovalRequest>>>,
    responses: Arc<RwLock<HashMap<String, ApprovalResponse>>>,
}

impl ApprovalManager {
    pub fn new() -> Self {
        Self {
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            responses: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a pending approval request
    pub async fn add_request(&self, request: ApprovalRequest) {
        let session_id = request.session_id.clone();
        self.pending_requests.write().await.insert(session_id.clone(), request);
        self.responses.write().await.insert(session_id, ApprovalResponse::Pending);
    }

    /// Get pending requests
    pub async fn get_pending_requests(&self) -> Vec<ApprovalRequest> {
        self.pending_requests.read().await.values().cloned().collect()
    }

    /// Approve a request
    pub async fn approve(&self, session_id: &str) {
        self.responses.write().await.insert(session_id.to_string(), ApprovalResponse::Accepted);
        self.pending_requests.write().await.remove(session_id);
    }

    /// Reject a request
    pub async fn reject(&self, session_id: &str) {
        self.responses.write().await.insert(session_id.to_string(), ApprovalResponse::Rejected);
        self.pending_requests.write().await.remove(session_id);
    }

    /// Get response for a session
    pub async fn get_response(&self, session_id: &str) -> Option<ApprovalResponse> {
        self.responses.read().await.get(session_id).cloned()
    }

    /// Clear old responses
    pub async fn cleanup(&self) {
        // Keep only last 100 responses
        let mut responses = self.responses.write().await;
        if responses.len() > 100 {
            responses.clear();
        }
    }
}

impl Default for ApprovalManager {
    fn default() -> Self {
        Self::new()
    }
}
