// Message framing and serialization

use super::ProtocolError;
use serde::{Deserialize, Serialize};

/// Serialize a message to JSON
pub fn serialize<T: Serialize>(msg: &T) -> Result<Vec<u8>, ProtocolError> {
    serde_json::to_vec(msg).map_err(ProtocolError::Serialization)
}

/// Deserialize a message from JSON
pub fn deserialize<'a, T: Deserialize<'a>>(data: &'a [u8]) -> Result<T, ProtocolError> {
    serde_json::from_slice(data).map_err(ProtocolError::Serialization)
}

/// Serialize to JSON string
pub fn serialize_string<T: Serialize>(msg: &T) -> Result<String, ProtocolError> {
    serde_json::to_string(msg).map_err(ProtocolError::Serialization)
}

/// Deserialize from JSON string
pub fn deserialize_string<'a, T: Deserialize<'a>>(data: &'a str) -> Result<T, ProtocolError> {
    serde_json::from_str(data).map_err(ProtocolError::Serialization)
}
