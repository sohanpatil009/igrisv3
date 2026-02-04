// REST API implementation

pub mod commands;
pub mod events;

pub use commands::FileShareCommand;
pub use events::FileShareEvent;

use crate::file_share::transfer::TransferOrchestrator;
use crate::file_share::protocol::{DeviceInfo, InfoResponse, PrepareUploadRequest, PrepareUploadResponse};
use crate::file_share::crypto::TlsConfig;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use serde::Deserialize;
use std::sync::Arc;
use tokio::task::JoinHandle;

#[derive(Clone)]
pub struct FileShareApi {
    device_info: DeviceInfo,
    orchestrator: Arc<TransferOrchestrator>,
    port: u16,
    tls_config: Option<Arc<TlsConfig>>,
    server_handle: Option<Arc<JoinHandle<()>>>,
}

impl FileShareApi {
    pub async fn new(port: u16, orchestrator: Arc<TransferOrchestrator>) -> anyhow::Result<Self> {
        let device_info = DeviceInfo::new("IGRIS".to_string(), "temp".to_string(), port);

        Ok(Self {
            device_info,
            orchestrator,
            port,
            tls_config: None,
            server_handle: None,
        })
    }

    /// Enable HTTPS with TLS
    pub fn with_tls(mut self, tls_config: TlsConfig) -> Self {
        self.tls_config = Some(Arc::new(tls_config));
        self
    }

    /// Start the HTTP/HTTPS server
    pub async fn start_server(&mut self) -> anyhow::Result<()> {
        let app_state = AppState {
            device_info: self.device_info.clone(),
            orchestrator: self.orchestrator.clone(),
        };

        let app = Router::new()
            .route("/api/localsend/v2/info", get(info_handler))
            .route("/api/localsend/v2/register", post(register_handler))
            .route("/api/localsend/v2/prepare-upload", post(prepare_upload_handler))
            .route("/api/localsend/v2/upload", post(upload_handler))
            .route("/api/localsend/v2/cancel", post(cancel_handler))
            .with_state(app_state);

        let addr = format!("0.0.0.0:{}", self.port);

        // Start HTTPS or HTTP server
        if let Some(tls_config) = &self.tls_config {
            // HTTPS server
            let rustls_config = RustlsConfig::from_config(tls_config.server_config.clone());
            
            let handle = tokio::spawn(async move {
                if let Err(e) = axum_server::bind_rustls(addr.parse::<std::net::SocketAddr>().unwrap(), rustls_config)
                    .serve(app.into_make_service())
                    .await
                {
                    eprintln!("HTTPS Server error: {}", e);
                }
            });

            self.server_handle = Some(Arc::new(handle));
        } else {
            // HTTP server (fallback)
            let listener = tokio::net::TcpListener::bind(&addr).await?;

            let handle = tokio::spawn(async move {
                if let Err(e) = axum::serve(listener, app).await {
                    eprintln!("HTTP Server error: {}", e);
                }
            });

            self.server_handle = Some(Arc::new(handle));
        }

        Ok(())
    }

    /// Stop the HTTP server
    pub async fn stop_server(&mut self) -> anyhow::Result<()> {
        if let Some(handle) = self.server_handle.take() {
            if let Ok(handle) = Arc::try_unwrap(handle) {
                handle.abort();
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
struct AppState {
    device_info: DeviceInfo,
    orchestrator: Arc<TransferOrchestrator>,
}

// Handler: GET /api/localsend/v2/info
async fn info_handler(State(state): State<AppState>) -> Json<InfoResponse> {
    Json(InfoResponse {
        alias: state.device_info.alias.clone(),
        version: state.device_info.version.clone(),
        device_model: state.device_info.device_model.clone(),
        device_type: state.device_info.device_type.as_ref().map(|t| format!("{:?}", t).to_lowercase()),
        fingerprint: state.device_info.fingerprint.clone(),
        download: state.device_info.download,
    })
}

// Handler: POST /api/localsend/v2/register
async fn register_handler(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<InfoResponse> {
    // Return our device info
    Json(InfoResponse {
        alias: state.device_info.alias.clone(),
        version: state.device_info.version.clone(),
        device_model: state.device_info.device_model.clone(),
        device_type: state.device_info.device_type.as_ref().map(|t| format!("{:?}", t).to_lowercase()),
        fingerprint: state.device_info.fingerprint.clone(),
        download: state.device_info.download,
    })
}

// Handler: POST /api/localsend/v2/prepare-upload
async fn prepare_upload_handler(
    State(state): State<AppState>,
    Json(request): Json<PrepareUploadRequest>,
) -> Result<Json<PrepareUploadResponse>, StatusCode> {
    let session_id = uuid::Uuid::new_v4().to_string();
    
    match state.orchestrator.handle_prepare_request(session_id.clone(), request).await {
        Ok(tokens) => Ok(Json(PrepareUploadResponse {
            session_id,
            files: tokens,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
struct UploadQuery {
    #[serde(rename = "sessionId")]
    session_id: String,
    #[serde(rename = "fileId")]
    file_id: String,
    token: String,
}

// Handler: POST /api/localsend/v2/upload
async fn upload_handler(
    State(state): State<AppState>,
    Query(params): Query<UploadQuery>,
    body: axum::body::Bytes,
) -> StatusCode {
    match state.orchestrator.receive_file(&params.session_id, &params.file_id, body.to_vec()).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[derive(Deserialize)]
struct CancelQuery {
    #[serde(rename = "sessionId")]
    session_id: String,
}

// Handler: POST /api/localsend/v2/cancel
async fn cancel_handler(
    State(state): State<AppState>,
    Query(params): Query<CancelQuery>,
) -> StatusCode {
    match state.orchestrator.cancel_transfer(&params.session_id).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
