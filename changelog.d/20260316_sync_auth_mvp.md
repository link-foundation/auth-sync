### Added
- Complete sync-auth MVP: bidirectional auth credential sync for dev tools via Git repositories
- Library crate with `AuthProvider` and `GitBackend` traits for extensibility
- 7 built-in providers: gh, glab, claude, codex, gemini, opencode, qwen-coder
- CLI binary with subcommands: pull, push, sync, watch, status, providers, init, daemon
- Shallow clone support for fast initial setup
- Conflict resolution that skips expired credentials
- Watch mode with configurable sync interval
- Daemon management (start/stop/restart/systemd setup)
- TOML config file support with CLI and environment variable overrides
- Comprehensive test suite (unit + integration tests)
- Docker and CI/CD usage examples in README

### Changed
- Package renamed from `my-package` to `sync-auth`
- Complete rewrite of library and binary for credential sync functionality
