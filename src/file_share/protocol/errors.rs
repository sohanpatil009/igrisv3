// Protocol error types

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Invalid body")]
    InvalidBody,

    #[error("PIN required")]
    PinRequired,

    #[error("Invalid PIN")]
    InvalidPin,

    #[error("Rejected by user")]
    Rejected,

    #[error("Blocked by another session")]
    BlockedBySession,

    #[error("Too many requests")]
    TooManyRequests,

    #[error("Missing parameters")]
    MissingParameters,

    #[error("Invalid token or IP address")]
    InvalidToken,

    #[error("Unknown error: {0}")]
    Unknown(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Network error: {0}")]
    Network(String),
}

impl ProtocolError {
    pub fn http_status(&self) -> u16 {
        match self {
            Self::InvalidBody => 400,
            Self::PinRequired | Self::InvalidPin => 401,
            Self::Rejected | Self::InvalidToken => 403,
            Self::BlockedBySession => 409,
            Self::TooManyRequests => 429,
            Self::MissingParameters => 400,
            _ => 500,
        }
    }
}
