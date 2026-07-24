//! The generation → validate → repair loop (IMPLEMENTATION_PLAN §4.4).
//!
//! A prompt goes to the model; its reply is stripped to the `.wf` source (prose
//! discarded — FR-6); the source is parsed + semantically validated through the
//! same [`compile_source`] gate the preview uses; and on a compiler diagnostic
//! the model is re-prompted with the exact error, bounded by `max_attempts`.
//! Reliability comes from this harness, not the model (§4.4) — no model has seen
//! WebFluent, so every output is validated before it can reach the canvas.
//!
//! GPUI-free and synchronous: [`collect_text`] blocks until the stream ends, so
//! the studio runs [`generate_page`] off the UI thread. Deterministically testable
//! against [`wf_ai::ScriptedProvider`].

use wf_ai::{collect_text, ChatMessage, ChatRequest, Provider};

use crate::compile_source;

/// Per-request generation knobs.
#[derive(Debug, Clone)]
pub struct GenConfig {
    /// The provider model id (e.g. `claude-opus-4-8`).
    pub model: String,
    pub max_tokens: u32,
    /// Total model calls before giving up (the first try plus repairs).
    pub max_attempts: usize,
}

impl GenConfig {
    /// Sensible defaults for a model: 4k tokens, 3 attempts (§4.6 default).
    pub fn for_model(model: impl Into<String>) -> Self {
        Self { model: model.into(), max_tokens: 4096, max_attempts: 3 }
    }
}

/// A successful generation: the validated `.wf` source and how many model calls
/// it took (1 = first try, >1 = self-heal rounds).
#[derive(Debug, Clone, PartialEq)]
pub struct GenOutcome {
    pub source: String,
    pub attempts: usize,
}

/// Why a generation did not produce validated `.wf`.
#[derive(Debug, Clone, PartialEq)]
pub enum GenError {
    /// The provider/transport failed (terminal — never retried here).
    Provider(String),
    /// The reply contained no extractable `.wf` (terminal).
    NoWfBlock,
    /// Still did not compile after `attempts` tries; `last_error` is the final diagnostic.
    Unrepaired { last_error: String, attempts: usize },
}

impl std::fmt::Display for GenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenError::Provider(e) => write!(f, "the AI provider failed: {e}"),
            GenError::NoWfBlock => write!(f, "the AI reply contained no WebFluent to compile"),
            GenError::Unrepaired { last_error, attempts } => {
                write!(f, "could not produce a compiling page after {attempts} attempts (last error: {last_error})")
            }
        }
    }
}

impl std::error::Error for GenError {}

/// Generate one validated `.wf` page from a prompt, re-prompting with the compiler
/// diagnostic on each failure until it compiles or `max_attempts` is spent.
pub fn generate_page(
    provider: &dyn Provider,
    system_prompt: &str,
    user_prompt: &str,
    config: &GenConfig,
) -> Result<GenOutcome, GenError> {
    let mut messages = vec![ChatMessage::system(system_prompt), ChatMessage::user(user_prompt)];
    let mut attempts = 0;

    loop {
        attempts += 1;
        let request = ChatRequest {
            model: config.model.clone(),
            messages: messages.clone(),
            max_tokens: config.max_tokens,
        };
        let reply = collect_text(provider.stream_chat(request)).map_err(GenError::Provider)?;
        let wf = extract_wf(&reply).ok_or(GenError::NoWfBlock)?;

        match compile_source(&wf) {
            Ok(_) => return Ok(GenOutcome { source: wf, attempts }),
            Err(diagnostic) => {
                let last_error = diagnostic.to_string();
                if attempts >= config.max_attempts {
                    return Err(GenError::Unrepaired { last_error, attempts });
                }
                // Feed the failed attempt + the exact diagnostic back for a repair.
                messages.push(ChatMessage::assistant(reply));
                messages.push(ChatMessage::user(format!(
                    "That WebFluent did not compile:\n{last_error}\n\
                     Return the corrected, complete page in a single ```wf code block.",
                )));
            }
        }
    }
}

/// Extract `.wf` source from a model reply: the first fenced code block (```wf
/// or a bare fence), else the whole reply if it already looks like WebFluent.
fn extract_wf(text: &str) -> Option<String> {
    if let Some(block) = extract_fenced(text) {
        return Some(block);
    }
    let t = text.trim();
    let looks_like_wf = t.starts_with("Page ")
        || t.starts_with("Component ")
        || t.starts_with("App ")
        || t.starts_with("Store ");
    looks_like_wf.then(|| t.to_string())
}

/// The contents of the first ``` fenced block, skipping the opening fence line
/// (which may carry a language tag like `wf`). `None` if there is no closed fence.
fn extract_fenced(text: &str) -> Option<String> {
    let start = text.find("```")?;
    let after = &text[start + 3..];
    let body = &after[after.find('\n').map(|i| i + 1)?..];
    let end = body.find("```")?;
    let block = body[..end].trim();
    (!block.is_empty()).then(|| block.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wf_ai::{ChatDelta, ProviderKind, ScriptedProvider};

    const GOOD: &str = "```wf\nPage Home (path: \"/\") { Container { Text(\"hi\") } }\n```";
    // Parses but references an undeclared component → the semantic gate rejects it.
    const BAD: &str = "```wf\nPage Home (path: \"/\") { Ghost() }\n```";

    fn cfg() -> GenConfig {
        GenConfig::for_model("test")
    }

    #[test]
    fn generates_valid_wf_on_the_first_try() {
        let p = ScriptedProvider::with_text(GOOD);
        let out = generate_page(&p, "sys", "make a page", &cfg()).unwrap();
        assert_eq!(out.attempts, 1);
        assert!(out.source.contains("Page Home"));
    }

    #[test]
    fn repairs_after_a_bad_attempt() {
        let p = ScriptedProvider::new(ProviderKind::Anthropic);
        p.push_text(BAD).push_text(GOOD); // bad, then good
        let out = generate_page(&p, "sys", "make a page", &cfg()).unwrap();
        assert_eq!(out.attempts, 2, "one repair round");
        assert!(out.source.contains("Text(\"hi\")"));
    }

    #[test]
    fn gives_up_after_max_attempts() {
        let p = ScriptedProvider::new(ProviderKind::Anthropic);
        p.push_text(BAD).push_text(BAD).push_text(BAD);
        let err = generate_page(&p, "sys", "x", &cfg()).unwrap_err();
        match err {
            GenError::Unrepaired { attempts, .. } => assert_eq!(attempts, 3),
            other => panic!("expected Unrepaired, got {other:?}"),
        }
    }

    #[test]
    fn a_provider_error_is_terminal_and_not_repaired() {
        let p = ScriptedProvider::new(ProviderKind::Anthropic);
        p.push_deltas(vec![ChatDelta::Error("429 rate limited".into())]);
        assert!(matches!(generate_page(&p, "s", "u", &cfg()), Err(GenError::Provider(_))));
    }

    #[test]
    fn a_reply_without_wf_is_rejected() {
        let p = ScriptedProvider::with_text("Sure! I can help with that.");
        assert_eq!(generate_page(&p, "s", "u", &cfg()), Err(GenError::NoWfBlock));
    }

    #[test]
    fn extract_prefers_a_wf_fence() {
        let got = extract_wf("Here you go:\n```wf\nPage A (path: \"/\") { Text(\"x\") }\n```\nEnjoy!");
        assert_eq!(got.as_deref(), Some("Page A (path: \"/\") { Text(\"x\") }"));
    }

    #[test]
    fn extract_accepts_a_bare_fence() {
        let got = extract_wf("```\nPage A (path: \"/\") {}\n```");
        assert_eq!(got.as_deref(), Some("Page A (path: \"/\") {}"));
    }

    #[test]
    fn extract_accepts_unfenced_source() {
        let got = extract_wf("Page Home (path: \"/\") { Text(\"hi\") }");
        assert!(got.unwrap().starts_with("Page Home"));
    }

    #[test]
    fn extract_rejects_prose() {
        assert_eq!(extract_wf("I'm not sure what you mean."), None);
    }
}
