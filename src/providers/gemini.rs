//! Gemini CLI auth provider.

use crate::{AuthProvider, CredentialFile, ValidationResult};
use std::path::PathBuf;

/// Provider for Google Gemini CLI credentials.
///
/// Gemini CLI stores config in `~/.gemini/`.
/// Key files: `.env` (API key), `oauth_creds.json` (OAuth tokens),
/// `mcp-oauth-tokens.json` (MCP OAuth tokens).
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
        vec![CredentialFile {
            relative_path: "gemini/dot-gemini".to_string(),
            local_path: home.join(".gemini"),
            is_dir: true,
        }]
    }

    async fn validate(&self) -> ValidationResult {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
        let gemini_dir = home.join(".gemini");
        // Check for either API key env file or OAuth credentials
        if gemini_dir.join(".env").exists() || gemini_dir.join("oauth_creds.json").exists() {
            ValidationResult::Valid
        } else {
            ValidationResult::Missing
        }
    }
}
