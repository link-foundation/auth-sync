//! Claude Code auth provider.

use crate::{AuthProvider, CredentialFile, ValidationResult};
use std::path::PathBuf;

/// Provider for Claude Code credentials.
///
/// Syncs `~/.claude/` directory and `~/.claude.json` config file.
#[derive(Debug, Clone, Default)]
pub struct ClaudeProvider;

#[async_trait::async_trait]
impl AuthProvider for ClaudeProvider {
    fn name(&self) -> &str {
        "claude"
    }

    fn display_name(&self) -> &str {
        "Claude Code"
    }

    fn credential_files(&self) -> Vec<CredentialFile> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
        vec![
            CredentialFile {
                relative_path: "claude/dot-claude".to_string(),
                local_path: home.join(".claude"),
                is_dir: true,
            },
            CredentialFile {
                relative_path: "claude/claude.json".to_string(),
                local_path: home.join(".claude.json"),
                is_dir: false,
            },
        ]
    }

    async fn validate(&self) -> ValidationResult {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
        let config = home.join(".claude.json");
        if config.exists() {
            ValidationResult::Valid
        } else {
            ValidationResult::Missing
        }
    }
}
