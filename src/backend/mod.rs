//! Git backend for credential storage and retrieval.

mod git;

pub use git::GitRepo;

use std::path::Path;

/// Trait for Git-like backends that store and retrieve credential snapshots.
///
/// Implement this trait to provide a custom storage backend (e.g. GitLab,
/// a custom Git server, or even a non-Git store).
#[async_trait::async_trait]
pub trait GitBackend: Send + Sync {
    /// Clone the repository (shallow if supported) to `local_path`.
    async fn clone_repo(
        &self,
        url: &str,
        local_path: &Path,
        shallow: bool,
    ) -> Result<(), crate::SyncError>;

    /// Pull latest changes from remote.
    async fn pull(&self, local_path: &Path) -> Result<(), crate::SyncError>;

    /// Stage, commit, and push local changes.
    async fn push(&self, local_path: &Path, message: &str) -> Result<(), crate::SyncError>;

    /// Check if the local path is already a valid repo clone.
    fn is_cloned(&self, local_path: &Path) -> bool;
}
