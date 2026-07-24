//! A scripted [`Provider`] for offline tests and a fake-backed demo mode.
//!
//! The whole generation → validate → repair loop is built against the [`Provider`]
//! seam, so a fake that replays canned responses lets the loop (and its bounded
//! self-heal) be tested deterministically with no network. Responses are a FIFO
//! queue: successive `stream_chat` calls (e.g. a repair round) pop the next turn.

use std::collections::VecDeque;
use std::sync::Mutex;

use crate::{ChatDelta, ChatRequest, Provider, ProviderKind};

/// A [`Provider`] that replays queued turns instead of calling a network.
pub struct ScriptedProvider {
    kind: ProviderKind,
    turns: Mutex<VecDeque<Vec<ChatDelta>>>,
}

impl ScriptedProvider {
    /// An empty scripted provider; queue turns with [`push_text`](Self::push_text)
    /// / [`push_deltas`](Self::push_deltas).
    pub fn new(kind: ProviderKind) -> Self {
        Self { kind, turns: Mutex::new(VecDeque::new()) }
    }

    /// A one-turn provider that emits `text` then completes.
    pub fn with_text(text: impl Into<String>) -> Self {
        let p = Self::new(ProviderKind::Anthropic);
        p.push_text(text);
        p
    }

    /// Queue a turn that streams `text` as a single delta then `Done`.
    pub fn push_text(&self, text: impl Into<String>) -> &Self {
        self.turns
            .lock()
            .unwrap()
            .push_back(vec![ChatDelta::Text(text.into()), ChatDelta::Done]);
        self
    }

    /// Queue a turn that streams an exact sequence of deltas.
    pub fn push_deltas(&self, deltas: Vec<ChatDelta>) -> &Self {
        self.turns.lock().unwrap().push_back(deltas);
        self
    }
}

impl Provider for ScriptedProvider {
    fn kind(&self) -> ProviderKind {
        self.kind
    }

    fn stream_chat(&self, _req: ChatRequest) -> async_channel::Receiver<ChatDelta> {
        let turn = self
            .turns
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| vec![ChatDelta::Error("scripted provider exhausted".into())]);
        let (tx, rx) = async_channel::unbounded();
        for delta in turn {
            let _ = tx.send_blocking(delta);
        }
        rx
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect_text;

    fn req() -> ChatRequest {
        ChatRequest { model: "test".into(), messages: vec![], max_tokens: 64 }
    }

    #[test]
    fn scripted_provider_collects_to_text() {
        let p = ScriptedProvider::with_text("hello world");
        assert_eq!(collect_text(p.stream_chat(req())).unwrap(), "hello world");
    }

    #[test]
    fn turns_replay_fifo() {
        let p = ScriptedProvider::new(ProviderKind::Anthropic);
        p.push_text("first").push_text("second");
        assert_eq!(collect_text(p.stream_chat(req())).unwrap(), "first");
        assert_eq!(collect_text(p.stream_chat(req())).unwrap(), "second");
    }

    #[test]
    fn multiple_text_deltas_concatenate() {
        let p = ScriptedProvider::new(ProviderKind::Anthropic);
        p.push_deltas(vec![
            ChatDelta::Text("Page ".into()),
            ChatDelta::Text("Home {}".into()),
            ChatDelta::Done,
        ]);
        assert_eq!(collect_text(p.stream_chat(req())).unwrap(), "Page Home {}");
    }

    #[test]
    fn error_delta_is_surfaced() {
        let p = ScriptedProvider::new(ProviderKind::Anthropic);
        p.push_deltas(vec![ChatDelta::Text("partial".into()), ChatDelta::Error("boom".into())]);
        assert_eq!(collect_text(p.stream_chat(req())), Err("boom".into()));
    }

    #[test]
    fn exhausted_provider_yields_an_error() {
        let p = ScriptedProvider::new(ProviderKind::Anthropic);
        assert!(collect_text(p.stream_chat(req())).is_err());
    }
}
