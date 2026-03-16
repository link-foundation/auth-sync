//! Qwen Code auth provider.

use crate::{AuthProvider, CredentialFile, ValidationResult};
use std::path::PathBuf;

/// Provider for Qwen Code CLI credentials (officially "Qwen Code", npm: `qwen-code`).
///
/// Qwen Code stores config in `~/.qwen/`.
/// Key files: `credentials.json` (OAuth tokens), `settings.json` (API keys/settings).
#[derive(Debug, Clone, Default)]
pub struct QwenCoderProvider;

#[async_trait::async_trait]
impl AuthProvider for QwenCoderProvider {
    fn name(&self) -> &str {
        "qwen-coder"
    }

    fn display_name(&self) -> &str {
        "Qwen Code"
    }

    fn credential_files(&self) -> Vec<CredentialFile> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
        vec![CredentialFile {
            relative_path: "qwen-coder/dot-qwen".to_string(),
            local_path: home.join(".qwen"),
            is_dir: true,
        }]
    }

    async fn validate(&self) -> ValidationResult {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
        let qwen_dir = home.join(".qwen");
        if qwen_dir.join("credentials.json").exists() || qwen_dir.join("settings.json").exists() {
            ValidationResult::Valid
        } else {
            ValidationResult::Missing
        }
    }
}
