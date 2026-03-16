//! OpenAI Codex CLI auth provider.

use crate::{AuthProvider, CredentialFile, ValidationResult};
use std::path::PathBuf;

/// Provider for OpenAI Codex CLI credentials.
///
/// Codex CLI stores auth in `~/.codex/` (controlled by `CODEX_HOME` env var).
/// Key files: `auth.json` (tokens), `config.toml` (settings).
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
        let codex_home = std::env::var("CODEX_HOME").map_or_else(
            |_| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("~"))
                    .join(".codex")
            },
            PathBuf::from,
        );
        vec![CredentialFile {
            relative_path: "codex/dot-codex".to_string(),
            local_path: codex_home,
            is_dir: true,
        }]
    }

    async fn validate(&self) -> ValidationResult {
        let codex_home = std::env::var("CODEX_HOME").map_or_else(
            |_| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("~"))
                    .join(".codex")
            },
            PathBuf::from,
        );
        if codex_home.join("auth.json").exists() {
            ValidationResult::Valid
        } else {
            ValidationResult::Missing
        }
    }
}
