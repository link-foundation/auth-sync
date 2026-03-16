//! Configuration for sync-auth.

use std::path::PathBuf;

/// Configuration for the sync engine.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct SyncConfig {
    /// URL of the Git repository to sync credentials through.
    pub repo_url: String,

    /// Local path where the sync repository is cloned.
    pub local_path: PathBuf,

    /// List of provider names to sync (e.g. \["gh", "claude"\]).
    /// Empty means sync all available providers.
    pub providers: Vec<String>,

    /// Whether to use shallow clone (--depth 1) for initial clone.
    pub shallow_clone: bool,

    /// Git branch to use for sync.
    pub branch: String,

    /// Interval in seconds for watch mode.
    pub watch_interval_secs: u64,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            repo_url: String::new(),
            local_path: default_sync_path(),
            providers: Vec::new(),
            shallow_clone: true,
            branch: "main".to_string(),
            watch_interval_secs: 60,
        }
    }
}

/// Returns the default path for the sync repository.
fn default_sync_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("sync-auth")
        .join("repo")
}

impl SyncConfig {
    /// Load config from a TOML file, falling back to defaults for missing fields.
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, crate::SyncError> {
        if !path.exists() {
            return Err(crate::SyncError::Config(format!(
                "config file not found: {}",
                path.display()
            )));
        }
        let content =
            std::fs::read_to_string(path).map_err(|e| crate::SyncError::Config(e.to_string()))?;
        toml::from_str(&content)
            .map_err(|e| crate::SyncError::Config(format!("invalid config TOML: {e}")))
    }

    /// Returns the default config file path.
    pub fn default_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("sync-auth")
            .join("config.toml")
    }
}
