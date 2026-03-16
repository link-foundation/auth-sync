//! Sync engine — orchestrates bidirectional credential sync.

use crate::backend::{GitBackend, GitRepo};
use crate::providers;
use crate::{AuthProvider, CredentialFile, SyncConfig, SyncError, ValidationResult};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// The sync engine coordinates pulling credentials from a Git repo to local
/// filesystem and pushing local credentials to the repo.
pub struct SyncEngine {
    pub config: SyncConfig,
    pub backend: Box<dyn GitBackend>,
    pub providers: Vec<Box<dyn AuthProvider>>,
}

impl SyncEngine {
    /// Create a new `SyncEngine` with the given config and default Git backend.
    pub fn new(config: SyncConfig) -> Result<Self, SyncError> {
        if config.repo_url.is_empty() {
            return Err(SyncError::Config("repo_url must not be empty".to_string()));
        }

        let active_providers = resolve_providers(&config.providers)?;
        info!(
            providers = ?active_providers.iter().map(|p| p.name()).collect::<Vec<_>>(),
            "initialized sync engine"
        );

        Ok(Self {
            config,
            backend: Box::new(GitRepo),
            providers: active_providers,
        })
    }

    /// Create a `SyncEngine` with a custom backend (for testing or alternative storage).
    pub fn with_backend(
        config: SyncConfig,
        backend: Box<dyn GitBackend>,
    ) -> Result<Self, SyncError> {
        let active_providers = resolve_providers(&config.providers)?;
        Ok(Self {
            config,
            backend,
            providers: active_providers,
        })
    }

    /// Ensure the sync repo is cloned locally; clone if not.
    pub async fn ensure_repo(&self) -> Result<(), SyncError> {
        if self.backend.is_cloned(&self.config.local_path) {
            debug!(path = %self.config.local_path.display(), "repo already cloned");
            return Ok(());
        }
        self.backend
            .clone_repo(
                &self.config.repo_url,
                &self.config.local_path,
                self.config.shallow_clone,
            )
            .await
    }

    /// Pull credentials from the remote repo to local filesystem.
    ///
    /// 1. Ensure repo is cloned
    /// 2. Git pull
    /// 3. Copy files from repo → local credential paths
    pub async fn pull(&self) -> Result<SyncReport, SyncError> {
        self.ensure_repo().await?;
        self.backend.pull(&self.config.local_path).await?;

        let mut report = SyncReport::default();

        for provider in &self.providers {
            for cred in provider.credential_files() {
                let repo_path = self.config.local_path.join(&cred.relative_path);
                if repo_path.exists() {
                    copy_recursive(&repo_path, &cred.local_path).await?;
                    report.pulled.push(cred.relative_path.clone());
                    info!(
                        provider = provider.name(),
                        path = %cred.local_path.display(),
                        "pulled credential"
                    );
                } else {
                    debug!(
                        provider = provider.name(),
                        repo_path = %repo_path.display(),
                        "no credential in repo, skipping"
                    );
                }
            }
        }

        Ok(report)
    }

    /// Push local credentials to the remote repo.
    ///
    /// 1. Ensure repo is cloned
    /// 2. Git pull (to avoid conflicts)
    /// 3. Copy local credential files → repo
    /// 4. Git commit + push
    pub async fn push(&self) -> Result<SyncReport, SyncError> {
        self.ensure_repo().await?;

        // Pull first to minimize conflicts
        if self.backend.is_cloned(&self.config.local_path) {
            let _ = self.backend.pull(&self.config.local_path).await;
        }

        let mut report = SyncReport::default();

        for provider in &self.providers {
            // Validate before pushing — don't push expired credentials
            let validation = provider.validate().await;
            if validation == ValidationResult::Expired {
                warn!(
                    provider = provider.name(),
                    "skipping push: credentials are expired"
                );
                report
                    .skipped
                    .push(format!("{}: credentials expired", provider.name()));
                continue;
            }

            for cred in provider.credential_files() {
                if cred.local_path.exists() {
                    let repo_path = self.config.local_path.join(&cred.relative_path);
                    copy_recursive(&cred.local_path, &repo_path).await?;
                    report.pushed.push(cred.relative_path.clone());
                    info!(
                        provider = provider.name(),
                        path = %cred.local_path.display(),
                        "staged credential for push"
                    );
                } else {
                    debug!(
                        provider = provider.name(),
                        path = %cred.local_path.display(),
                        "local credential not found, skipping"
                    );
                }
            }
        }

        if !report.pushed.is_empty() {
            let message = format!(
                "sync-auth: update credentials ({})",
                report
                    .pushed
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            self.backend.push(&self.config.local_path, &message).await?;
        }

        Ok(report)
    }

    /// Bidirectional sync: pull then push.
    pub async fn sync(&self) -> Result<SyncReport, SyncError> {
        let mut report = self.pull().await?;
        let push_report = self.push().await?;
        report.pushed = push_report.pushed;
        report.skipped.extend(push_report.skipped);
        Ok(report)
    }

    /// Watch for local credential changes and sync periodically.
    pub async fn watch(&self) -> Result<(), SyncError> {
        use tokio::time::{interval, Duration};
        info!(
            interval_secs = self.config.watch_interval_secs,
            "starting watch mode"
        );

        let mut tick = interval(Duration::from_secs(self.config.watch_interval_secs));
        loop {
            tick.tick().await;
            match self.sync().await {
                Ok(report) => {
                    if !report.pushed.is_empty() || !report.pulled.is_empty() {
                        info!(?report, "sync cycle completed with changes");
                    } else {
                        debug!("sync cycle: no changes");
                    }
                }
                Err(e) => {
                    warn!(error = %e, "sync cycle failed, will retry next interval");
                }
            }
        }
    }

    /// List all providers and their credential status.
    pub async fn status(&self) -> Vec<ProviderStatus> {
        let mut statuses = Vec::new();
        for provider in &self.providers {
            let validation = provider.validate().await;
            let files: Vec<_> = provider
                .credential_files()
                .into_iter()
                .map(|c| {
                    let repo_exists = self.config.local_path.join(&c.relative_path).exists();
                    FileStatus {
                        relative_path: c.relative_path,
                        local_exists: c.local_path.exists(),
                        repo_exists,
                    }
                })
                .collect();
            statuses.push(ProviderStatus {
                name: provider.name().to_string(),
                display_name: provider.display_name().to_string(),
                validation,
                files,
            });
        }
        statuses
    }
}

/// Report of a sync operation.
#[derive(Debug, Default)]
pub struct SyncReport {
    pub pulled: Vec<String>,
    pub pushed: Vec<String>,
    pub skipped: Vec<String>,
}

/// Status of a provider's credentials.
#[derive(Debug)]
pub struct ProviderStatus {
    pub name: String,
    pub display_name: String,
    pub validation: ValidationResult,
    pub files: Vec<FileStatus>,
}

/// Status of a single credential file.
#[derive(Debug)]
pub struct FileStatus {
    pub relative_path: String,
    pub local_exists: bool,
    pub repo_exists: bool,
}

/// Resolve provider list: if empty, use all; otherwise look up by name.
fn resolve_providers(names: &[String]) -> Result<Vec<Box<dyn AuthProvider>>, SyncError> {
    if names.is_empty() {
        return Ok(providers::all_providers());
    }
    names
        .iter()
        .map(|name| {
            providers::provider_by_name(name)
                .ok_or_else(|| SyncError::ProviderNotFound(name.clone()))
        })
        .collect()
}

/// Recursively copy a file or directory, creating parent dirs as needed.
fn copy_recursive<'a>(
    src: &'a Path,
    dst: &'a Path,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), SyncError>> + Send + 'a>> {
    Box::pin(async move {
        if src.is_dir() {
            tokio::fs::create_dir_all(dst).await?;
            let mut entries = tokio::fs::read_dir(src).await?;
            while let Some(entry) = entries.next_entry().await? {
                let entry_path = entry.path();
                let file_name = entry.file_name().to_string_lossy().to_string();
                let dst_child = dst.join(&file_name);
                copy_recursive(&entry_path, &dst_child).await?;
            }
        } else if src.is_file() {
            if let Some(parent) = dst.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::copy(src, dst).await?;
        }
        Ok(())
    })
}

/// Helper to get a PathBuf for credential files (used by providers).
pub fn _credential_path(relative: &str) -> CredentialFile {
    CredentialFile {
        relative_path: relative.to_string(),
        local_path: PathBuf::from(relative),
        is_dir: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_all_providers() {
        let providers = resolve_providers(&[]).unwrap();
        assert_eq!(providers.len(), 7);
    }

    #[test]
    fn test_resolve_specific_providers() {
        let names = vec!["gh".to_string(), "claude".to_string()];
        let providers = resolve_providers(&names).unwrap();
        assert_eq!(providers.len(), 2);
        assert_eq!(providers[0].name(), "gh");
        assert_eq!(providers[1].name(), "claude");
    }

    #[test]
    fn test_resolve_unknown_provider() {
        let names = vec!["nonexistent".to_string()];
        let result = resolve_providers(&names);
        assert!(result.is_err());
    }

    #[test]
    fn test_new_engine_requires_repo_url() {
        let config = SyncConfig::default();
        let result = SyncEngine::new(config);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_copy_recursive_file() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src.txt");
        let dst = tmp.path().join("nested").join("dst.txt");
        tokio::fs::write(&src, "hello").await.unwrap();
        copy_recursive(&src, &dst).await.unwrap();
        let content = tokio::fs::read_to_string(&dst).await.unwrap();
        assert_eq!(content, "hello");
    }

    #[tokio::test]
    async fn test_copy_recursive_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let src_dir = tmp.path().join("src_dir");
        let dst_dir = tmp.path().join("dst_dir");
        tokio::fs::create_dir_all(&src_dir).await.unwrap();
        tokio::fs::write(src_dir.join("a.txt"), "aaa")
            .await
            .unwrap();
        tokio::fs::write(src_dir.join("b.txt"), "bbb")
            .await
            .unwrap();
        copy_recursive(&src_dir, &dst_dir).await.unwrap();
        assert_eq!(
            tokio::fs::read_to_string(dst_dir.join("a.txt"))
                .await
                .unwrap(),
            "aaa"
        );
        assert_eq!(
            tokio::fs::read_to_string(dst_dir.join("b.txt"))
                .await
                .unwrap(),
            "bbb"
        );
    }
}
