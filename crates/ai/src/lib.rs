//! wf-ai — AI provider abstraction for WebFluent Studio (BYOK).
//!
//! Six launch providers behind one [`Provider`] trait. All but Anthropic speak
//! the OpenAI chat-completions protocol (Gemini via its compatibility
//! endpoint), so there are only two adapters — [`anthropic`] and [`openai`] —
//! selected by [`ProviderKind::wire`]. See IMPLEMENTATION_PLAN §4.3.
//!
//! GPUI-free: `stream_chat` returns an [`async_channel::Receiver`] the studio
//! drains from its executor; each call drives reqwest on a dedicated thread
//! (see [`stream`]) so this crate is runtime-agnostic for callers.

mod anthropic;
mod keystore;
mod openai;
mod prompt;
mod scripted;
mod sse;
mod stream;

use serde::{Deserialize, Serialize};

pub use anthropic::AnthropicAdapter;
pub use keystore::{default_key_store, ChainKeyStore, EnvKeyStore, InMemoryKeyStore, KeyStore, KeyringStore};
pub use openai::OpenAiCompatAdapter;
pub use prompt::LANGUAGE_CARD;
pub use scripted::ScriptedProvider;

/// The six launch providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProviderKind {
    Anthropic,
    OpenAi,
    Gemini,
    DeepSeek,
    Kimi,
    Glm,
}

/// Which wire protocol a provider speaks. Anthropic has its own Messages API;
/// everyone else is OpenAI chat-completions-compatible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wire {
    Anthropic,
    OpenAi,
}

/// A selectable model for a provider: its API id and a display name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelInfo {
    pub id: &'static str,
    pub name: &'static str,
}

impl ProviderKind {
    pub const ALL: [ProviderKind; 6] = [
        ProviderKind::Anthropic,
        ProviderKind::OpenAi,
        ProviderKind::Gemini,
        ProviderKind::DeepSeek,
        ProviderKind::Kimi,
        ProviderKind::Glm,
    ];

    pub fn wire(self) -> Wire {
        match self {
            ProviderKind::Anthropic => Wire::Anthropic,
            _ => Wire::OpenAi,
        }
    }

    /// API base, without the endpoint path. Anthropic appends `/messages`;
    /// OpenAI-compatible providers append `/chat/completions`.
    pub fn base_url(self) -> &'static str {
        match self {
            ProviderKind::Anthropic => "https://api.anthropic.com/v1",
            ProviderKind::OpenAi => "https://api.openai.com/v1",
            ProviderKind::DeepSeek => "https://api.deepseek.com/v1",
            ProviderKind::Kimi => "https://api.moonshot.ai/v1",
            ProviderKind::Glm => "https://open.bigmodel.cn/api/paas/v4",
            ProviderKind::Gemini => "https://generativelanguage.googleapis.com/v1beta/openai",
        }
    }

    /// The recommended default model — the first of [`models`]. Shown pre-selected;
    /// the eval harness (§4.4) refines these with evidence in M5.
    pub fn default_model(self) -> &'static str {
        self.models()[0].id
    }

    /// The user-selectable models for this provider, recommended default first.
    pub fn models(self) -> &'static [ModelInfo] {
        match self {
            ProviderKind::Anthropic => &[
                ModelInfo { id: "claude-opus-4-8", name: "Claude Opus 4.8" },
                ModelInfo { id: "claude-sonnet-5", name: "Claude Sonnet 5" },
                ModelInfo { id: "claude-haiku-4-5-20251001", name: "Claude Haiku 4.5" },
            ],
            ProviderKind::OpenAi => &[
                ModelInfo { id: "gpt-4o", name: "GPT-4o" },
                ModelInfo { id: "gpt-4o-mini", name: "GPT-4o mini" },
            ],
            ProviderKind::Gemini => &[
                ModelInfo { id: "gemini-2.0-flash", name: "Gemini 2.0 Flash" },
                ModelInfo { id: "gemini-1.5-pro", name: "Gemini 1.5 Pro" },
            ],
            ProviderKind::DeepSeek => &[
                ModelInfo { id: "deepseek-chat", name: "DeepSeek Chat" },
                ModelInfo { id: "deepseek-reasoner", name: "DeepSeek Reasoner" },
            ],
            ProviderKind::Kimi => &[
                ModelInfo { id: "moonshot-v1-32k", name: "Moonshot v1 (32k)" },
                ModelInfo { id: "moonshot-v1-128k", name: "Moonshot v1 (128k)" },
            ],
            ProviderKind::Glm => &[
                ModelInfo { id: "glm-4", name: "GLM-4" },
                ModelInfo { id: "glm-4-plus", name: "GLM-4 Plus" },
            ],
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            ProviderKind::Anthropic => "Anthropic (Claude)",
            ProviderKind::OpenAi => "OpenAI",
            ProviderKind::Gemini => "Google Gemini",
            ProviderKind::DeepSeek => "DeepSeek",
            ProviderKind::Kimi => "Moonshot (Kimi)",
            ProviderKind::Glm => "Zhipu (GLM)",
        }
    }

    /// Stable slug — the keyring account and config key for this provider.
    pub fn slug(self) -> &'static str {
        match self {
            ProviderKind::Anthropic => "anthropic",
            ProviderKind::OpenAi => "openai",
            ProviderKind::Gemini => "gemini",
            ProviderKind::DeepSeek => "deepseek",
            ProviderKind::Kimi => "moonshot",
            ProviderKind::Glm => "zhipu",
        }
    }

    /// The conventional environment variable holding this provider's key — the
    /// env fallback when nothing is in the OS keychain (dev/CI).
    pub fn key_env(self) -> &'static str {
        match self {
            ProviderKind::Anthropic => "ANTHROPIC_API_KEY",
            ProviderKind::OpenAi => "OPENAI_API_KEY",
            ProviderKind::Gemini => "GEMINI_API_KEY",
            ProviderKind::DeepSeek => "DEEPSEEK_API_KEY",
            ProviderKind::Kimi => "MOONSHOT_API_KEY",
            ProviderKind::Glm => "ZHIPU_API_KEY",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
        }
    }
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub max_tokens: u32,
}

/// One streamed increment of a chat response. A stream always ends with exactly
/// one terminal delta — [`ChatDelta::Done`] on success or [`ChatDelta::Error`].
#[derive(Debug, Clone, PartialEq)]
pub enum ChatDelta {
    Text(String),
    Done,
    Error(String),
}

/// A chat-capable model provider. Implementations stream deltas into the
/// returned channel; the studio consumes them from async context.
pub trait Provider: Send + Sync {
    fn kind(&self) -> ProviderKind;
    fn stream_chat(&self, req: ChatRequest) -> async_channel::Receiver<ChatDelta>;
}

/// Build the right adapter for a provider + key. The model comes from the
/// [`ChatRequest`], so one adapter serves every model that provider offers.
pub fn provider_for(kind: ProviderKind, api_key: String) -> Box<dyn Provider> {
    match kind.wire() {
        Wire::Anthropic => Box::new(AnthropicAdapter::new(api_key)),
        Wire::OpenAi => Box::new(OpenAiCompatAdapter::new(kind, api_key)),
    }
}

/// Drain a chat stream to its full text (concatenated [`ChatDelta::Text`]), or the
/// first [`ChatDelta::Error`]. Blocks the calling thread until the stream ends, so
/// run it off the UI thread — the stream itself is driven on its own thread (see
/// [`stream`]). A stream that closes without an explicit terminal delta yields the
/// text accumulated so far.
pub fn collect_text(rx: async_channel::Receiver<ChatDelta>) -> Result<String, String> {
    let mut out = String::new();
    loop {
        match rx.recv_blocking() {
            Ok(ChatDelta::Text(t)) => out.push_str(&t),
            Ok(ChatDelta::Done) => return Ok(out),
            Ok(ChatDelta::Error(e)) => return Err(e),
            Err(_) => return Ok(out), // channel closed without a terminal delta
        }
    }
}
