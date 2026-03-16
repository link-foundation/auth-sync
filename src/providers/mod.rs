//! Built-in auth providers for common dev tools.
//!
//! Each provider knows where its tool stores credentials on disk and can
//! optionally validate them. Add a new provider by implementing [`AuthProvider`].

mod claude;
mod codex;
mod gemini;
mod gh;
mod glab;
mod opencode;
mod qwen_coder;

pub use self::claude::ClaudeProvider;
pub use self::codex::CodexProvider;
pub use self::gemini::GeminiProvider;
pub use self::gh::GhProvider;
pub use self::glab::GlabProvider;
pub use self::opencode::OpencodeProvider;
pub use self::qwen_coder::QwenCoderProvider;

use crate::AuthProvider;

/// Returns all built-in providers.
pub fn all_providers() -> Vec<Box<dyn AuthProvider>> {
    vec![
        Box::new(GhProvider),
        Box::new(GlabProvider),
        Box::new(ClaudeProvider),
        Box::new(CodexProvider),
        Box::new(GeminiProvider),
        Box::new(OpencodeProvider),
        Box::new(QwenCoderProvider),
    ]
}

/// Returns a single built-in provider by name, or `None` if unknown.
pub fn provider_by_name(name: &str) -> Option<Box<dyn AuthProvider>> {
    match name {
        "gh" => Some(Box::new(GhProvider)),
        "glab" => Some(Box::new(GlabProvider)),
        "claude" => Some(Box::new(ClaudeProvider)),
        "codex" => Some(Box::new(CodexProvider)),
        "gemini" => Some(Box::new(GeminiProvider)),
        "opencode" => Some(Box::new(OpencodeProvider)),
        "qwen-coder" => Some(Box::new(QwenCoderProvider)),
        _ => None,
    }
}
