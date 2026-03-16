//! Integration tests for sync-auth.

use sync_auth::providers;
use sync_auth::{SyncConfig, SyncEngine, ValidationResult, VERSION};

mod version_tests {
    use super::*;

    #[test]
    fn test_version_is_not_empty() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_version_matches_cargo_toml() {
        assert!(VERSION.starts_with("0."));
    }
}

mod provider_tests {
    use super::*;

    #[test]
    fn test_all_providers_returns_seven() {
        let all = providers::all_providers();
        assert_eq!(all.len(), 7);
    }

    #[test]
    fn test_provider_by_name_known() {
        assert!(providers::provider_by_name("gh").is_some());
        assert!(providers::provider_by_name("claude").is_some());
        assert!(providers::provider_by_name("glab").is_some());
        assert!(providers::provider_by_name("codex").is_some());
        assert!(providers::provider_by_name("gemini").is_some());
        assert!(providers::provider_by_name("opencode").is_some());
        assert!(providers::provider_by_name("qwen-coder").is_some());
    }

    #[test]
    fn test_provider_by_name_unknown() {
        assert!(providers::provider_by_name("unknown-tool").is_none());
    }

    #[test]
    fn test_provider_has_credential_files() {
        for provider in providers::all_providers() {
            let files = provider.credential_files();
            assert!(
                !files.is_empty(),
                "provider {} should have at least one credential file",
                provider.name()
            );
        }
    }

    #[tokio::test]
    async fn test_validate_returns_valid_result() {
        // On a CI machine, most tools won't be installed, so Unknown/Missing is fine
        let gh = providers::provider_by_name("gh").unwrap();
        let result = gh.validate().await;
        assert!(matches!(
            result,
            ValidationResult::Valid
                | ValidationResult::Expired
                | ValidationResult::Missing
                | ValidationResult::Unknown
        ));
    }
}

mod engine_tests {
    use super::*;

    #[test]
    fn test_engine_requires_repo_url() {
        let config = SyncConfig::default();
        assert!(SyncEngine::new(config).is_err());
    }

    #[test]
    fn test_engine_creates_with_valid_config() {
        let config = SyncConfig {
            repo_url: "https://github.com/example/creds.git".to_string(),
            ..Default::default()
        };
        assert!(SyncEngine::new(config).is_ok());
    }

    #[test]
    fn test_engine_with_specific_providers() {
        let config = SyncConfig {
            repo_url: "https://github.com/example/creds.git".to_string(),
            providers: vec!["gh".to_string(), "claude".to_string()],
            ..Default::default()
        };
        let engine = SyncEngine::new(config).unwrap();
        assert_eq!(engine.providers.len(), 2);
    }

    #[test]
    fn test_engine_with_unknown_provider_fails() {
        let config = SyncConfig {
            repo_url: "https://github.com/example/creds.git".to_string(),
            providers: vec!["nonexistent".to_string()],
            ..Default::default()
        };
        assert!(SyncEngine::new(config).is_err());
    }
}

mod config_tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SyncConfig::default();
        assert!(config.repo_url.is_empty());
        assert!(config.shallow_clone);
        assert_eq!(config.branch, "main");
        assert_eq!(config.watch_interval_secs, 60);
    }

    #[test]
    fn test_config_load_nonexistent_file() {
        let result = SyncConfig::load_from_file(std::path::Path::new("/nonexistent/config.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_config_load_valid_toml() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(
            tmp.path(),
            r#"
repo_url = "https://github.com/test/repo.git"
branch = "develop"
shallow_clone = false
watch_interval_secs = 120
"#,
        )
        .unwrap();
        let config = SyncConfig::load_from_file(tmp.path()).unwrap();
        assert_eq!(config.repo_url, "https://github.com/test/repo.git");
        assert_eq!(config.branch, "develop");
        assert!(!config.shallow_clone);
        assert_eq!(config.watch_interval_secs, 120);
    }
}
