//! The website **preview**, served to the canvas webview over `wf://`.
//!
//! The Studio chrome is native GPUI — only the generated site (the cinematic
//! "Layali" rooftop-venue demo) renders in the webview. It's a self-contained
//! vanilla page (no framework, no network): the backend pushes state into it
//! with `window.__wfApply(state)` over `evaluate_script`, and it reports canvas
//! clicks back over the IPC bridge (see [`crate::ipc`]).

/// The document the preview webview boots.
pub const PREVIEW_ENTRY: &str = "index.html";

const LAYALI_HTML: &[u8] = include_bytes!("preview/layali.html");

const HTML: &str = "text/html; charset=utf-8";

/// Resolve a request path to `(mime, bytes)`, or `None` for a 404.
pub fn resource(path: &str) -> Option<(&'static str, &'static [u8])> {
    Some(match path.trim_start_matches('/') {
        "" | "index.html" => (HTML, LAYALI_HTML),
        _ => return None,
    })
}
