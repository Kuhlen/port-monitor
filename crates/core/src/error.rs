use serde::{Deserialize, Serialize};

// Variants must stay pure so the frontend can Deserialize them.
// Infra errors get flattened into Port/Update in infra.rs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, thiserror::Error)]
pub enum AppError {
    #[error("port: {0}")]
    Port(String),
    #[error("invalid: {0}")]
    Validation(String),
    #[error("update: {0}")]
    Update(String),
    #[error("already connected")]
    AlreadyConnected,
    #[error("not connected")]
    NotConnected,
    #[error("IPC unavailable")]
    IpcUnavailable,
}
