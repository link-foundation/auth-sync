//! GitLab CLI (`glab`) auth provider.

use crate::{AuthProvider, CredentialFile, ValidationResult};

/// Provider for GitLab CLI credentials (`~/.config/glab-cli/`).
#[derive(Debug, Clone, Default)]
pub struct GlabProvider;

#[async_trait::async_trait]
impl AuthProvider for GlabProvider {
    fn name(&self) -> &str {
        "glab"
    }

    fn display_name(&self) -> &str {
        "GitLab CLI"
    }

    fn credential_files(&self) -> Vec<CredentialFile> {
        let base = dirs::config_dir()
            .unwrap_or_else(|| "~/.config".into())
            .join("glab-cli");
        vec![CredentialFile {
            relative_path: "glab-cli".to_string(),
            local_path: base,
            is_dir: true,
        }]
    }

    async fn validate(&self) -> ValidationResult {
        match tokio::process::Command::new("glab")
            .args(["auth", "status"])
            .output()
            .await
        {
            Ok(output) if output.status.success() => ValidationResult::Valid,
            Ok(_) => ValidationResult::Expired,
            Err(_) => ValidationResult::Unknown,
        }
    }
}
