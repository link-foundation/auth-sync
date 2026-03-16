//! Qwen Coder auth provider.

use crate::{AuthProvider, CredentialFile, ValidationResult};
use std::path::PathBuf;

/// Provider for Qwen Coder CLI credentials.
///
/// Qwen Coder stores config in `~/.qwen-coder/` or `~/.config/qwen-coder/`.
#[derive(Debug, Clone, Default)]
pub struct QwenCoderProvider;

#[async_trait::async_trait]
impl AuthProvider for QwenCoderProvider {
    fn name(&self) -> &str {
        "qwen-coder"
    }

    fn display_name(&self) -> &str {
        "Qwen Coder"
    }

    fn credential_files(&self) -> Vec<CredentialFile> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
        let config_dir = dirs::config_dir().unwrap_or_else(|| home.join(".config"));
        vec![
            CredentialFile {
                relative_path: "qwen-coder/dot-qwen-coder".to_string(),
                local_path: home.join(".qwen-coder"),
                is_dir: true,
            },
            CredentialFile {
                relative_path: "qwen-coder/config-qwen-coder".to_string(),
                local_path: config_dir.join("qwen-coder"),
                is_dir: true,
            },
        ]
    }

    async fn validate(&self) -> ValidationResult {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
        if home.join(".qwen-coder").exists() {
            ValidationResult::Valid
        } else {
            ValidationResult::Missing
        }
    }
}
