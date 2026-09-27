// infra errors flattened to String in data; app maps variants to user text
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AppError {
    #[error("port: {0}")]
    Port(String),
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("already connected")]
    AlreadyConnected,
    #[error("not connected")]
    NotConnected,
}
