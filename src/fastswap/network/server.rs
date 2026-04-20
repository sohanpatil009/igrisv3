use crate::fastswap::models::*;
use axum::{
    body::Bytes,
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
pub struct ServerState {
    pub local_device: Device,
    pub sessions: Arc<RwLock<Vec<TransferState>>>,
}

pub fn create_router(state: ServerState) -> Router {
    Router::new()
        .route("/api/localsend/v2/info", get(info_handler))
        .route("/api/localsend/v2/register", post(register_handler))
        .route("/api/localsend/v2/prepare-upload", post(prepare_upload_handler))
        .route("/api/localsend/v2/confirm-upload", post(confirm_upload_handler))
        .route("/api/localsend/v2/upload", post(upload_handler))
        .layer(
            tower::ServiceBuilder::new()
                .layer(axum::extract::DefaultBodyLimit::max(10 * 1024 * 1024 * 1024)) // 10GB limit
        )
        .with_state(state)
}

async fn info_handler(State(state): State<ServerState>) -> Json<Device> {
    Json(state.local_device.clone())
}

async fn register_handler(
    State(state): State<ServerState>,
    Json(request): Json<RegisterRequest>,
) -> Json<RegisterResponse> {
    tracing::info!("Device registered: {}", request.alias);
    
    Json(RegisterResponse {
        alias: state.local_device.alias.clone(),
        version: "2.0".to_string(),
        device_model: state.local_device.device_model.clone(),
        device_type: state.local_device.device_type.clone(),
        fingerprint: state.local_device.id.clone(),
    })
}

async fn prepare_upload_handler(
    State(state): State<ServerState>,
    Json(request): Json<PrepareUploadRequest>,
) -> Result<Json<PrepareUploadResponse>, StatusCode> {
    tracing::info!("Preparing upload from {}", request.info.alias);
    tracing::info!("Files to receive: {}", request.files.len());
    
    let session_id = Uuid::new_v4().to_string();
    let mut file_responses = Vec::new();
    
    for file in &request.files {
        file_responses.push(FileResponse {
            id: file.id.clone(),
            token: Uuid::new_v4().to_string(),
        });
    }
    
    // Create pending transfer for UI approval
    let pending = crate::fastswap::PendingTransfer {
        session_id: session_id.clone(),
        sender_name: request.info.alias.clone(),
        sender_device: request.info.device_model.clone(),
        file_count: request.files.len(),
        total_size: request.files.iter().map(|f| f.size).sum(),
        files: request.files.iter().map(|f| f.file_name.clone()).collect(),
    };
    
    crate::fastswap::add_pending_transfer(pending).await;
    tracing::info!("Added pending transfer for approval: {}", session_id);
    
    // Store session
    let transfer_state = TransferState {
        session_id: session_id.clone(),
        files: request.files.iter().map(|f| FileTransfer {
            id: f.id.clone(),
            name: f.file_name.clone(),
            path: std::path::PathBuf::from(&f.file_name),
            size: f.size,
            transferred: 0,
            token: file_responses.iter()
                .find(|fr| fr.id == f.id)
                .map(|fr| fr.token.clone()),
        }).collect(),
        total_size: request.files.iter().map(|f| f.size).sum(),
        transferred: 0,
        status: TransferStatus::Preparing,
        confirmed: false,
    };
    
    state.sessions.write().await.push(transfer_state);
    
    Ok(Json(PrepareUploadResponse {
        session_id,
        files: file_responses,
    }))
}

async fn confirm_upload_handler(
    State(state): State<ServerState>,
    Json(request): Json<ConfirmUploadRequest>,
) -> Result<Json<ConfirmUploadResponse>, StatusCode> {
    tracing::info!("Confirming upload for session: {}", request.session_id);
    
    // Wait for user approval (poll every 500ms for up to 60 seconds)
    let max_wait_time = std::time::Duration::from_secs(60);
    let poll_interval = std::time::Duration::from_millis(500);
    let start_time = std::time::Instant::now();
    
    tracing::info!("Waiting for user approval...");
    
    loop {
        // Check if approved
        if crate::fastswap::is_transfer_approved(&request.session_id).await {
            tracing::info!("✅ Transfer approved by user");
            break;
        }
        
        // Check if denied (removed from pending without approval)
        let pending = crate::fastswap::get_pending_transfers().await;
        let still_pending = pending.iter().any(|t| t.session_id == request.session_id);
        
        if !still_pending && !crate::fastswap::is_transfer_approved(&request.session_id).await {
            tracing::warn!("❌ Transfer denied by user: {}", request.session_id);
            return Err(StatusCode::FORBIDDEN);
        }
        
        // Check timeout
        if start_time.elapsed() > max_wait_time {
            tracing::warn!("⏱️ Transfer approval timeout: {}", request.session_id);
            // Clean up pending transfer
            crate::fastswap::deny_transfer(&request.session_id).await;
            return Err(StatusCode::REQUEST_TIMEOUT);
        }
        
        // Wait before next check
        tokio::time::sleep(poll_interval).await;
    }
    
    let mut sessions = state.sessions.write().await;
    let session = sessions.iter_mut()
        .find(|s| s.session_id == request.session_id)
        .ok_or_else(|| {
            tracing::error!("Session not found: {}", request.session_id);
            StatusCode::NOT_FOUND
        })?;
    
    session.confirmed = true;
    session.status = TransferStatus::Transferring;
    
    tracing::info!("✅ Upload confirmed for session: {}", request.session_id);
    
    Ok(Json(ConfirmUploadResponse {
        status: "ready".to_string(),
    }))
}

#[derive(Deserialize)]
struct UploadQuery {
    #[serde(rename = "sessionId")]
    session_id: String,
    #[serde(rename = "fileId")]
    file_id: String,
    token: String,
}

async fn upload_handler(
    State(state): State<ServerState>,
    Query(query): Query<UploadQuery>,
    body: Bytes,  // Raw body, not multipart
) -> Result<StatusCode, StatusCode> {
    tracing::info!("Receiving file: {} ({} bytes)", query.file_id, body.len());
    
    // Get download directory
    let download_dir = dirs::download_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap());
    
    // Find the file in session
    let sessions = state.sessions.read().await;
    let session = sessions.iter()
        .find(|s| s.session_id == query.session_id)
        .ok_or_else(|| {
            tracing::error!("Session not found: {}", query.session_id);
            StatusCode::NOT_FOUND
        })?;
    
    // Check if session is confirmed
    if !session.confirmed {
        tracing::error!("Session not confirmed: {}", query.session_id);
        return Err(StatusCode::PRECONDITION_FAILED);
    }
    
    let file = session.files.iter()
        .find(|f| f.id == query.file_id)
        .ok_or_else(|| {
            tracing::error!("File not found in session: {}", query.file_id);
            StatusCode::NOT_FOUND
        })?;
    
    // Verify token
    if let Some(expected_token) = &file.token {
        if expected_token != &query.token {
            tracing::error!("Invalid token for file {}", query.file_id);
            return Err(StatusCode::UNAUTHORIZED);
        }
    }
    
    // Check if body size matches expected size
    if body.len() as u64 != file.size {
        tracing::warn!(
            "File size mismatch: expected {}, got {}",
            file.size,
            body.len()
        );
    }
    
    let file_path = download_dir.join(&file.name);
    tracing::info!("Saving file to: {:?}", file_path);
    
    // Write file
    tokio::fs::write(&file_path, &body).await.map_err(|e| {
        tracing::error!("Failed to write file: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    tracing::info!("✅ File saved successfully: {:?} ({} bytes)", file_path, body.len());
    Ok(StatusCode::OK)
}

pub async fn start_server(port: u16, local_device: Device) -> Result<u16, Box<dyn std::error::Error>> {
    let state = ServerState {
        local_device: local_device.clone(),
        sessions: Arc::new(RwLock::new(Vec::new())),
    };
    
    let app = create_router(state);
    
    // Try multiple ports if the default is in use
    for try_port in port..port+10 {
        let addr = format!("0.0.0.0:{}", try_port);
        
        match tokio::net::TcpListener::bind(&addr).await {
            Ok(listener) => {
                tracing::info!("✅ Server started successfully on port {}", try_port);
                tracing::info!("Device: {} ({})", local_device.alias, local_device.ip);
                
                tokio::spawn(async move {
                    if let Err(e) = axum::serve(listener, app).await {
                        tracing::error!("Server error: {}", e);
                    }
                });
                
                return Ok(try_port);
            }
            Err(_e) => {
                if try_port == port {
                    tracing::warn!("Port {} in use, trying alternatives...", try_port);
                }
                continue;
            }
        }
    }
    
    Err(format!("Could not bind to any port in range {}-{}", port, port+9).into())
}
