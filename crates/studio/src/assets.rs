//! Embedded icon set for the Studio chrome.
//!
//! `gpui-component` references icons by path (`icons/foo.svg`) but ships none,
//! so the host app must supply an [`gpui::AssetSource`]. We embed the exact SVG
//! shapes used by the product design mock so the native chrome matches it
//! icon-for-icon. GPUI renders an SVG as a monochrome coverage mask and re-tints
//! it with the element's `text_color`, so the baked-in color is irrelevant.

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
    (
        "sparkle",
        stroke_icon!("1.8", "<path d=\"M12 3l1.9 5.1L19 10l-5.1 1.9L12 17l-1.9-5.1L5 10l5.1-1.9z\"/>"),
    ),
    (
        "history",
        stroke_icon!(
            "1.9",
            "<path d=\"M3 3v5h5\"/><path d=\"M3.05 13A9 9 0 1 0 6 5.3L3 8\"/><path d=\"M12 7v5l3 2\"/>"
        ),
    ),
    (
        "settings",
        stroke_icon!("1.9", "<circle cx=\"12\" cy=\"12\" r=\"3\"/><path d=\"M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9c.2.61.76 1.05 1.42 1.09H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z\"/>"),
    ),
    (
        "desktop",
        stroke_icon!("1.9", "<rect x=\"2\" y=\"4\" width=\"20\" height=\"13\" rx=\"2\"/><path d=\"M8 21h8M12 17v4\"/>"),
    ),
    (
        "tablet",
        stroke_icon!("1.9", "<rect x=\"5\" y=\"2\" width=\"14\" height=\"20\" rx=\"2\"/><path d=\"M11 18h2\"/>"),
    ),
    (
        "mobile",
        stroke_icon!("1.9", "<rect x=\"7\" y=\"2\" width=\"10\" height=\"20\" rx=\"2.5\"/><path d=\"M11 18h2\"/>"),
    ),
    (
        "undo",
        stroke_icon!("2", "<path d=\"M9 14L4 9l5-5\"/><path d=\"M4 9h11a6 6 0 0 1 0 12h-4\"/>"),
    ),
    (
        "redo",
        stroke_icon!("2", "<path d=\"M15 14l5-5-5-5\"/><path d=\"M20 9H9a6 6 0 0 0 0 12h4\"/>"),
    ),
    (
        "check-circle",
        stroke_icon!("2", "<circle cx=\"12\" cy=\"12\" r=\"9\"/><path d=\"M8 12l3 3 5-6\"/>"),
    ),
    (
        "paperclip",
        stroke_icon!("1.8", "<path d=\"M21.4 11.05 12.25 20.2a5 5 0 0 1-7.07-7.07l9.19-9.19a3 3 0 0 1 4.24 4.24l-9.2 9.19a1 1 0 0 1-1.41-1.41l8.49-8.49\"/>"),
    ),
    (
        "skill",
        stroke_icon!("1.8", "<path d=\"M12 2 3 7v10l9 5 9-5V7z\"/><path d=\"M3 7l9 5 9-5\"/><path d=\"M12 22V12\"/>"),
    ),
    ("plus", stroke_icon!("2.2", "<path d=\"M3 12h18M12 3v18\"/>")),
    ("send", stroke_icon!("2", "<path d=\"M5 12h14M13 6l6 6-6 6\"/>")),
    ("stop", fill_icon!("<rect x=\"6\" y=\"6\" width=\"12\" height=\"12\" rx=\"2.5\"/>")),
    (
        "alert-circle",
        stroke_icon!("2", "<circle cx=\"12\" cy=\"12\" r=\"9\"/><path d=\"M12 8v4M12 16h.01\"/>"),
    ),
    ("check", stroke_icon!("3", "<path d=\"M4 12l5 5L20 6\"/>")),
    ("close", stroke_icon!("2", "<path d=\"M6 6l12 12M18 6L6 18\"/>")),
    (
        "lock",
        stroke_icon!("1.9", "<rect x=\"3\" y=\"11\" width=\"18\" height=\"11\" rx=\"2\"/><path d=\"M7 11V7a5 5 0 0 1 10 0v4\"/>"),
    ),
    ("chevron-right", stroke_icon!("2", "<path d=\"M9 6l6 6-6 6\"/>")),
    (
        "wifi-off",
        stroke_icon!("1.9", "<path d=\"M1 1l22 22\"/><path d=\"M8.5 4.7A11 11 0 0 1 21 8\"/><path d=\"M3 8a11 11 0 0 1 3.2-2.3\"/><path d=\"M6.3 11.3A6 6 0 0 1 9 9.9\"/><path d=\"M17.5 11.5A6 6 0 0 0 14 10\"/><path d=\"M9.5 14.6a2.5 2.5 0 0 1 5 0\"/><path d=\"M12 20h.01\"/>"),
    ),
    (
        "refresh",
        stroke_icon!("2", "<path d=\"M3 3v5h5\"/><path d=\"M3.05 13A9 9 0 1 0 6 5.3L3 8\"/>"),
    ),
    (
        "alert-triangle",
        stroke_icon!("1.9", "<path d=\"M12 9v4M12 17h.01\"/><path d=\"M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z\"/>"),
    ),
    // `gpui_component::TitleBar`'s built-in window controls (client-side
    // decorations) look these up by name; without them the buttons render blank.
    ("window-minimize", stroke_icon!("1.9", "<path d=\"M5 12h14\"/>")),
    (
        "window-maximize",
        stroke_icon!("1.9", "<rect x=\"5\" y=\"5\" width=\"14\" height=\"14\" rx=\"1.5\"/>"),
    ),
    (
        "window-restore",
        stroke_icon!(
            "1.9",
            "<rect x=\"8\" y=\"4\" width=\"12\" height=\"12\" rx=\"1.5\"/><path d=\"M4 8v12h12v-4\"/>"
        ),
    ),
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
