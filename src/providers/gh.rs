//! GitHub CLI (`gh`) auth provider.

use crate::{AuthProvider, CredentialFile, ValidationResult};

/// Provider for GitHub CLI credentials (`~/.config/gh/`).
#[derive(Debug, Clone, Default)]
pub struct GhProvider;

#[async_trait::async_trait]
impl AuthProvider for GhProvider {
    fn name(&self) -> &str {
        "gh"
    }

    fn display_name(&self) -> &str {
        "GitHub CLI"
    }

    fn credential_files(&self) -> Vec<CredentialFile> {
        let base = dirs::config_dir()
            .unwrap_or_else(|| "~/.config".into())
            .join("gh");
        vec![CredentialFile {
            relative_path: "gh".to_string(),
            local_path: base,
            is_dir: true,
        }]
    }

    async fn validate(&self) -> ValidationResult {
        match tokio::process::Command::new("gh")
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
