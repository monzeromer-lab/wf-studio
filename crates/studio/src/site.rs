//! The website **preview**, served to the canvas webview over `wf://`.
//!
//! The Studio chrome is native GPUI — only the generated site (`CafeSite`) is
//! rendered by the webview, using the design's own runtime (`support.js`).
//! React is vendored (not from a CDN) so the preview renders offline and never
//! makes a network request just to boot.

/// The document the preview webview boots (path must end in `.dc.html` for the
/// runtime to pick it as the root component).
pub const PREVIEW_ENTRY: &str = "CafeSite.dc.html";

const CAFESITE_HTML: &[u8] = include_bytes!("../../../docs/CafeSite.dc.html");
const SUPPORT_JS: &[u8] = include_bytes!("../../../docs/support.js");
const REACT_JS: &[u8] = include_bytes!("../vendor/react.js");
const REACT_DOM_JS: &[u8] = include_bytes!("../vendor/react-dom.js");

const HTML: &str = "text/html; charset=utf-8";
const JS: &str = "text/javascript; charset=utf-8";

/// Resolve a request path to `(mime, bytes)`, or `None` for a 404.
pub fn resource(path: &str) -> Option<(&'static str, &'static [u8])> {
    Some(match path.trim_start_matches('/') {
        "" | "CafeSite.dc.html" => (HTML, CAFESITE_HTML),
        "support.js" => (JS, SUPPORT_JS),
        "vendor/react.js" => (JS, REACT_JS),
        "vendor/react-dom.js" => (JS, REACT_DOM_JS),
        _ => return None,
    })
}
