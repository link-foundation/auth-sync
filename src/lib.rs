//! `sync-auth` — Bidirectional auth credential sync for dev tools via Git repositories.
//!
//! This library provides a trait-based, extensible system for syncing authentication
//! credentials for developer tools (GitHub CLI, GitLab CLI, Claude Code, Codex,
//! Gemini CLI, etc.) through a Git repository backend.
//!
//! # Architecture
//!
//! - [`AuthProvider`] trait: implement to add support for any dev tool's credentials
//! - [`GitBackend`] trait: implement to customize how credentials are stored/fetched
//! - [`SyncEngine`]: orchestrates bidirectional sync between local and remote
//! - Built-in providers for common tools via [`providers`] module
//!
//! # Example
//!
//! ```no_run
//! use sync_auth::{SyncEngine, SyncConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = SyncConfig {
//!         repo_url: "https://github.com/user/my-credentials.git".to_string(),
//!         local_path: "/tmp/sync-auth-repo".into(),
//!         providers: vec!["gh".to_string(), "claude".to_string()],
//!         ..Default::default()
//!     };
//!     let engine = SyncEngine::new(config)?;
//!     engine.pull().await?;
//!     Ok(())
//! }
//! ```

pub mod backend;
pub mod providers;

mod config;
mod engine;
mod error;

pub use config::SyncConfig;
pub use engine::SyncEngine;
pub use error::SyncError;

/// Package version (matches Cargo.toml version).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Credential file entry describing a single file to sync.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CredentialFile {
    /// Relative path within the provider's namespace in the sync repo.
    pub relative_path: String,
    /// Absolute path on the local filesystem.
    pub local_path: std::path::PathBuf,
    /// Whether this is a directory (recursive sync).
    pub is_dir: bool,
}

/// Result of validating a credential.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    /// Credential is valid and usable.
    Valid,
    /// Credential exists but is expired or invalid.
    Expired,
    /// Credential is missing or empty.
    Missing,
    /// Validation could not be performed (tool not installed, etc.).
    Unknown,
}

/// Trait for auth credential providers.
///
/// Implement this trait to add support for syncing credentials of any dev tool.
/// Each provider declares its name, the credential files it manages, and
/// optionally a validation method to check credential freshness.
#[async_trait::async_trait]
pub trait AuthProvider: Send + Sync {
    /// Unique name for this provider (e.g. "gh", "claude").
    fn name(&self) -> &str;

    /// Human-readable display name.
    fn display_name(&self) -> &str;

    /// List of credential files/directories this provider manages.
    fn credential_files(&self) -> Vec<CredentialFile>;

    /// Validate whether the current local credentials are still valid.
    /// Defaults to [`ValidationResult::Unknown`].
    async fn validate(&self) -> ValidationResult {
        ValidationResult::Unknown
    }
}
