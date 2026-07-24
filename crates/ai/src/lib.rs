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
use tracing::{debug, error, trace, warn};

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
    /// Whether the adapter should send `thinking:{type:"disabled"}` for this model.
    /// True only for models that reason by DEFAULT *and* let you turn it off (Claude
    /// Sonnet 5, DeepSeek V4, GLM auto-deciders, Kimi K2.6) — the studio never
    /// surfaces a chain-of-thought, and left on it burns the token budget (emptying
    /// the connection test, truncating generations). False for models that never
    /// think (so the param is unnecessary) and for compulsory-reasoning models
    /// (Fable 5, GLM-4.7, Kimi K3, o-series) that would reject or ignore it.
    pub disable_thinking: bool,
}

impl ModelInfo {
    const fn new(id: &'static str, name: &'static str, disable_thinking: bool) -> Self {
        Self { id, name, disable_thinking }
    }

    /// The model in `models` matching `id`, if any — used by the adapters to decide
    /// whether to send the thinking-disable param.
    pub fn lookup(kind: ProviderKind, id: &str) -> Option<ModelInfo> {
        kind.models().iter().copied().find(|m| m.id == id)
    }
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

    /// Built-in API base, without the endpoint path. Anthropic appends
    /// `/messages`; OpenAI-compatible providers append `/chat/completions`.
    /// Prefer [`resolved_base_url`] at call time so overrides apply.
    pub fn base_url(self) -> &'static str {
        match self {
            ProviderKind::Anthropic => "https://api.anthropic.com/v1",
            ProviderKind::OpenAi => "https://api.openai.com/v1",
            ProviderKind::DeepSeek => "https://api.deepseek.com/v1",
            ProviderKind::Kimi => "https://api.moonshot.ai/v1",
            // z.ai is the international GLM platform (its own catalog: glm-4.6, glm-5.2…);
            // mainland-China keys use https://open.bigmodel.cn/api/paas/v4 with a
            // different catalog — override with WF_ZHIPU_BASE_URL (see resolved_base_url).
            ProviderKind::Glm => "https://api.z.ai/api/paas/v4",
            ProviderKind::Gemini => "https://generativelanguage.googleapis.com/v1beta/openai",
        }
    }

    /// The effective API base: an optional per-provider environment override,
    /// `WF_<SLUG>_BASE_URL` (slug upper-cased — e.g. `WF_ZHIPU_BASE_URL`), else the
    /// built-in [`base_url`]. The override lets international GLM users target
    /// `api.z.ai`, and enables corporate proxies or self-hosted OpenAI-compatible
    /// gateways, without a rebuild. A Settings field is the eventual UI for this.
    pub fn resolved_base_url(self) -> String {
        let var = format!("WF_{}_BASE_URL", self.slug().to_uppercase());
        std::env::var(&var)
            .ok()
            .map(|s| s.trim().trim_end_matches('/').to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.base_url().to_string())
    }

    /// The recommended default model — the first of [`models`]. Shown pre-selected;
    /// the eval harness (§4.4) refines these with evidence in M5.
    pub fn default_model(self) -> &'static str {
        self.models()[0].id
    }

    /// The user-selectable models for this provider, recommended default first.
    /// Catalogs verified against each provider's live docs on 2026-07-24; the `M`
    /// helper's third arg is `disable_thinking` (see [`ModelInfo`]). Model IDs move
    /// fast — the eval pass (§4.4) re-confirms these against real keys.
    pub fn models(self) -> &'static [ModelInfo] {
        use ModelInfo as M;
        match self {
            // Opus 4.8 & Haiku 4.5 don't think unless asked (no param needed); Sonnet 5
            // reasons by default and is turned off; Fable 5 reasons compulsorily (can't).
            ProviderKind::Anthropic => {
                const L: &[ModelInfo] = &[
                    M::new("claude-opus-4-8", "Claude Opus 4.8", false),
                    M::new("claude-sonnet-5", "Claude Sonnet 5", true),
                    M::new("claude-haiku-4-5", "Claude Haiku 4.5", false),
                    M::new("claude-fable-5", "Claude Fable 5", false),
                ];
                L
            }
            // GPT-5.6/5.5/5.4 chat models default reasoning OFF; the -pro tiers and the
            // o-series reason by default and can't be disabled via a thinking param.
            ProviderKind::OpenAi => {
                const L: &[ModelInfo] = &[
                    M::new("gpt-5.6-terra", "GPT-5.6 Terra", false),
                    M::new("gpt-5.6-sol", "GPT-5.6 Sol", false),
                    M::new("gpt-5.6-luna", "GPT-5.6 Luna", false),
                    M::new("gpt-5.5", "GPT-5.5", false),
                    M::new("gpt-5.5-pro", "GPT-5.5 Pro", false),
                    M::new("gpt-5.4", "GPT-5.4", false),
                    M::new("gpt-5.4-mini", "GPT-5.4 mini", false),
                    M::new("gpt-5.4-nano", "GPT-5.4 nano", false),
                    M::new("gpt-5.4-pro", "GPT-5.4 Pro", false),
                    M::new("gpt-4.1", "GPT-4.1", false),
                    M::new("gpt-4.1-mini", "GPT-4.1 mini", false),
                    M::new("gpt-4.1-nano", "GPT-4.1 nano", false),
                    M::new("gpt-4o", "GPT-4o", false),
                    M::new("gpt-4o-mini", "GPT-4o mini", false),
                    M::new("o3", "o3", false),
                    M::new("o4-mini", "o4-mini", false),
                ];
                L
            }
            // Gemini's OpenAI-compat endpoint keeps any chain-of-thought OUT of
            // delta.content, so the final answer arrives regardless; thinking is
            // controlled by reasoning_effort (not a thinking param), so leave it unset.
            ProviderKind::Gemini => {
                const L: &[ModelInfo] = &[
                    M::new("gemini-3.5-flash-lite", "Gemini 3.5 Flash-Lite", false),
                    M::new("gemini-3.6-flash", "Gemini 3.6 Flash", false),
                    M::new("gemini-3.5-flash", "Gemini 3.5 Flash", false),
                    M::new("gemini-2.5-pro", "Gemini 2.5 Pro", false),
                    M::new("gemini-2.5-flash", "Gemini 2.5 Flash", false),
                    M::new("gemini-3.1-flash-lite", "Gemini 3.1 Flash-Lite", false),
                    M::new("gemini-2.5-flash-lite", "Gemini 2.5 Flash-Lite", false),
                ];
                L
            }
            // Both V4 models default thinking ON and support disabling it.
            ProviderKind::DeepSeek => {
                const L: &[ModelInfo] = &[
                    M::new("deepseek-v4-flash", "DeepSeek V4 Flash", true),
                    M::new("deepseek-v4-pro", "DeepSeek V4 Pro", true),
                ];
                L
            }
            // Only K2.6 supports disabling thinking; K2.7-code & K3 reason compulsorily.
            ProviderKind::Kimi => {
                const L: &[ModelInfo] = &[
                    M::new("kimi-k2.6", "Kimi K2.6", true),
                    M::new("kimi-k2.7-code", "Kimi K2.7 Code", false),
                    M::new("kimi-k2.7-code-highspeed", "Kimi K2.7 Code (High-Speed)", false),
                    M::new("kimi-k3", "Kimi K3", false),
                ];
                L
            }
            // z.ai catalog. GLM-4.5-series and up auto-decide thinking (disable-able);
            // the 4.7 family reasons compulsorily; the legacy 4-32B has no thinking.
            ProviderKind::Glm => {
                const L: &[ModelInfo] = &[
                    M::new("glm-4.6", "GLM-4.6", true),
                    M::new("glm-5.2", "GLM-5.2", true),
                    M::new("glm-4.5-air", "GLM-4.5-Air", true),
                    M::new("glm-5.1", "GLM-5.1", true),
                    M::new("glm-5", "GLM-5", true),
                    M::new("glm-5-turbo", "GLM-5-Turbo", true),
                    M::new("glm-4.5", "GLM-4.5", true),
                    M::new("glm-4.5-x", "GLM-4.5-X", true),
                    M::new("glm-4.5-airx", "GLM-4.5-AirX", true),
                    M::new("glm-4.5-flash", "GLM-4.5-Flash", true),
                    M::new("glm-4.7", "GLM-4.7", false),
                    M::new("glm-4.7-flashx", "GLM-4.7-FlashX", false),
                    M::new("glm-4.7-flash", "GLM-4.7-Flash", false),
                    M::new("glm-4-32b-0414-128k", "GLM-4-32B (0414)", false),
                ];
                L
            }
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
    let wire = kind.wire();
    debug!(provider = ?kind, wire = ?wire, "building provider adapter");
    match wire {
        Wire::Anthropic => Box::new(AnthropicAdapter::new(api_key)),
        Wire::OpenAi => Box::new(OpenAiCompatAdapter::new(kind, api_key)),
    }
}

/// A length-capped, panic-safe preview of `s` for logging. Truncates on a char
/// boundary (never mid-byte, so multi-byte UTF-8 is safe) and appends `…` when
/// the text was cut. Used to log prompts/responses at `debug` without dumping
/// the full (potentially huge) text — that goes to `trace`.
pub(crate) fn preview(s: &str, max: usize) -> String {
    match s.char_indices().nth(max) {
        Some((idx, _)) => format!("{}…", &s[..idx]),
        None => s.to_string(),
    }
}

/// Render a chat prompt to a single string for logging — one `role: content`
/// line per message. Diagnostics only; never sent on the wire.
pub(crate) fn render_prompt(messages: &[ChatMessage]) -> String {
    let mut out = String::new();
    for m in messages {
        let role = match m.role {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
        };
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(role);
        out.push_str(": ");
        out.push_str(&m.content);
    }
    out
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
            Ok(ChatDelta::Done) => {
                if out.is_empty() {
                    warn!("chat stream completed with an empty response");
                } else {
                    debug!(len = out.len(), preview = %preview(&out, 400), "collected chat response");
                }
                trace!(response = %out, "full chat response");
                return Ok(out);
            }
            Ok(ChatDelta::Error(e)) => {
                error!(error = %e, "chat stream failed");
                return Err(e);
            }
            Err(_) => {
                debug!(len = out.len(), "chat stream closed without a terminal delta");
                return Ok(out); // channel closed without a terminal delta
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Guard the hand-maintained model tables: every provider must offer at least
    /// one model, ids must be unique and non-blank, the default must be the first
    /// entry and resolve via lookup, and only real entries carry `disable_thinking`.
    #[test]
    fn model_tables_are_well_formed() {
        for kind in ProviderKind::ALL {
            let models = kind.models();
            assert!(!models.is_empty(), "{kind:?} has no models");

            // default_model() is models()[0] and looks up.
            let default = kind.default_model();
            assert_eq!(default, models[0].id, "{kind:?} default is not the first model");
            let looked_up = ModelInfo::lookup(kind, default);
            assert!(looked_up.is_some(), "{kind:?} default {default} does not resolve");

            let mut seen = std::collections::HashSet::new();
            for m in models {
                assert!(!m.id.trim().is_empty(), "{kind:?} has a blank model id");
                assert!(!m.name.trim().is_empty(), "{kind:?} model {} has a blank name", m.id);
                assert!(seen.insert(m.id), "{kind:?} has a duplicate model id: {}", m.id);
            }

            // A model unknown to the provider never resolves (so no stray thinking param).
            assert!(ModelInfo::lookup(kind, "nope-not-a-model").is_none());
        }
    }
}
