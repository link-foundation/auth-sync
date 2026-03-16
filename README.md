# sync-auth

Bidirectional auth credential sync for dev tools via Git repositories.

[![CI/CD Pipeline](https://github.com/link-foundation/auth-sync/workflows/CI%2FCD%20Pipeline/badge.svg)](https://github.com/link-foundation/auth-sync/actions)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org/)
[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](http://unlicense.org/)

Sync authentication credentials for developer tools (GitHub CLI, GitLab CLI, Claude Code, Codex, Gemini CLI, and more) through a Git repository. Works in containers, CI runners, and across machines.

## Problem

Developers using AI coding tools and platform CLIs need to re-authenticate in every new container, CI runner, or machine. There's no universal way to sync these credentials.

## Features

- **Bidirectional sync** -- local credentials to/from a Git repository
- **7 built-in providers**: `gh`, `glab`, `claude`, `codex`, `gemini`, `opencode`, `qwen-coder`
- **Extensible** -- add custom providers by implementing the `AuthProvider` trait
- **Conflict resolution** -- skips expired/dead tokens, prefers fresher credentials
- **Shallow clone** -- fast initial setup with `--depth 1`
- **Watch mode** -- continuous monitoring and periodic sync
- **Daemon support** -- start/stop/restart as background process or systemd service
- **CI/CD ready** -- usable in GitHub Actions, GitLab CI, Docker containers
- **Config file + env vars** -- TOML config with CLI/env override support

## Supported Providers

| Provider | Tool | Credential paths |
|---|---|---|
| `gh` | GitHub CLI | `~/.config/gh/` |
| `glab` | GitLab CLI | `~/.config/glab-cli/` |
| `claude` | Claude Code | `~/.claude/`, `~/.claude.json` |
| `codex` | OpenAI Codex CLI | `~/.codex/`, `~/.config/codex/` |
| `gemini` | Gemini CLI | `~/.gemini/`, `~/.config/gemini/` |
| `opencode` | Opencode | `~/.opencode/`, `~/.config/opencode/` |
| `qwen-coder` | Qwen Coder | `~/.qwen-coder/`, `~/.config/qwen-coder/` |

## Quick Start

### Install

```bash
cargo install sync-auth
```

### Initialize config

```bash
sync-auth init
# Edit ~/.config/sync-auth/config.toml with your repo URL
```

### Basic usage

```bash
# Pull credentials from remote repo
sync-auth --repo https://github.com/USER/credentials.git pull

# Push local credentials to remote repo
sync-auth --repo https://github.com/USER/credentials.git push

# Bidirectional sync
sync-auth --repo https://github.com/USER/credentials.git sync

# Sync only specific providers
sync-auth --repo https://github.com/USER/credentials.git -p gh,claude sync

# Watch mode (sync every 60 seconds)
sync-auth --repo https://github.com/USER/credentials.git watch --interval 60

# Check status
sync-auth --repo https://github.com/USER/credentials.git status

# List available providers
sync-auth providers
```

### Environment variables

All CLI options can be set via environment variables:

```bash
export SYNC_AUTH_REPO="https://github.com/USER/credentials.git"
export SYNC_AUTH_PROVIDERS="gh,claude"
export SYNC_AUTH_BRANCH="main"

sync-auth sync
```

### Daemon mode

```bash
# Start as background daemon
sync-auth --repo https://github.com/USER/credentials.git daemon start --interval 60

# Stop daemon
sync-auth daemon stop

# Restart
sync-auth daemon restart

# Print systemd service unit for permanent installation
sync-auth daemon setup
```

## Library Usage

```rust
use sync_auth::{SyncEngine, SyncConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = SyncConfig {
        repo_url: "https://github.com/user/my-credentials.git".to_string(),
        providers: vec!["gh".to_string(), "claude".to_string()],
        ..Default::default()
    };

    let engine = SyncEngine::new(config)?;

    // Pull credentials from repo to local filesystem
    let report = engine.pull().await?;
    println!("Pulled {} credentials", report.pulled.len());

    // Push local credentials to repo
    let report = engine.push().await?;
    println!("Pushed {} credentials", report.pushed.len());

    Ok(())
}
```

### Custom provider

```rust
use sync_auth::{AuthProvider, CredentialFile, ValidationResult};

#[derive(Debug)]
struct MyToolProvider;

#[async_trait::async_trait]
impl AuthProvider for MyToolProvider {
    fn name(&self) -> &str { "my-tool" }
    fn display_name(&self) -> &str { "My Custom Tool" }

    fn credential_files(&self) -> Vec<CredentialFile> {
        vec![CredentialFile {
            relative_path: "my-tool/config".to_string(),
            local_path: dirs::home_dir().unwrap().join(".my-tool"),
            is_dir: true,
        }]
    }

    async fn validate(&self) -> ValidationResult {
        ValidationResult::Unknown
    }
}
```

## Configuration

Config file location: `~/.config/sync-auth/config.toml`

```toml
# Git repository URL (required)
repo_url = "https://github.com/USER/credentials.git"

# Providers to sync (empty = all)
providers = ["gh", "claude", "glab"]

# Git branch
branch = "main"

# Use shallow clone for initial setup
shallow_clone = true

# Watch mode interval (seconds)
watch_interval_secs = 60
```

## Docker / CI Usage

### GitHub Actions

```yaml
- name: Sync credentials
  run: |
    cargo install sync-auth
    sync-auth --repo ${{ secrets.CREDENTIALS_REPO }} pull
```

### Docker (with link-foundation/sandbox)

```bash
# On host: push credentials
sync-auth --repo https://github.com/USER/credentials.git push

# In container: pull credentials
docker exec my-sandbox sync-auth --repo https://github.com/USER/credentials.git pull
```

## Architecture

```
sync-auth
├── Library crate (sync_auth)
│   ├── AuthProvider trait     -- extensible provider system
│   ├── GitBackend trait       -- pluggable storage backend
│   ├── SyncEngine             -- orchestrates sync operations
│   ├── SyncConfig             -- TOML-based configuration
│   └── providers/             -- built-in providers (gh, claude, etc.)
└── CLI binary (sync-auth)
    └── Thin wrapper over the library with clap-based CLI
```

## Prior Art

- [link-assistant/claude-profiles](https://github.com/link-assistant/claude-profiles) -- Node.js CLI that syncs Claude credentials via GitHub Gists (Claude-only, size-limited)

## Development

```bash
cargo build           # Build
cargo test            # Run all tests
cargo run -- --help   # Run CLI
cargo clippy          # Lint
cargo fmt             # Format
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

[Unlicense](LICENSE) -- Public Domain
