//! `sync-auth` CLI — Bidirectional auth credential sync for dev tools.

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use sync_auth::{SyncConfig, SyncEngine};
use tracing::{error, info};

#[derive(Parser)]
#[command(
    name = "sync-auth",
    about = "Bidirectional auth credential sync for dev tools via Git repositories",
    version = sync_auth::VERSION,
    long_about = "Sync authentication credentials for developer tools (GitHub CLI, GitLab CLI, \
                   Claude Code, Codex, Gemini CLI, etc.) through a Git repository.\n\n\
                   On first run, the repo is shallow-cloned. Subsequent syncs pull/push changes."
)]
struct Cli {
    /// Git repository URL to sync through
    #[arg(short, long, env = "SYNC_AUTH_REPO")]
    repo: Option<String>,

    /// Local path for the sync repository clone
    #[arg(short, long, env = "SYNC_AUTH_LOCAL_PATH")]
    local_path: Option<PathBuf>,

    /// Config file path
    #[arg(short, long, env = "SYNC_AUTH_CONFIG")]
    config: Option<PathBuf>,

    /// Providers to sync (comma-separated, e.g. "gh,claude").
    /// If omitted, all providers are synced.
    #[arg(short, long, env = "SYNC_AUTH_PROVIDERS", value_delimiter = ',')]
    providers: Option<Vec<String>>,

    /// Git branch to use
    #[arg(short, long, default_value = "main", env = "SYNC_AUTH_BRANCH")]
    branch: String,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Pull credentials from remote repo to local filesystem
    Pull,

    /// Push local credentials to remote repo
    Push,

    /// Bidirectional sync (pull then push)
    Sync,

    /// Watch for changes and sync periodically
    Watch {
        /// Sync interval in seconds
        #[arg(short, long, default_value = "60")]
        interval: u64,
    },

    /// Show status of all providers and credentials
    Status,

    /// List available providers
    Providers,

    /// Initialize a new config file
    Init,

    /// Daemon management (Unix only)
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
}

#[derive(Subcommand)]
enum DaemonAction {
    /// Start the daemon
    Start {
        /// Sync interval in seconds
        #[arg(short, long, default_value = "60")]
        interval: u64,
    },
    /// Stop the daemon
    Stop,
    /// Restart the daemon
    Restart {
        /// Sync interval in seconds
        #[arg(short, long, default_value = "60")]
        interval: u64,
    },
    /// Print systemd unit file for installation
    Setup,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize tracing
    let filter = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(filter)),
        )
        .with_target(false)
        .init();

    if let Err(e) = run(cli).await {
        error!("{e}");
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<(), sync_auth::SyncError> {
    // Handle commands that don't need a full engine
    match &cli.command {
        Commands::Providers => {
            println!("Available providers:");
            for p in sync_auth::providers::all_providers() {
                println!("  {:<15} {}", p.name(), p.display_name());
            }
            return Ok(());
        }
        Commands::Init => {
            return init_config().await;
        }
        _ => {}
    }

    let config = build_config(&cli)?;
    let engine = SyncEngine::new(config)?;

    match cli.command {
        Commands::Pull => {
            let report = engine.pull().await?;
            println!("Pulled {} credential(s).", report.pulled.len());
            for p in &report.pulled {
                println!("  + {p}");
            }
        }
        Commands::Push => {
            let report = engine.push().await?;
            println!("Pushed {} credential(s).", report.pushed.len());
            for p in &report.pushed {
                println!("  + {p}");
            }
            for s in &report.skipped {
                println!("  - {s}");
            }
        }
        Commands::Sync => {
            let report = engine.sync().await?;
            println!(
                "Sync complete: {} pulled, {} pushed.",
                report.pulled.len(),
                report.pushed.len()
            );
        }
        Commands::Watch { interval } => {
            println!("Starting watch mode (interval: {interval}s). Press Ctrl+C to stop.");
            let mut config = engine.config.clone();
            config.watch_interval_secs = interval;
            let engine = SyncEngine::new(config)?;
            engine.watch().await?;
        }
        Commands::Status => {
            let statuses = engine.status().await;
            println!("Provider status:");
            for s in &statuses {
                let validation = match s.validation {
                    sync_auth::ValidationResult::Valid => "valid",
                    sync_auth::ValidationResult::Expired => "EXPIRED",
                    sync_auth::ValidationResult::Missing => "missing",
                    sync_auth::ValidationResult::Unknown => "unknown",
                };
                println!("  {} ({}) -- {}", s.name, s.display_name, validation);
                for f in &s.files {
                    let local = if f.local_exists { "+" } else { "-" };
                    let repo = if f.repo_exists { "+" } else { "-" };
                    println!("    {} local:{} repo:{}", f.relative_path, local, repo);
                }
            }
        }
        Commands::Daemon { action } => {
            handle_daemon(action, cli.repo.as_deref(), cli.config.as_deref()).await?;
        }
        Commands::Providers | Commands::Init => unreachable!(),
    }

    Ok(())
}

fn build_config(cli: &Cli) -> Result<SyncConfig, sync_auth::SyncError> {
    let config_path = cli
        .config
        .clone()
        .unwrap_or_else(SyncConfig::default_config_path);

    let mut config = if config_path.exists() {
        info!(path = %config_path.display(), "loading config from file");
        SyncConfig::load_from_file(&config_path)?
    } else {
        SyncConfig::default()
    };

    // CLI args override config file
    if let Some(ref repo) = cli.repo {
        config.repo_url.clone_from(repo);
    }
    if let Some(ref local_path) = cli.local_path {
        config.local_path.clone_from(local_path);
    }
    if let Some(ref providers) = cli.providers {
        config.providers.clone_from(providers);
    }
    config.branch.clone_from(&cli.branch);

    Ok(config)
}

async fn init_config() -> Result<(), sync_auth::SyncError> {
    let path = SyncConfig::default_config_path();
    if path.exists() {
        println!("Config already exists at: {}", path.display());
        return Ok(());
    }

    let template = r#"# sync-auth configuration
# See https://github.com/link-foundation/auth-sync for documentation

# Git repository URL (required)
repo_url = ""

# Providers to sync (empty = all)
# providers = ["gh", "claude", "glab"]

# Git branch
branch = "main"

# Use shallow clone
shallow_clone = true

# Watch mode interval (seconds)
watch_interval_secs = 60
"#;

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&path, template).await?;
    println!("Created config at: {}", path.display());
    Ok(())
}

async fn handle_daemon(
    action: DaemonAction,
    repo: Option<&str>,
    config: Option<&std::path::Path>,
) -> Result<(), sync_auth::SyncError> {
    let pid_path = dirs::runtime_dir()
        .or_else(dirs::data_local_dir)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("sync-auth.pid");

    match action {
        DaemonAction::Start { interval } => {
            if pid_path.exists() {
                let pid = tokio::fs::read_to_string(&pid_path).await?;
                println!("Daemon may already be running (PID: {})", pid.trim());
                return Ok(());
            }

            println!("Starting sync-auth daemon (interval: {interval}s)...");
            let exe =
                std::env::current_exe().map_err(|e| sync_auth::SyncError::Config(e.to_string()))?;
            let mut cmd = tokio::process::Command::new(exe);
            if let Some(repo) = repo {
                cmd.args(["--repo", repo]);
            }
            if let Some(config) = config {
                cmd.args(["--config", &config.to_string_lossy()]);
            }
            cmd.args(["watch", "--interval", &interval.to_string()]);
            cmd.stdin(std::process::Stdio::null());
            cmd.stdout(std::process::Stdio::null());
            cmd.stderr(std::process::Stdio::null());

            let child = cmd.spawn().map_err(|e| {
                sync_auth::SyncError::Config(format!("failed to spawn daemon: {e}"))
            })?;
            let pid = child.id().unwrap_or(0);

            if let Some(parent) = pid_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::write(&pid_path, pid.to_string()).await?;
            println!("Daemon started (PID: {pid})");
        }
        DaemonAction::Stop => {
            if !pid_path.exists() {
                println!("No daemon PID file found.");
                return Ok(());
            }
            let pid_str = tokio::fs::read_to_string(&pid_path).await?;
            let pid: u32 = pid_str
                .trim()
                .parse()
                .map_err(|e| sync_auth::SyncError::Config(format!("invalid PID: {e}")))?;

            // Send SIGTERM via kill command
            let _ = tokio::process::Command::new("kill")
                .arg(pid.to_string())
                .output()
                .await;
            tokio::fs::remove_file(&pid_path).await?;
            println!("Daemon stopped (PID: {pid})");
        }
        DaemonAction::Restart { interval } => {
            let stop = DaemonAction::Stop;
            let _ = Box::pin(handle_daemon(stop, repo, config)).await;
            let start = DaemonAction::Start { interval };
            Box::pin(handle_daemon(start, repo, config)).await?;
        }
        DaemonAction::Setup => {
            print_systemd_unit()?;
        }
    }
    Ok(())
}

fn print_systemd_unit() -> Result<(), sync_auth::SyncError> {
    let exe = std::env::current_exe().map_err(|e| sync_auth::SyncError::Config(e.to_string()))?;

    let unit = format!(
        r"[Unit]
Description=sync-auth credential sync daemon
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart={exe} watch --interval 60
Restart=on-failure
RestartSec=10

[Install]
WantedBy=default.target
",
        exe = exe.display()
    );

    let service_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("systemd")
        .join("user");

    println!("Systemd user service unit:\n");
    println!("{unit}");
    println!("To install, save this to:");
    println!("  {}/sync-auth.service", service_dir.display());
    println!("\nThen run:");
    println!("  systemctl --user daemon-reload");
    println!("  systemctl --user enable --now sync-auth");

    Ok(())
}
