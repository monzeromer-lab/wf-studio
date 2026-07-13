//! The native bridge between the webview design and the Rust backend.
//!
//! Today it carries page lifecycle + error signals (the seed of the
//! self-healing loop, FR-19) and canvas section clicks (so a click in the
//! preview becomes the chat's selected edit target). It is also the seam
//! where the design's actions get wired to real work: `wf-core` compilation
//! and `wf-ai` generation will replace the design's simulated pipeline by
//! handling request messages here and pushing results back with
//! `evaluate_script`.

use std::sync::OnceLock;
use std::sync::mpsc::{Receiver, Sender, channel};

/// Injected before any page script runs (after the React-resources override).
/// Forwards page load, runtime errors, and canvas section clicks to
/// [`on_message`] over wry's IPC channel.
pub const BRIDGE_JS: &str = r#"
(function () {
  if (window.__wfBridge) return;
  function post(kind, payload) {
    try { window.ipc.postMessage(JSON.stringify({ kind: kind, payload: payload })); }
    catch (_) {}
  }
  window.__wfBridge = { post: post };

  window.addEventListener('error', function (e) {
    post('runtime-error', { message: String(e.message || e.error || 'error'), source: String(e.filename || ''), line: e.lineno || 0 });
  });
  window.addEventListener('unhandledrejection', function (e) {
    post('runtime-error', { message: 'Unhandled rejection: ' + String(e.reason), source: '', line: 0 });
  });
  window.addEventListener('DOMContentLoaded', function () { post('page-loaded', {}); });

  // Click-to-select: report every `data-wf-el` ancestor of the clicked
  // element, innermost first, so the backend (which owns the recognized
  // section list) can resolve the nearest one that actually matters —
  // clicking a menu item or the header's brand mark should still select the
  // enclosing "menu" or "header" section. The backend is the single source
  // of truth for what's selected; it pushes the resulting highlight back via
  // `evaluate_script` (see `apply_ipc`/`sync_preview_selection` in app.rs),
  // rather than this script guessing at it optimistically.
  document.addEventListener('click', function (e) {
    var keys = [];
    var el = e.target;
    while (el && el.getAttribute) {
      if (el.hasAttribute('data-wf-el')) keys.push(el.getAttribute('data-wf-el'));
      el = el.parentElement;
    }
    post('select', { keys: keys });
  });
})();
"#;

/// One bridge message the backend needs to act on (as opposed to the purely
/// diagnostic ones, which [`parse`] logs and discards on the spot).
pub enum Event {
    /// A known, selectable section was clicked, or `None` to clear the
    /// selection (e.g. a click outside any `[data-wf-el]`).
    Select(Option<&'static str>),
}

/// Parse one `{ kind, payload }` JSON message from the design.
fn parse(message: &str) -> Option<Event> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(message) else {
        eprintln!("wf-studio bridge (raw): {message}");
        return None;
    };
    let kind = value.get("kind").and_then(|k| k.as_str()).unwrap_or("");
    match kind {
        "page-loaded" => {
            eprintln!("wf-studio: design rendered");
            None
        }
        "runtime-error" | "console-error" => {
            let payload = value.get("payload").cloned().unwrap_or_default();
            eprintln!("wf-studio bridge {kind}: {payload}");
            None
        }
        "select" => {
            let keys = value.get("payload").and_then(|p| p.get("keys")).and_then(|k| k.as_array());
            let resolved = keys
                .into_iter()
                .flatten()
                .find_map(|k| k.as_str().and_then(crate::state::section_key));
            Some(Event::Select(resolved))
        }
        _ => {
            eprintln!("wf-studio bridge: {message}");
            None
        }
    }
}

static QUEUE: OnceLock<Sender<String>> = OnceLock::new();

/// wry's IPC handler callback: queues the raw message for [`drain`] to parse
/// and apply on the GPUI loop. Has no `Context<StudioApp>` of its own (it's a
/// plain `'static` closure registered once at webview-build time), hence the
/// queue instead of calling back into app state directly.
pub fn on_message(message: String) {
    if let Some(tx) = QUEUE.get() {
        let _ = tx.send(message);
    }
}

/// Take the receiving end of the bridge queue. Must be called exactly once
/// (from [`crate::app`]), before the preview webview can post any message.
pub fn take_receiver() -> Receiver<String> {
    let (tx, rx) = channel();
    QUEUE.set(tx).ok().expect("ipc::take_receiver called twice");
    rx
}

/// Drain and parse every message queued since the last call.
pub fn drain(rx: &Receiver<String>) -> Vec<Event> {
    std::iter::from_fn(|| rx.try_recv().ok()).filter_map(|m| parse(&m)).collect()
}
