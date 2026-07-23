//! WebFluent Studio design tokens — the **cinematic** redesign.
//!
//! Ported from the product design system (`_ds/…/tokens/*.css` in the Claude
//! Design project): a cool, near-black cinematic palette with a soft **blue**
//! primary accent and a **violet** secondary, glossy blue→violet gradients and
//! colored glows. Everything the UI colors, spaces, or labels resolves through
//! here rather than hard-coding hex, so a design revision is a one-file change.
//!
//! The canonical functions mirror the CSS custom properties (`--wf-*`). The
//! block of *compatibility aliases* at the bottom keeps the older token names
//! (from the warm-café port) resolving to their nearest cinematic value, so the
//! not-yet-rewritten view modules recolor in place and the crate stays green.
//!
//! A design-token module deliberately defines the whole palette up front — the
//! rewrite (phases 2–6) consumes the rest — so unused tokens are expected here.
#![allow(dead_code)]

use gpui::{Background, BoxShadow, Hsla, hsla, linear_color_stop, linear_gradient, point, px, rgb, rgba};

/// Opaque color from a `0xRRGGBB` literal — keeps the design's hex codes as-is.
pub fn hex(v: u32) -> Hsla {
    rgb(v).into()
}
/// Translucent color from a `0xRRGGBBAA` literal.
pub fn hexa(v: u32) -> Hsla {
    rgba(v).into()
}
/// White at a given alpha — hairlines/overlays are all `rgba(255,…)`.
pub fn white(alpha: f32) -> Hsla {
    hsla(0.0, 0.0, 1.0, alpha)
}
/// Black at a given alpha — scrims and shadows.
pub fn black(alpha: f32) -> Hsla {
    hsla(0.0, 0.0, 0.0, alpha)
}

// ── Surfaces (cool near-blacks) ─────────────────────────────────────────────
/// True-black desktop backdrop / page.
pub fn bg_void() -> Hsla {
    hex(0x000000)
}
/// Deepest — insets, inputs, wells.
pub fn bg_sunken() -> Hsla {
    hex(0x0A0B0E)
}
/// App window base.
pub fn bg_base() -> Hsla {
    hex(0x0D0E12)
}
/// Toolbar strip.
pub fn bg_toolbar() -> Hsla {
    hex(0x111319)
}
/// Side panels, modals.
pub fn bg_panel() -> Hsla {
    hex(0x14161C)
}
/// Chips, rows, raised cards on panels.
pub fn bg_raised() -> Hsla {
    hex(0x1B1E25)
}
/// Row / ghost hover.
pub fn bg_hover() -> Hsla {
    hex(0x232733)
}

// ── Text (cool neutrals) ────────────────────────────────────────────────────
/// Primary text / headings.
pub fn text_strong() -> Hsla {
    hex(0xF4F6FB)
}
/// Default body.
pub fn text_body() -> Hsla {
    hex(0xDCE1EA)
}
/// Secondary controls.
pub fn text_soft() -> Hsla {
    hex(0xC1C7D3)
}
/// Supporting copy.
pub fn text_muted() -> Hsla {
    hex(0x99A0AE)
}
/// Captions, metadata.
pub fn text_caption() -> Hsla {
    hex(0x6E7482)
}
/// Placeholder, disabled, eyebrows.
pub fn text_faint() -> Hsla {
    hex(0x494F5C)
}
/// Resting icon color.
pub fn icon_color() -> Hsla {
    hex(0x99A0AE)
}

// ── Blue accent (primary / brand) ───────────────────────────────────────────
pub fn accent() -> Hsla {
    hex(0x93C0F2)
}
pub fn accent_hover() -> Hsla {
    hex(0xAFD2F7)
}
/// Deeper — gradient start, pressed.
pub fn accent_strong() -> Hsla {
    hex(0x6FA6EC)
}
/// Light-fill button + accent text on dark.
pub fn accent_soft() -> Hsla {
    hex(0xBAD7F8)
}
/// Navy text ON light-blue fills.
pub fn accent_contrast() -> Hsla {
    hex(0x08131F)
}
/// Accent chip / soft fill (~0.14).
pub fn accent_tint() -> Hsla {
    hexa(0x93C0F224)
}
/// Focus / selected border (~0.55).
pub fn accent_ring() -> Hsla {
    hexa(0x93C0F28C)
}

// ── Violet secondary ────────────────────────────────────────────────────────
pub fn violet() -> Hsla {
    hex(0x8A6DF2)
}
pub fn violet_hover() -> Hsla {
    hex(0xA48EF6)
}
pub fn violet_strong() -> Hsla {
    hex(0x6D4FE0)
}
/// Violet text on dark.
pub fn violet_soft() -> Hsla {
    hex(0xC0B0F8)
}
pub fn violet_tint() -> Hsla {
    hexa(0x8A6DF229)
}
pub fn violet_ring() -> Hsla {
    hexa(0x8A6DF28C)
}

// ── Semantic (cool-tuned) ───────────────────────────────────────────────────
pub fn success() -> Hsla {
    hex(0x5CCB9A)
}
pub fn success_tint() -> Hsla {
    hexa(0x5CCB9A21)
}
pub fn warning() -> Hsla {
    hex(0xE9BE6A)
}
pub fn warning_tint() -> Hsla {
    hexa(0xE9BE6A24)
}
pub fn danger() -> Hsla {
    hex(0xEF7A85)
}
pub fn danger_tint() -> Hsla {
    hexa(0xEF7A8529)
}
/// "Structure" edit-kind tag (cool teal).
pub fn structure() -> Hsla {
    hex(0x7FB3B0)
}

// ── Decorative avatar / presence tones ──────────────────────────────────────
pub fn tone_blue() -> Hsla {
    hex(0x7FB0EE)
}
pub fn tone_violet() -> Hsla {
    hex(0x8A6DF2)
}
pub fn tone_teal() -> Hsla {
    hex(0x5CCB9A)
}

// ── Hairline borders (on dark) ──────────────────────────────────────────────
pub fn line_faint() -> Hsla {
    white(0.05)
}
pub fn line() -> Hsla {
    white(0.07)
}
pub fn line_strong() -> Hsla {
    white(0.10)
}
pub fn line_bright() -> Hsla {
    white(0.15)
}

// ── Output surface — the "generated site" theme (also cinematic dark) ────────
pub fn site_bg() -> Hsla {
    hex(0x070809)
}
pub fn site_surface() -> Hsla {
    hex(0x14161C)
}
pub fn site_elevated() -> Hsla {
    hex(0x1B1E25)
}
pub fn site_ink() -> Hsla {
    hex(0xF4F6FB)
}
pub fn site_muted() -> Hsla {
    hex(0xA3AAB8)
}
pub fn site_faint() -> Hsla {
    hex(0x6E7482)
}
pub fn site_line() -> Hsla {
    white(0.08)
}

// ── Corner radii (px), matching `--wf-radius-*` ─────────────────────────────
pub const RADIUS_XS: f32 = 6.0;
pub const RADIUS_SM: f32 = 9.0;
pub const RADIUS_MD: f32 = 11.0;
pub const RADIUS_LG: f32 = 14.0;
pub const RADIUS_XL: f32 = 18.0;
pub const RADIUS_2XL: f32 = 22.0;
pub const RADIUS_3XL: f32 = 26.0;
pub const RADIUS_PILL: f32 = 9999.0;

// ── Signature gradients (blue → violet, cinematic) ──────────────────────────
/// Logo tiles, hero CTAs — `linear-gradient(135deg, #93C0F2, #8A6DF2)`.
pub fn accent_grad() -> Background {
    linear_gradient(135.0, linear_color_stop(accent(), 0.0), linear_color_stop(violet(), 1.0))
}
/// Deeper glossy fill — `linear-gradient(135deg, #6FA6EC, #6D4FE0)`.
pub fn accent_grad_soft() -> Background {
    linear_gradient(135.0, linear_color_stop(accent_strong(), 0.0), linear_color_stop(violet_strong(), 1.0))
}

// ── Colored glows (the cinematic signature) ─────────────────────────────────
/// Blue button / focus bloom — `0 8px 30px -6px rgba(147,192,242,0.45)`.
pub fn glow_accent() -> Vec<BoxShadow> {
    vec![BoxShadow { color: hexa(0x93C0F273), offset: point(px(0.0), px(8.0)), blur_radius: px(30.0), spread_radius: px(-6.0) }]
}
/// Violet accent bloom — `0 8px 30px -6px rgba(138,109,242,0.5)`.
pub fn glow_violet() -> Vec<BoxShadow> {
    vec![BoxShadow { color: hexa(0x8A6DF280), offset: point(px(0.0), px(8.0)), blur_radius: px(30.0), spread_radius: px(-6.0) }]
}
/// Menu / popover elevation — `0 16px 40px -12px rgba(0,0,0,0.7)`.
pub fn shadow_pop() -> Vec<BoxShadow> {
    vec![BoxShadow { color: black(0.7), offset: point(px(0.0), px(16.0)), blur_radius: px(40.0), spread_radius: px(-12.0) }]
}
/// Modal elevation — `0 34px 80px -22px rgba(0,0,0,0.75)`.
pub fn shadow_modal() -> Vec<BoxShadow> {
    vec![BoxShadow { color: black(0.75), offset: point(px(0.0), px(34.0)), blur_radius: px(80.0), spread_radius: px(-22.0) }]
}

// ── Fonts ───────────────────────────────────────────────────────────────────
/// Display / headings / product name / big numerals.
pub const FONT_DISPLAY: &str = "Space Grotesk";
/// UI + body (Latin) — geometric, modern, cool.
pub const FONT_UI: &str = "Manrope";
/// RTL Arabic UI + content (falls back to Noto Sans Arabic where unbundled).
pub const FONT_ARABIC: &str = "IBM Plex Sans Arabic";
/// Keys, commands, URLs.
pub const FONT_MONO: &str = "monospace";

/// Edit-kind accent (Review chips) → `(foreground, tinted bg)`.
pub fn chip_kind(kind: crate::state::ChipKind) -> (Hsla, Hsla) {
    use crate::state::ChipKind::*;
    match kind {
        Text => (accent(), accent_tint()),
        Structure => (structure(), hexa(0x7FB3B029)),
        // Style / Behavior share the violet family.
        Style | Behavior => (violet_soft(), violet_tint()),
    }
}

// ════════════════════════════════════════════════════════════════════════════
// Compatibility aliases — older token names used by the not-yet-rewritten view
// modules, repointed to their nearest cinematic value. Removed as each module
// is rewritten onto the canonical names above.
// ════════════════════════════════════════════════════════════════════════════
pub fn window_bg() -> Hsla {
    bg_base()
}
pub fn panel() -> Hsla {
    bg_panel()
}
pub fn toolbar() -> Hsla {
    bg_toolbar()
}
pub fn sunken() -> Hsla {
    bg_sunken()
}
pub fn canvas() -> Hsla {
    bg_sunken()
}
pub fn elevated() -> Hsla {
    bg_raised()
}
pub fn seg_active() -> Hsla {
    bg_hover()
}
pub fn raised() -> Hsla {
    bg_raised()
}
pub fn ob_rail() -> Hsla {
    bg_base()
}
pub fn bubble() -> Hsla {
    bg_panel()
}
pub fn toast_bg() -> Hsla {
    bg_raised()
}
pub fn text() -> Hsla {
    text_strong()
}
pub fn text_dim() -> Hsla {
    text_muted()
}
pub fn muted() -> Hsla {
    text_muted()
}
pub fn faint() -> Hsla {
    text_caption()
}
pub fn disabled() -> Hsla {
    text_faint()
}
/// Legacy warm-terracotta "deep accent" → cinematic deeper blue.
pub fn accent_deep() -> Hsla {
    accent_strong()
}
pub fn warn() -> Hsla {
    warning()
}
pub fn warn_soft() -> Hsla {
    warning()
}
/// Legacy skills "gold" → amber.
pub fn gold() -> Hsla {
    warning()
}
pub fn gold_soft() -> Hsla {
    warning()
}
pub fn tl_red() -> Hsla {
    danger()
}
pub fn tl_amber() -> Hsla {
    warning()
}
pub fn tl_green() -> Hsla {
    success()
}
pub fn hairline() -> Hsla {
    line()
}
pub fn border_strong() -> Hsla {
    line_bright()
}
