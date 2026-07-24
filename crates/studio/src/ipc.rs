//! The native bridge between the webview site and the Rust backend.
//!
//! It carries page lifecycle + runtime errors and, crucially, canvas clicks:
//! a click in the preview becomes the chat's selected edit target. The backend
//! pushes state back into the page with `window.__wfApply(...)` over
//! `evaluate_script` (see `sync_preview` in `app.rs`).

use std::sync::OnceLock;
use std::sync::mpsc::{Receiver, Sender, channel};

/// Injected before any page script runs. Forwards page load, runtime errors,
/// and canvas element clicks (with the shift/meta/ctrl modifier for additive
/// multi-select) to [`on_message`] over wry's IPC channel.
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
  window.addEventListener('DOMContentLoaded', function () { post('page-loaded', {}); });

  // Click-to-select: report every `data-wf-node` ancestor of the clicked element
  // (Slice-2 node ids), innermost first, plus whether an additive modifier was
  // held. An empty list (a click on bare page) clears the selection. The backend
  // resolves the id to source via the node map and highlights the element.
  document.addEventListener('click', function (e) {
    var keys = [];
    var el = e.target;
    while (el && el.getAttribute) {
      if (el.hasAttribute('data-wf-node')) keys.push(el.getAttribute('data-wf-node'));
      el = el.parentElement;
    }
    post('select', { keys: keys, shift: !!(e.shiftKey || e.metaKey || e.ctrlKey) });
  });
})();
"#;

/// One bridge message the backend acts on.
pub enum Event {
    /// A site element was clicked. `additive` = a modifier was held.
    Select { key: String, additive: bool },
    /// A click landed outside any element — clear the selection.
    Deselect,
    /// The page finished loading — push the current state into it.
    PageLoaded,
    /// The preview threw a runtime error — a candidate for self-healing (§4.6).
    RuntimeError { message: String },
}

/// Parse one `{ kind, payload }` JSON message from the page.
fn parse(message: &str) -> Option<Event> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(message) else {
        eprintln!("wf-studio bridge (raw): {message}");
        return None;
    };
    let kind = value.get("kind").and_then(|k| k.as_str()).unwrap_or("");
    match kind {
        "page-loaded" => Some(Event::PageLoaded),
        "runtime-error" => {
            let message = value
                .get("payload")
                .and_then(|p| p.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("runtime error")
                .to_string();
            Some(Event::RuntimeError { message })
        }
        "console-error" => {
            let payload = value.get("payload").cloned().unwrap_or_default();
            eprintln!("wf-studio bridge {kind}: {payload}");
            None
        }
        "select" => {
            let payload = value.get("payload");
            let additive = payload.and_then(|p| p.get("shift")).and_then(|s| s.as_bool()).unwrap_or(false);
            let first = payload
                .and_then(|p| p.get("keys"))
                .and_then(|k| k.as_array())
                .and_then(|arr| arr.iter().find_map(|k| k.as_str()));
            match first {
                Some(key) => Some(Event::Select { key: key.to_string(), additive }),
                None => Some(Event::Deselect),
            }
        }
        _ => {
            eprintln!("wf-studio bridge: {message}");
            None
        }
    }
}

static QUEUE: OnceLock<Sender<String>> = OnceLock::new();

/// wry's IPC handler callback: queues the raw message for [`drain`] to parse
/// and apply on the GPUI loop.
pub fn on_message(message: String) {
    if let Some(tx) = QUEUE.get() {
        let _ = tx.send(message);
    }
}

/// Take the receiving end of the bridge queue. Must be called exactly once.
pub fn take_receiver() -> Receiver<String> {
    let (tx, rx) = channel();
    QUEUE.set(tx).ok().expect("ipc::take_receiver called twice");
    rx
}

/// Drain and parse every message queued since the last call.
pub fn drain(rx: &Receiver<String>) -> Vec<Event> {
    std::iter::from_fn(|| rx.try_recv().ok()).filter_map(|m| parse(&m)).collect()
}
