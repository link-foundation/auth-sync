//! Default Git backend implementation using the `git` CLI.

use std::path::Path;
use tokio::process::Command;
use tracing::{debug, info};

/// Default Git backend that shells out to the `git` command.
#[derive(Debug, Clone, Default)]
pub struct GitRepo;

impl GitRepo {
    async fn run_git(args: &[&str], cwd: &Path) -> Result<String, crate::SyncError> {
        debug!(args = ?args, cwd = %cwd.display(), "running git command");
        let output = Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .await
            .map_err(|e| crate::SyncError::Git(format!("failed to run git: {e}")))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            Ok(stdout)
        } else {
            Err(crate::SyncError::Git(format!(
                "git {} failed (exit {}): {}",
                args.join(" "),
                output.status.code().unwrap_or(-1),
                stderr
            )))
        }
    }
}

#[async_trait::async_trait]
impl super::GitBackend for GitRepo {
    async fn clone_repo(
        &self,
        url: &str,
        local_path: &Path,
        shallow: bool,
    ) -> Result<(), crate::SyncError> {
        let parent = local_path
            .parent()
            .ok_or_else(|| crate::SyncError::Git("invalid local path".to_string()))?;
        tokio::fs::create_dir_all(parent).await?;

        let local_str = local_path.to_string_lossy().to_string();
        let mut args = vec!["clone"];
        if shallow {
            args.extend_from_slice(&["--depth", "1"]);
        }
        args.extend_from_slice(&[url, &local_str]);

        info!(url = url, path = %local_path.display(), shallow = shallow, "cloning repository");

        let output = Command::new("git")
            .args(&args)
            .output()
            .await
            .map_err(|e| crate::SyncError::Git(format!("failed to run git clone: {e}")))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(crate::SyncError::Git(format!("git clone failed: {stderr}")))
        }
    }

    async fn pull(&self, local_path: &Path) -> Result<(), crate::SyncError> {
        info!(path = %local_path.display(), "pulling latest changes");
        Self::run_git(&["pull", "--rebase", "--autostash"], local_path).await?;
        Ok(())
    }

    async fn push(&self, local_path: &Path, message: &str) -> Result<(), crate::SyncError> {
        // Stage all changes
        Self::run_git(&["add", "-A"], local_path).await?;

        // Check if there's anything to commit
        let status = Self::run_git(&["status", "--porcelain"], local_path).await?;
        if status.trim().is_empty() {
            info!("no changes to push");
            return Ok(());
        }

        info!(message = message, "committing and pushing changes");
        Self::run_git(&["commit", "-m", message], local_path).await?;
        Self::run_git(&["push"], local_path).await?;
        Ok(())
    }

    fn is_cloned(&self, local_path: &Path) -> bool {
        local_path.join(".git").is_dir()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::GitBackend;

    #[test]
    fn test_is_cloned_false_for_nonexistent() {
        let repo = GitRepo;
        assert!(!repo.is_cloned(Path::new("/nonexistent/path")));
    }

    #[tokio::test]
    async fn test_clone_creates_parent_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("a").join("b").join("repo");
        let repo = GitRepo;

        // This will fail because URL is invalid, but parent dirs should be created
        let result: Result<(), crate::SyncError> =
            repo.clone_repo("invalid://url", &nested, true).await;
        assert!(result.is_err());

        // Parent directory should have been created
        assert!(nested.parent().unwrap().exists());
    }
}
