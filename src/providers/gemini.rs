//! Gemini CLI auth provider.

use crate::{AuthProvider, CredentialFile, ValidationResult};
use std::path::PathBuf;

/// Provider for Google Gemini CLI credentials.
///
/// Gemini CLI stores config in `~/.gemini/`.
#[derive(Debug, Clone, Default)]
pub struct GeminiProvider;

#[async_trait::async_trait]
impl AuthProvider for GeminiProvider {
    fn name(&self) -> &str {
        "gemini"
    }

    fn display_name(&self) -> &str {
        "Gemini CLI"
    }

    fn credential_files(&self) -> Vec<CredentialFile> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
        let config_dir = dirs::config_dir().unwrap_or_else(|| home.join(".config"));
        vec![
            CredentialFile {
                relative_path: "gemini/dot-gemini".to_string(),
                local_path: home.join(".gemini"),
                is_dir: true,
            },
            CredentialFile {
                relative_path: "gemini/config-gemini".to_string(),
                local_path: config_dir.join("gemini"),
                is_dir: true,
            },
        ]
    }

    async fn validate(&self) -> ValidationResult {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
        if home.join(".gemini").exists() {
            ValidationResult::Valid
        } else {
            ValidationResult::Missing
        }
    }
}
