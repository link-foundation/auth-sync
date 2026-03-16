//! OpenAI Codex CLI auth provider.

use crate::{AuthProvider, CredentialFile, ValidationResult};
use std::path::PathBuf;

/// Provider for OpenAI Codex CLI credentials.
///
/// Codex CLI stores auth config in `~/.codex/` (or `~/.config/codex/` on some setups).
#[derive(Debug, Clone, Default)]
pub struct CodexProvider;

#[async_trait::async_trait]
impl AuthProvider for CodexProvider {
    fn name(&self) -> &str {
        "codex"
    }

    fn display_name(&self) -> &str {
        "OpenAI Codex CLI"
    }

    fn credential_files(&self) -> Vec<CredentialFile> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
        let config_dir = dirs::config_dir().unwrap_or_else(|| home.join(".config"));
        vec![
            CredentialFile {
                relative_path: "codex/dot-codex".to_string(),
                local_path: home.join(".codex"),
                is_dir: true,
            },
            CredentialFile {
                relative_path: "codex/config-codex".to_string(),
                local_path: config_dir.join("codex"),
                is_dir: true,
            },
        ]
    }

    async fn validate(&self) -> ValidationResult {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
        if home.join(".codex").exists() {
            ValidationResult::Valid
        } else {
            ValidationResult::Missing
        }
    }
}
