//! Basic usage example for sync-auth.
//!
//! This example shows how to use the sync-auth library programmatically.
//!
//! Run with: `cargo run --example basic_usage`

use sync_auth::providers;
use sync_auth::{SyncConfig, SyncEngine};

#[tokio::main]
async fn main() {
    // List available providers
    println!("Available auth providers:");
    for provider in providers::all_providers() {
        println!("  {:<15} {}", provider.name(), provider.display_name());
        for cred in provider.credential_files() {
            let exists = if cred.local_path.exists() {
                "exists"
            } else {
                "not found"
            };
            println!(
                "    {} -> {} ({})",
                cred.relative_path,
                cred.local_path.display(),
                exists
            );
        }
    }
    println!();

    // Create a sync config (replace with your actual repo URL)
    let config = SyncConfig {
        repo_url: "https://github.com/YOUR_USER/YOUR_CREDENTIALS_REPO.git".to_string(),
        providers: vec!["gh".to_string(), "claude".to_string()],
        ..Default::default()
    };

    println!("Config:");
    println!("  repo:    {}", config.repo_url);
    println!("  path:    {}", config.local_path.display());
    println!("  branch:  {}", config.branch);
    println!("  shallow: {}", config.shallow_clone);

    // Create engine (this would fail with a placeholder URL, so just demo the setup)
    match SyncEngine::new(config) {
        Ok(engine) => {
            println!(
                "\nEngine created with {} provider(s).",
                engine.providers.len()
            );
            // In real usage you'd do:
            //   engine.pull().await?;   // Pull credentials from repo
            //   engine.push().await?;   // Push credentials to repo
            //   engine.sync().await?;   // Bidirectional sync
            //   engine.watch().await?;  // Watch mode
        }
        Err(e) => {
            eprintln!("Failed to create engine: {e}");
        }
    }
}
