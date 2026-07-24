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
mod openai;
mod scripted;
mod sse;
mod stream;

use serde::{Deserialize, Serialize};

pub use anthropic::AnthropicAdapter;
pub use openai::OpenAiCompatAdapter;
pub use scripted::ScriptedProvider;

/// The six launch providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

    /// Sensible default model, shown pre-selected in settings (user-overridable,
    /// PBRD §4.1). The eval harness (§4.4) picks the recommended defaults with
    /// evidence in M5; these are the reasonable-out-of-the-box choices.
    pub fn default_model(self) -> &'static str {
        match self {
            ProviderKind::Anthropic => "claude-opus-4-8",
            ProviderKind::OpenAi => "gpt-4o",
            ProviderKind::DeepSeek => "deepseek-chat",
            ProviderKind::Kimi => "moonshot-v1-32k",
            ProviderKind::Glm => "glm-4",
            ProviderKind::Gemini => "gemini-2.0-flash",
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
