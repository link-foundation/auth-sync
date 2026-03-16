//! Opencode auth provider.

use crate::{AuthProvider, CredentialFile, ValidationResult};
use std::path::PathBuf;

/// Provider for opencode credentials.
///
/// Opencode stores auth in `~/.local/share/opencode/auth.json` (XDG data dir)
/// and config/plugins in `~/.config/opencode/`.
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
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("~"))
                    .join(".local/share")
            })
            .join("opencode");
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("~"))
                    .join(".config")
            })
            .join("opencode");
        vec![
            CredentialFile {
                relative_path: "opencode/data".to_string(),
                local_path: data_dir,
                is_dir: true,
            },
            CredentialFile {
                relative_path: "opencode/config".to_string(),
                local_path: config_dir,
                is_dir: true,
            },
        ]
    }

    async fn validate(&self) -> ValidationResult {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("~"))
                    .join(".local/share")
            })
            .join("opencode");
        if data_dir.join("auth.json").exists() {
            ValidationResult::Valid
        } else {
            ValidationResult::Missing
        }
    }
}
