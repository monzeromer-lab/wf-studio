//! Embedded icon set for the Studio chrome.
//!
//! `gpui-component` references icons by path (`icons/foo.svg`) but ships none,
//! so the host app must supply an [`gpui::AssetSource`]. We embed the exact SVG
//! shapes used by the product design system (its `mkIcons` table) so the native
//! chrome matches it icon-for-icon. GPUI renders an SVG as a monochrome coverage
//! mask and re-tints it with the element's `text_color`, so the baked-in color
//! is irrelevant — every glyph is authored `stroke="#000"` / `fill="#000"`.

use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};

macro_rules! stroke_icon {
    ($w:expr, $inner:expr) => {
        concat!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" ",
            "stroke=\"#000\" stroke-width=\"",
            $w,
            "\" stroke-linecap=\"round\" stroke-linejoin=\"round\">",
            $inner,
            "</svg>"
        )
    };
}

macro_rules! fill_icon {
    ($inner:expr) => {
        concat!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"#000\">",
            $inner,
            "</svg>"
        )
    };
}

const ICONS: &[(&str, &str)] = &[
    // ── brand / status ──────────────────────────────────────────────────────
    ("sparkle", stroke_icon!("1.8", "<path d=\"M12 3l1.9 5.1L19 10l-5.1 1.9L12 17l-1.9-5.1L5 10l5.1-1.9z\"/>")),
    ("check", stroke_icon!("2.2", "<path d=\"M20 6L9 17l-5-5\"/>")),
    (
        "check-circle",
        stroke_icon!("1.8", "<circle cx=\"12\" cy=\"12\" r=\"9\"/><path d=\"M8.5 12.5l2.4 2.4 4.6-5.3\"/>"),
    ),
    ("close", stroke_icon!("2", "<path d=\"M18 6L6 18M6 6l12 12\"/>")),
    ("plus", stroke_icon!("2", "<path d=\"M12 5v14M5 12h14\"/>")),
    ("minus", stroke_icon!("2", "<path d=\"M5 12h14\"/>")),
    (
        "shield",
        stroke_icon!("1.85", "<path d=\"M12 3l7 3v5c0 4.5-3 8-7 10-4-2-7-5.5-7-10V6z\"/><path d=\"M9.5 12l1.8 1.8L15 10\"/>"),
    ),
    (
        "alert-circle",
        stroke_icon!("2", "<circle cx=\"12\" cy=\"12\" r=\"9\"/><path d=\"M12 8v4M12 16h.01\"/>"),
    ),
    (
        "alert-triangle",
        stroke_icon!("1.9", "<path d=\"M12 9v4M12 17h.01\"/><path d=\"M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z\"/>"),
    ),
    // ── title bar / toolbar ─────────────────────────────────────────────────
    ("settings", stroke_icon!("1.6", "<circle cx=\"12\" cy=\"12\" r=\"3\"/><path d=\"M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9c.2.61.76 1.05 1.42 1.09H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z\"/>")),
    ("clock", stroke_icon!("1.85", "<circle cx=\"12\" cy=\"12\" r=\"9\"/><path d=\"M12 8v4l3 2\"/>")),
    ("share", stroke_icon!("1.85", "<path d=\"M4 12v8a1 1 0 0 0 1 1h14a1 1 0 0 0 1-1v-8\"/><path d=\"M12 3v13\"/><path d=\"M8 7l4-4 4 4\"/>")),
    ("cloud", stroke_icon!("1.85", "<path d=\"M18 10h-1.26A8 8 0 1 0 9 20h9a5 5 0 0 0 0-10z\"/>")),
    ("undo", stroke_icon!("1.85", "<path d=\"M9 14L4 9l5-5\"/><path d=\"M4 9h11a6 6 0 0 1 0 12h-4\"/>")),
    ("redo", stroke_icon!("1.85", "<path d=\"M15 14l5-5-5-5\"/><path d=\"M20 9H9a6 6 0 0 0 0 12h4\"/>")),
    ("refresh", stroke_icon!("1.85", "<path d=\"M21 12a9 9 0 1 1-2.6-6.3\"/><path d=\"M21 4v5h-5\"/>")),
    ("monitor", stroke_icon!("1.85", "<rect x=\"3\" y=\"4\" width=\"18\" height=\"13\" rx=\"2\"/><path d=\"M8 21h8\"/><path d=\"M12 17v4\"/>")),
    ("tablet", stroke_icon!("1.85", "<rect x=\"5\" y=\"3\" width=\"14\" height=\"18\" rx=\"2\"/><path d=\"M12 17h.01\"/>")),
    ("phone", stroke_icon!("1.85", "<rect x=\"7\" y=\"3\" width=\"10\" height=\"18\" rx=\"2\"/><path d=\"M11 18h2\"/>")),
    // legacy device names still referenced by the old toolbar/canvas
    ("desktop", stroke_icon!("1.9", "<rect x=\"2\" y=\"4\" width=\"20\" height=\"13\" rx=\"2\"/><path d=\"M8 21h8M12 17v4\"/>")),
    ("mobile", stroke_icon!("1.9", "<rect x=\"7\" y=\"2\" width=\"10\" height=\"20\" rx=\"2.5\"/><path d=\"M11 18h2\"/>")),
    ("history", stroke_icon!("1.9", "<path d=\"M3 3v5h5\"/><path d=\"M3.05 13A9 9 0 1 0 6 5.3L3 8\"/><path d=\"M12 7v5l3 2\"/>")),
    // ── chevrons / arrows / window ──────────────────────────────────────────
    ("chevron-down", stroke_icon!("1.85", "<path d=\"M6 9l6 6 6-6\"/>")),
    ("chevron-right", stroke_icon!("1.85", "<path d=\"M9 6l6 6-6 6\"/>")),
    ("arrow-right", stroke_icon!("2", "<path d=\"M5 12h14\"/><path d=\"M13 6l6 6-6 6\"/>")),
    ("arrow-left", stroke_icon!("2", "<path d=\"M19 12H5\"/><path d=\"M11 6l-6 6 6 6\"/>")),
    ("square", stroke_icon!("1.85", "<rect x=\"5\" y=\"5\" width=\"14\" height=\"14\" rx=\"2\"/>")),
    ("sitemenu", stroke_icon!("2", "<path d=\"M4 6h16\"/><path d=\"M4 12h16\"/><path d=\"M4 18h16\"/>")),
    // ── composer / chat ─────────────────────────────────────────────────────
    ("send", stroke_icon!("1.85", "<path d=\"M22 2L11 13\"/><path d=\"M22 2l-7 20-4-9-9-4 20-7z\"/>")),
    ("stop", fill_icon!("<rect x=\"6\" y=\"6\" width=\"12\" height=\"12\" rx=\"2.5\"/>")),
    ("paperclip", stroke_icon!("1.8", "<path d=\"M21.44 11.05l-9.19 9.19a5 5 0 0 1-7.07-7.07l9.19-9.19a3.34 3.34 0 0 1 4.72 4.72l-9.2 9.19a1.67 1.67 0 0 1-2.36-2.36l8.49-8.48\"/>")),
    ("grid", stroke_icon!("1.7", "<path d=\"M4 4h7v7H4z\"/><path d=\"M13 4h7v7h-7z\"/><path d=\"M4 13h7v7H4z\"/><path d=\"M13 13h7v7h-7z\"/>")),
    ("boxes", stroke_icon!("1.6", "<path d=\"M3 8l4-2 4 2v4l-4 2-4-2z\"/><path d=\"M13 8l4-2 4 2v4l-4 2-4-2z\"/><path d=\"M8 15l4-2 4 2v4l-4 2-4-2z\"/>")),
    ("cpu", stroke_icon!("1.6", "<rect x=\"4\" y=\"4\" width=\"16\" height=\"16\" rx=\"2.5\"/><path d=\"M9 9h6v6H9z\"/><path d=\"M9 2v3M15 2v3M9 19v3M15 19v3M2 9h3M2 15h3M19 9h3M19 15h3\"/>")),
    ("plug", stroke_icon!("1.85", "<path d=\"M9 2v6M15 2v6\"/><path d=\"M7 8h10v3a5 5 0 0 1-10 0z\"/><path d=\"M12 16v6\"/>")),
    ("skill", stroke_icon!("1.8", "<path d=\"M12 2 3 7v10l9 5 9-5V7z\"/><path d=\"M3 7l9 5 9-5\"/><path d=\"M12 22V12\"/>")),
    // ── right panel / inspector ─────────────────────────────────────────────
    ("panel-close", stroke_icon!("1.85", "<rect x=\"3\" y=\"4\" width=\"18\" height=\"16\" rx=\"2\"/><path d=\"M15 4l-6 8 6 8\"/>")),
    ("layers", stroke_icon!("1.6", "<path d=\"M12 3l9 5-9 5-9-5 9-5z\"/><path d=\"M3 13l9 5 9-5\"/><path d=\"M3 17l9 5 9-5\"/>")),
    ("eye", stroke_icon!("1.85", "<path d=\"M2 12s3.5-6.5 10-6.5S22 12 22 12s-3.5 6.5-10 6.5S2 12 2 12z\"/><circle cx=\"12\" cy=\"12\" r=\"2.6\"/>")),
    ("sliders", stroke_icon!("1.85", "<path d=\"M4 21v-7M4 10V3M12 21v-9M12 8V3M20 21v-5M20 12V3\"/><path d=\"M1 14h6M9 8h6M17 16h6\"/>")),
    ("type", stroke_icon!("1.85", "<path d=\"M4 7V5h16v2\"/><path d=\"M9 19h6\"/><path d=\"M12 5v14\"/>")),
    ("image", stroke_icon!("1.85", "<rect x=\"3\" y=\"4\" width=\"18\" height=\"16\" rx=\"2\"/><circle cx=\"8.5\" cy=\"9.5\" r=\"1.5\"/><path d=\"M21 15l-5-5L5 21\"/>")),
    ("reset", stroke_icon!("1.85", "<path d=\"M3 12a9 9 0 1 0 3-6.7L3 8\"/><path d=\"M3 3v5h5\"/>")),
    ("target", stroke_icon!("1.85", "<circle cx=\"12\" cy=\"12\" r=\"8\"/><circle cx=\"12\" cy=\"12\" r=\"3\"/>")),
    ("split", stroke_icon!("2", "<path d=\"M8 7l-4 5 4 5\"/><path d=\"M16 7l4 5-4 5\"/><path d=\"M12 3v18\"/>")),
    ("drag", stroke_icon!("2.4", "<path d=\"M9 5h.01M9 12h.01M9 19h.01M15 5h.01M15 12h.01M15 19h.01\"/>")),
    ("loader", stroke_icon!("1.85", "<path d=\"M12 3v4M12 17v4M5.6 5.6l2.8 2.8M15.6 15.6l2.8 2.8M3 12h4M17 12h4M5.6 18.4l2.8-2.8M15.6 8.4l2.8-2.8\"/>")),
    // ── modals / misc ───────────────────────────────────────────────────────
    ("globe", stroke_icon!("1.85", "<circle cx=\"12\" cy=\"12\" r=\"9\"/><path d=\"M3 12h18\"/><path d=\"M12 3a15 15 0 0 1 0 18 15 15 0 0 1 0-18\"/>")),
    ("globe-big", stroke_icon!("1.85", "<circle cx=\"12\" cy=\"12\" r=\"10\"/><path d=\"M2 12h20\"/><path d=\"M12 2a15 15 0 0 1 0 20 15 15 0 0 1 0-20\"/>")),
    ("search", stroke_icon!("1.85", "<circle cx=\"10.5\" cy=\"10.5\" r=\"7\"/><path d=\"M21 21l-4.3-4.3\"/>")),
    ("link", stroke_icon!("1.85", "<path d=\"M9 15l6-6\"/><path d=\"M11 6l1-1a4 4 0 0 1 6 6l-1 1\"/><path d=\"M13 18l-1 1a4 4 0 0 1-6-6l1-1\"/>")),
    ("copy", stroke_icon!("1.85", "<rect x=\"9\" y=\"9\" width=\"11\" height=\"11\" rx=\"2\"/><path d=\"M5 15H4a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h10a1 1 0 0 1 1 1v1\"/>")),
    ("trash", stroke_icon!("1.85", "<path d=\"M3 6h18\"/><path d=\"M8 6V4a1 1 0 0 1 1-1h6a1 1 0 0 1 1 1v2\"/><path d=\"M6 6l1 14a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1l1-14\"/>")),
    ("download", stroke_icon!("1.85", "<path d=\"M12 3v12\"/><path d=\"M7 11l5 5 5-5\"/><path d=\"M5 21h14\"/>")),
    ("user", stroke_icon!("1.85", "<circle cx=\"12\" cy=\"8\" r=\"4\"/><path d=\"M4 21v-1a5 5 0 0 1 5-5h6a5 5 0 0 1 5 5v1\"/>")),
    ("server", stroke_icon!("1.85", "<rect x=\"3\" y=\"4\" width=\"18\" height=\"8\" rx=\"2\"/><rect x=\"3\" y=\"12\" width=\"18\" height=\"8\" rx=\"2\"/><path d=\"M6 8h.01M6 16h.01\"/>")),
    ("key", stroke_icon!("1.85", "<circle cx=\"15.5\" cy=\"8.5\" r=\"5.5\"/><path d=\"M15 7l4 4\"/><path d=\"M10.5 13.5L4 20v-3\"/>")),
    ("zap", stroke_icon!("1.85", "<path d=\"M13 2L4 14h7l-1 8 9-12h-7l1-8z\"/>")),
    ("logout", stroke_icon!("1.85", "<path d=\"M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4\"/><path d=\"M16 17l5-5-5-5\"/><path d=\"M21 12H9\"/>")),
    ("credit-card", stroke_icon!("1.85", "<rect x=\"2\" y=\"5\" width=\"20\" height=\"14\" rx=\"2.5\"/><path d=\"M2 10h20\"/>")),
    ("palette", stroke_icon!("1.85", "<path d=\"M12 3a9 9 0 1 0 0 18c1 0 1.5-.8 1.5-1.6 0-.5-.3-.9-.6-1.2-.3-.4-.5-.7-.5-1.2 0-.8.7-1.5 1.6-1.5H16a5 5 0 0 0 5-5c0-3.9-4-6.5-9-6.5z\"/><circle cx=\"7.5\" cy=\"11.5\" r=\"1\"/><circle cx=\"12\" cy=\"8.5\" r=\"1\"/><circle cx=\"16\" cy=\"11.5\" r=\"1\"/>")),
    ("lock", stroke_icon!("1.85", "<rect x=\"4\" y=\"11\" width=\"16\" height=\"10\" rx=\"2\"/><path d=\"M7 11V7a5 5 0 0 1 10 0v4\"/>")),
    ("wifi-off", stroke_icon!("1.9", "<path d=\"M1 1l22 22\"/><path d=\"M8.5 4.7A11 11 0 0 1 21 8\"/><path d=\"M3 8a11 11 0 0 1 3.2-2.3\"/><path d=\"M6.3 11.3A6 6 0 0 1 9 9.9\"/><path d=\"M17.5 11.5A6 6 0 0 0 14 10\"/><path d=\"M9.5 14.6a2.5 2.5 0 0 1 5 0\"/><path d=\"M12 20h.01\"/>")),
    // ── brand SSO (filled) ──────────────────────────────────────────────────
    ("google", fill_icon!("<path d=\"M21.35 11.1h-9.18v2.98h5.27c-.23 1.4-1.62 4.1-5.27 4.1-3.17 0-5.76-2.62-5.76-5.85s2.59-5.85 5.76-5.85c1.8 0 3.01.77 3.7 1.43l2.52-2.43C16.6 3.98 14.6 3.1 12.17 3.1 6.96 3.1 2.75 7.3 2.75 12.5s4.21 9.4 9.42 9.4c5.44 0 9.04-3.82 9.04-9.2 0-.62-.07-1.09-.16-1.6z\"/>")),
    ("apple", fill_icon!("<path d=\"M16.4 12.6c0-2.2 1.8-3.3 1.9-3.3-1-1.5-2.6-1.7-3.2-1.7-1.4-.1-2.6.8-3.3.8-.7 0-1.7-.8-2.8-.8-1.5 0-2.8.8-3.5 2.1-1.5 2.6-.4 6.5 1.1 8.6.7 1 1.5 2.2 2.6 2.1 1-.04 1.4-.67 2.7-.67 1.2 0 1.6.67 2.7.65 1.1-.02 1.8-1.05 2.5-2.06.8-1.2 1.1-2.3 1.1-2.36-.02-.01-2.1-.81-2.1-3.19zM14.3 6.2c.6-.7 1-1.7.9-2.7-.85.03-1.9.57-2.5 1.27-.55.62-1.03 1.62-.9 2.58.95.07 1.9-.48 2.5-1.15z\"/>")),
    // ── `gpui_component::TitleBar` client-side window controls ───────────────
    ("window-minimize", stroke_icon!("1.9", "<path d=\"M5 12h14\"/>")),
    ("window-maximize", stroke_icon!("1.9", "<rect x=\"5\" y=\"5\" width=\"14\" height=\"14\" rx=\"1.5\"/>")),
    ("window-restore", stroke_icon!("1.9", "<rect x=\"8\" y=\"4\" width=\"12\" height=\"12\" rx=\"1.5\"/><path d=\"M4 8v12h12v-4\"/>")),
    ("window-close", stroke_icon!("2", "<path d=\"M6 6l12 12M18 6L6 18\"/>")),
];

/// Serves the embedded [`ICONS`] under `icons/<name>.svg`.
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let name = path
            .trim_start_matches('/')
            .strip_prefix("icons/")
            .and_then(|n| n.strip_suffix(".svg"));
        Ok(name
            .and_then(|n| ICONS.iter().find(|(k, _)| *k == n))
            .map(|(_, svg)| Cow::Borrowed(svg.as_bytes())))
    }

    fn list(&self, _path: &str) -> Result<Vec<SharedString>> {
        Ok(ICONS.iter().map(|(k, _)| format!("icons/{k}.svg").into()).collect())
    }
}
