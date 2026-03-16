//! Error types for sync-auth.

/// Errors that can occur during sync operations.
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("git operation failed: {0}")]
    Git(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("provider not found: {0}")]
    ProviderNotFound(String),

    #[error("config error: {0}")]
    Config(String),

    #[error("sync conflict: {0}")]
    Conflict(String),

    #[error("serialization error: {0}")]
    Serialization(String),
}
