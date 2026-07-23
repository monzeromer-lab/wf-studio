//! The preview webview's entry path.
//!
//! The document is no longer a static mock — it's the **live compiled site**,
//! served over `wf://` from the project's [`webfluent::CompiledSite`] (see
//! `app::serve` / `app::resolve`). The old hand-written `layali.html` mock is
//! retired (the file remains under `preview/` for reference but is unused).

/// The document the preview webview boots; `serve` resolves it to the `/` page.
pub const PREVIEW_ENTRY: &str = "index.html";
