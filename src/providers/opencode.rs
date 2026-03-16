//! Opencode auth provider.

use crate::{AuthProvider, CredentialFile, ValidationResult};
use std::path::PathBuf;

/// Provider for opencode credentials.
///
/// Opencode stores config in `~/.opencode/` or `~/.config/opencode/`.
#[derive(Debug, Clone, Default)]
pub struct OpencodeProvider;

#[async_trait::async_trait]
impl AuthProvider for OpencodeProvider {
    fn name(&self) -> &str {
        "opencode"
    }

    fn display_name(&self) -> &str {
        "Opencode"
    }

    fn credential_files(&self) -> Vec<CredentialFile> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
        let config_dir = dirs::config_dir().unwrap_or_else(|| home.join(".config"));
        vec![
            CredentialFile {
                relative_path: "opencode/dot-opencode".to_string(),
                local_path: home.join(".opencode"),
                is_dir: true,
            },
            CredentialFile {
                relative_path: "opencode/config-opencode".to_string(),
                local_path: config_dir.join("opencode"),
                is_dir: true,
            },
        ]
    }

    async fn validate(&self) -> ValidationResult {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
        if home.join(".opencode").exists() {
            ValidationResult::Valid
        } else {
            ValidationResult::Missing
        }
    }
}
