//! WebFluent Studio design tokens.
//!
//! Ported from the product design mock (`docs/WebFluent Studio.dc.html`) so the
//! native GPUI chrome matches it: a warm, dark, café palette with a terracotta
//! accent. Everything the UI colors, spaces, or labels should resolve through
//! here rather than hard-coding hex, so a design revision is a one-file change.

use gpui::{Hsla, hsla, rgb, rgba};

/// Opaque color from a `0xRRGGBB` literal — keeps the mock's hex codes as-is.
pub fn hex(v: u32) -> Hsla {
    rgb(v).into()
}
/// Translucent color from a `0xRRGGBBAA` literal.
pub fn hexa(v: u32) -> Hsla {
    rgba(v).into()
}
/// White at a given alpha — the mock's hairlines/overlays are all `rgba(255,…)`.
pub fn white(alpha: f32) -> Hsla {
    hsla(0.0, 0.0, 1.0, alpha)
}
/// Black at a given alpha — scrims and shadows.
pub fn black(alpha: f32) -> Hsla {
    hsla(0.0, 0.0, 0.0, alpha)
}

// ── Surfaces ────────────────────────────────────────────────────────────────
pub fn window_bg() -> Hsla {
    hex(0x1c1917)
}
pub fn panel() -> Hsla {
    hex(0x211d1b)
}
pub fn toolbar() -> Hsla {
    hex(0x1e1a18)
}
pub fn sunken() -> Hsla {
    hex(0x14110f)
}
pub fn canvas() -> Hsla {
    hex(0x16130f)
}
pub fn elevated() -> Hsla {
    hex(0x262220)
}
pub fn seg_active() -> Hsla {
    hex(0x322c29)
}
/// Raised control face (inspector buttons, key field row).
pub fn raised() -> Hsla {
    hex(0x2a2523)
}
/// Onboarding rail.
pub fn ob_rail() -> Hsla {
    hex(0x1a1614)
}
/// Assistant message bubble.
pub fn bubble() -> Hsla {
    hex(0x2a2523)
}
/// Toast background (warm amber-brown).
pub fn toast_bg() -> Hsla {
    hex(0x2b2016)
}

// ── Text ────────────────────────────────────────────────────────────────────
pub fn text() -> Hsla {
    hex(0xf4efe9)
}
pub fn text_soft() -> Hsla {
    hex(0xd8cec6)
}
/// Icon/button foreground.
pub fn text_dim() -> Hsla {
    hex(0xc9bfb6)
}
pub fn muted() -> Hsla {
    hex(0x8b817a)
}
pub fn faint() -> Hsla {
    hex(0x6f665f)
}
/// Disabled control text.
pub fn disabled() -> Hsla {
    hex(0x4a433e)
}

// ── Accent + semantic ───────────────────────────────────────────────────────
pub fn accent() -> Hsla {
    hex(0xe2725b)
}
pub fn accent_hover() -> Hsla {
    hex(0xec8067)
}
pub fn accent_deep() -> Hsla {
    hex(0xc24328)
}
/// Soft terracotta used on tinted text (selection chip, error bubble text).
pub fn accent_soft() -> Hsla {
    hex(0xf0a48f)
}
pub fn success() -> Hsla {
    hex(0x63c088)
}
pub fn warn() -> Hsla {
    hex(0xe5a54b)
}
pub fn warn_soft() -> Hsla {
    hex(0xe5b877)
}
pub fn danger() -> Hsla {
    hex(0xec6a5e)
}
/// Skills gold.
pub fn gold() -> Hsla {
    hex(0xd9a066)
}
pub fn gold_soft() -> Hsla {
    hex(0xe0b877)
}

// Traffic-light dots.
pub fn tl_red() -> Hsla {
    hex(0xec6a5e)
}
pub fn tl_amber() -> Hsla {
    hex(0xf5bf4f)
}
pub fn tl_green() -> Hsla {
    hex(0x61c554)
}

/// Hairline separators — white at very low alpha, as in the mock.
pub fn hairline() -> Hsla {
    white(0.07)
}
/// Slightly stronger border used on popovers/cards.
pub fn border_strong() -> Hsla {
    white(0.14)
}

// ── Fonts ───────────────────────────────────────────────────────────────────
/// Display / headings.
pub const FONT_DISPLAY: &str = "Space Grotesk";
/// UI body text.
pub const FONT_UI: &str = "IBM Plex Sans";
/// Arabic UI + content (RTL). Applied once bidi mirroring of the chrome lands.
#[allow(dead_code)]
pub const FONT_ARABIC: &str = "IBM Plex Sans Arabic";

/// Chip-kind accent (FR-7 Visual Diff Review) → `(foreground, tinted bg)`.
pub fn chip_kind(kind: crate::state::ChipKind) -> (Hsla, Hsla) {
    use crate::state::ChipKind::*;
    match kind {
        Text => (gold(), hexa(0xd9a06624)),
        Structure => (hex(0x9bb06a), hexa(0x9bb06a24)),
        // Style / Behavior share the terracotta family.
        Style | Behavior => (accent_hover(), hexa(0xe2725b24)),
    }
}
