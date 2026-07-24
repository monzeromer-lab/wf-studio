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

use wf_ai::{collect_text, ChatMessage, ChatRequest, Provider, LANGUAGE_CARD};
use webfluent::{apply_edits, EditOp};

use crate::{compile_merged, compile_source};

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

/// A successful scoped edit: the new full source (base with the node replaced),
/// and how many model calls it took.
#[derive(Debug, Clone, PartialEq)]
pub struct EditOutcome {
    pub source: String,
    pub attempts: usize,
}

/// Edit one selected element by instruction: show the model the element's current
/// source, take its replacement, splice it back in place of that node, and
/// validate the whole page — re-prompting with the diagnostic on failure. The
/// model only ever sees the selected element (context pruning, §4.3/NFR-2), never
/// the whole file, and can only change that node's span (scoped-edit containment).
pub fn edit_node(
    provider: &dyn Provider,
    base_source: &str,
    node_id: &str,
    instruction: &str,
    config: &GenConfig,
) -> Result<EditOutcome, GenError> {
    // The base must compile so we can resolve the selected node's current source.
    let (site, merged, _ranges) = compile_merged([("<edit>", base_source)])
        .map_err(|e| GenError::Provider(format!("the page must compile before editing: {e}")))?;
    let info = site
        .node_map
        .info(node_id)
        .ok_or_else(|| GenError::Provider(format!("unknown node {node_id}")))?;
    let node_src = info.span.slice(&merged).to_string();

    let system = format!(
        "{LANGUAGE_CARD}\n\n# EDIT MODE\nYou are editing ONE existing element inside a page. \
         Return ONLY the replacement for that element — a single WebFluent element (with its \
         children and style, if any) — inside one ```wf block. Do NOT wrap it in a Page or \
         Component, and do NOT change anything the request did not ask you to."
    );
    let mut messages = vec![
        ChatMessage::system(system),
        ChatMessage::user(format!("Current element:\n```wf\n{node_src}\n```\n\nEdit request: {instruction}")),
    ];
    let mut attempts = 0;

    loop {
        attempts += 1;
        let request = ChatRequest {
            model: config.model.clone(),
            messages: messages.clone(),
            max_tokens: config.max_tokens,
        };
        let reply = collect_text(provider.stream_chat(request)).map_err(GenError::Provider)?;
        let replacement = extract_wf(&reply).ok_or(GenError::NoWfBlock)?;

        // Splice the replacement in place of the node (a scoped span edit), then
        // validate the whole page. Both the reparse-guard and the semantic gate
        // are compiler diagnostics we feed back for a repair.
        let spliced = apply_edits(base_source, &[EditOp::ReplaceNode { node: node_id.to_string(), wf: replacement }])
            .map_err(|e| e.to_string())
            .and_then(|s| compile_source(&s).map(|_| s).map_err(|e| e.to_string()));

        match spliced {
            Ok(source) => return Ok(EditOutcome { source, attempts }),
            Err(last_error) => {
                if attempts >= config.max_attempts {
                    return Err(GenError::Unrepaired { last_error, attempts });
                }
                messages.push(ChatMessage::assistant(reply));
                messages.push(ChatMessage::user(format!(
                    "That replacement did not work: {last_error}\n\
                     Return a corrected single element in one ```wf block.",
                )));
            }
        }
    }
}

/// A successful self-heal: the corrected source + the number of model calls.
#[derive(Debug, Clone, PartialEq)]
pub struct HealOutcome {
    pub source: String,
    pub attempts: usize,
}

/// Why a self-heal did not produce an acceptable fix.
#[derive(Debug, Clone, PartialEq)]
pub enum HealError {
    Provider(String),
    NoWfBlock,
    /// No compiling, design-preserving fix within the budget; `last_reason` is why
    /// the final attempt was rejected.
    GaveUp { last_reason: String, attempts: usize },
}

impl std::fmt::Display for HealError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealError::Provider(e) => write!(f, "the AI provider failed: {e}"),
            HealError::NoWfBlock => write!(f, "the AI reply contained no WebFluent"),
            HealError::GaveUp { last_reason, attempts } => {
                write!(f, "couldn't fix it without changing the design after {attempts} attempts ({last_reason})")
            }
        }
    }
}

impl std::error::Error for HealError {}

/// Repair a runtime error in a *compiling* page (FR-19–22 / §4.6). The model is
/// asked to fix only the cause; each candidate must compile AND pass the
/// **design-freeze**: it may not change anything visible (no `Text`/`Style`/
/// `Structure` diff — only invisible logic like state/actions/handlers). A fix
/// that alters the design is rejected and re-prompted — the AST paying rent, not
/// a prompt-hope. Bounded by `max_attempts`.
pub fn self_heal(
    provider: &dyn Provider,
    base_source: &str,
    runtime_error: &str,
    config: &GenConfig,
) -> Result<HealOutcome, HealError> {
    let system = format!(
        "{LANGUAGE_CARD}\n\n# FIX MODE\nA compiled page threw a runtime error. Return the \
         corrected, COMPLETE page in one ```wf block. Fix ONLY the cause of the error — do NOT \
         change any visible text, layout, styling, modifiers, or components. Change only logic: \
         state, derived, actions, event handlers, and expressions."
    );
    let mut messages = vec![
        ChatMessage::system(system),
        ChatMessage::user(format!("The page:\n```wf\n{base_source}\n```\n\nRuntime error:\n{runtime_error}")),
    ];
    let mut attempts = 0;

    loop {
        attempts += 1;
        let request = ChatRequest {
            model: config.model.clone(),
            messages: messages.clone(),
            max_tokens: config.max_tokens,
        };
        let reply = collect_text(provider.stream_chat(request)).map_err(HealError::Provider)?;
        let fix = extract_wf(&reply).ok_or(HealError::NoWfBlock)?;

        match validate_fix(base_source, &fix) {
            Ok(()) => return Ok(HealOutcome { source: fix, attempts }),
            Err(reason) => {
                if attempts >= config.max_attempts {
                    return Err(HealError::GaveUp { last_reason: reason, attempts });
                }
                messages.push(ChatMessage::assistant(reply));
                messages.push(ChatMessage::user(format!(
                    "That didn't work: {reason}. Return a corrected complete page that fixes ONLY \
                     the error and leaves everything visible exactly as it is.",
                )));
            }
        }
    }
}

/// A fix passes if it compiles and the design-freeze holds: it changed no element,
/// modifier, or style block (position-independent, so a logic fix that shifts ids
/// is fine). Text and logic may change; layout/styling may not.
fn validate_fix(base: &str, fix: &str) -> Result<(), String> {
    compile_source(fix).map_err(|e| format!("it didn't compile: {e}"))?;
    if !crate::diff::design_preserved(base, fix).map_err(|e| format!("couldn't compare it: {e}"))? {
        return Err("it changed the visible design (layout, styling, or components)".to_string());
    }
    Ok(())
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

    // ── scoped edit (M3.1) ────────────────────────────────────────────────────
    const BASE: &str = "Page Home (path: \"/\") { Container { Heading(\"Hi\", h1) } }";

    fn heading_id(src: &str) -> String {
        let (site, merged, _) = crate::compile_merged([("<t>", src)]).unwrap();
        site.node_map
            .iter()
            .find(|(_, i)| i.span.slice(&merged).starts_with("Heading"))
            .map(|(id, _)| id.clone())
            .expect("a Heading node")
    }

    #[test]
    fn edit_node_applies_a_valid_replacement() {
        let id = heading_id(BASE);
        let p = ScriptedProvider::with_text("```wf\nHeading(\"Hi\", h1, large)\n```");
        let out = edit_node(&p, BASE, &id, "make the heading large", &cfg()).unwrap();
        assert_eq!(out.attempts, 1);
        assert!(out.source.contains("large"), "the edit landed");
        assert!(out.source.contains("Container"), "the rest of the page is preserved");
    }

    #[test]
    fn edit_node_repairs_a_bad_replacement() {
        let id = heading_id(BASE);
        let p = ScriptedProvider::new(ProviderKind::Anthropic);
        // Ghost() parses but is an undeclared component → the gate rejects it → repair.
        p.push_text("```wf\nGhost()\n```").push_text("```wf\nHeading(\"Hi\", h2)\n```");
        let out = edit_node(&p, BASE, &id, "smaller heading", &cfg()).unwrap();
        assert_eq!(out.attempts, 2);
        assert!(out.source.contains("h2"));
    }

    #[test]
    fn edit_node_gives_up_after_max_attempts() {
        let id = heading_id(BASE);
        let p = ScriptedProvider::new(ProviderKind::Anthropic);
        p.push_text("```wf\nGhost()\n```")
            .push_text("```wf\nGhost()\n```")
            .push_text("```wf\nGhost()\n```");
        assert!(matches!(edit_node(&p, BASE, &id, "x", &cfg()), Err(GenError::Unrepaired { .. })));
    }

    #[test]
    fn edit_node_rejects_an_unknown_node() {
        let p = ScriptedProvider::with_text("```wf\nText(\"x\")\n```");
        assert!(edit_node(&p, BASE, "Nope:9", "x", &cfg()).is_err());
    }

    // ── self-heal + design-freeze (M4.2) ──────────────────────────────────────
    // Compiles, but `{count}` is undefined → a runtime ReferenceError.
    const BUGGY: &str = "Page Home (path: \"/\") { Text(\"{count}\") }";
    // A logic-only fix: adds state (shifts the Text's structural id) but changes
    // nothing visible.
    const LOGIC_FIX: &str = "```wf\nPage Home (path: \"/\") { state count = 0\n  Text(\"{count}\") }\n```";
    // Also adds a `bold` modifier → a design change the freeze must reject.
    const STYLE_FIX: &str = "```wf\nPage Home (path: \"/\") { state count = 0\n  Text(\"{count}\", bold) }\n```";

    #[test]
    fn self_heal_accepts_a_logic_only_fix() {
        let p = ScriptedProvider::with_text(LOGIC_FIX);
        let out = self_heal(&p, BUGGY, "ReferenceError: count is not defined", &cfg()).unwrap();
        assert_eq!(out.attempts, 1);
        assert!(out.source.contains("state count"));
        assert!(!out.source.contains("bold"));
    }

    #[test]
    fn design_freeze_rejects_a_visible_change_then_accepts_a_clean_fix() {
        let p = ScriptedProvider::new(ProviderKind::Anthropic);
        p.push_text(STYLE_FIX).push_text(LOGIC_FIX); // style change rejected, then clean
        let out = self_heal(&p, BUGGY, "ReferenceError", &cfg()).unwrap();
        assert_eq!(out.attempts, 2, "the bold-adding fix was rejected by the design-freeze");
        assert!(!out.source.contains("bold"));
    }

    #[test]
    fn self_heal_gives_up_if_every_fix_changes_the_design() {
        let p = ScriptedProvider::new(ProviderKind::Anthropic);
        p.push_text(STYLE_FIX).push_text(STYLE_FIX).push_text(STYLE_FIX);
        assert!(matches!(self_heal(&p, BUGGY, "err", &cfg()), Err(HealError::GaveUp { .. })));
    }

    #[test]
    fn self_heal_retries_a_non_compiling_fix() {
        let p = ScriptedProvider::new(ProviderKind::Anthropic);
        p.push_text("```wf\nPage Home (path: \"/\") { Ghost() }\n```").push_text(LOGIC_FIX);
        let out = self_heal(&p, BUGGY, "err", &cfg()).unwrap();
        assert_eq!(out.attempts, 2);
    }
}
