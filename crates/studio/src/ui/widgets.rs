//! Small shared building blocks for the Studio chrome.

use gpui::{Hsla, Svg, div, prelude::*, px, svg};

/// A monochrome icon from the embedded set (`crate::assets`), tinted with
/// `color`. GPUI renders the SVG as a coverage mask, so `color` fully controls
/// the paint.
pub fn icon(name: &str, size: f32, color: Hsla) -> Svg {
    svg()
        .path(format!("icons/{name}.svg"))
        .size(px(size))
        .flex_none()
        .text_color(color)
}

/// A filled circle — status/activity/history dots and traffic lights.
pub fn dot(size: f32, color: Hsla) -> gpui::Div {
    div().size(px(size)).rounded_full().bg(color).flex_none()
}

/// The terracotta rounded logo tile with a white sparkle (chat header,
/// empty state, onboarding rail).
pub fn brand_badge(size: f32, glyph: f32) -> gpui::Div {
    div()
        .size(px(size))
        .rounded(px(size * 0.28))
        .bg(crate::theme::accent_deep())
        .flex()
        .items_center()
        .justify_center()
        .flex_none()
        .child(icon("sparkle", glyph, crate::theme::white(1.0)))
}
