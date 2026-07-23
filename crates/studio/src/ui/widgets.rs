//! Small shared building blocks for the Studio chrome, plus the reusable
//! design-system components (`Button`, `Avatar`, `ProviderCard`, …) ported from
//! the Claude Design project's component kit.

use gpui::{App, ClickEvent, ElementId, Hsla, SharedString, Svg, Window, div, prelude::*, px, svg};
use gpui_component::{StyledExt, h_flex, v_flex};

use crate::theme;

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

/// A filled circle — status/presence/timeline dots.
pub fn dot(size: f32, color: Hsla) -> gpui::Div {
    div().size(px(size)).rounded_full().bg(color).flex_none()
}

/// The rounded logo tile — a blue→violet gradient with a violet glow and a
/// near-black sparkle (chat header, empty state, onboarding, title bar).
pub fn brand_badge(size: f32, glyph: f32) -> gpui::Div {
    div()
        .size(px(size))
        .rounded(px(size * 0.3))
        .bg(theme::accent_grad())
        .shadow(theme::glow_violet())
        .flex()
        .items_center()
        .justify_center()
        .flex_none()
        .child(icon("sparkle", glyph, theme::accent_contrast()))
}

// ── Button ──────────────────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    Sm,
    Md,
}

/// A design-system button. Build it fluently, then `.render(id, on_click)`:
/// `Btn::primary("Sign in").full().loading(busy).render("signin", cx.listener(…))`.
pub struct Btn {
    label: SharedString,
    variant: ButtonVariant,
    size: ButtonSize,
    icon: Option<&'static str>,
    icon_right: Option<&'static str>,
    full: bool,
    grow: bool,
    loading: bool,
    disabled: bool,
}

impl Btn {
    fn new(label: impl Into<SharedString>, variant: ButtonVariant) -> Self {
        Self { label: label.into(), variant, size: ButtonSize::Md, icon: None, icon_right: None, full: false, grow: false, loading: false, disabled: false }
    }
    pub fn primary(label: impl Into<SharedString>) -> Self {
        Self::new(label, ButtonVariant::Primary)
    }
    pub fn secondary(label: impl Into<SharedString>) -> Self {
        Self::new(label, ButtonVariant::Secondary)
    }
    pub fn sm(mut self) -> Self {
        self.size = ButtonSize::Sm;
        self
    }
    pub fn icon(mut self, name: &'static str) -> Self {
        self.icon = Some(name);
        self
    }
    pub fn icon_right(mut self, name: &'static str) -> Self {
        self.icon_right = Some(name);
        self
    }
    pub fn full(mut self) -> Self {
        self.full = true;
        self
    }
    /// Share a row equally with its siblings (flex-1). Use for two buttons in a
    /// row instead of `full` (which would make each demand 100% and overflow).
    pub fn grow(mut self) -> Self {
        self.grow = true;
        self
    }
    pub fn loading(mut self, v: bool) -> Self {
        self.loading = v;
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }

    pub fn render(self, id: impl Into<ElementId>, on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> gpui::Stateful<gpui::Div> {
        let inert = self.disabled || self.loading;
        let (h, px_pad, fs, gap, glyph) = match self.size {
            ButtonSize::Sm => (32.0, 13.0, 12.5, 6.0, 14.0),
            ButtonSize::Md => (40.0, 17.0, 14.0, 8.0, 15.0),
        };
        let (bg_normal, fg, border, hover_bg): (Option<gpui::Background>, Hsla, Option<Hsla>, Hsla) = match self.variant {
            ButtonVariant::Primary => (Some(theme::accent().into()), theme::accent_contrast(), None, theme::accent_hover()),
            ButtonVariant::Secondary => (Some(theme::bg_raised().into()), theme::text_soft(), Some(theme::line_strong()), theme::bg_hover()),
        };

        let mut b = h_flex()
            .id(id)
            .h(px(h))
            .px(px(px_pad))
            .gap(px(gap))
            .items_center()
            .justify_center()
            .rounded(px(theme::RADIUS_SM))
            .text_size(px(fs))
            .font_semibold()
            .text_color(fg)
            .flex_none();
        if self.full {
            b = b.w_full();
        }
        if self.grow {
            b = b.flex_1().min_w_0();
        }
        if let Some(bg) = bg_normal {
            b = b.bg(bg);
        }
        if let Some(bc) = border {
            b = b.border_1().border_color(bc);
        }
        if self.variant == ButtonVariant::Primary {
            b = b.shadow(theme::glow_accent());
        }
        if inert {
            b = b.opacity(0.55).cursor_default();
        } else {
            let hb = hover_bg;
            b = b.cursor_pointer().hover(move |s| s.bg(hb)).on_click(on_click);
        }
        if self.loading {
            b = b.child(icon("loader", glyph, fg));
        } else if let Some(name) = self.icon {
            b = b.child(icon(name, glyph, fg));
        }
        b = b.child(div().child(self.label));
        if let Some(name) = self.icon_right {
            b = b.child(icon(name, glyph, fg));
        }
        b
    }
}

// ── Avatar ──────────────────────────────────────────────────────────────────
#[derive(Clone, Copy)]
pub enum Tone {
    Blue,
    Violet,
    Teal,
}

impl Tone {
    fn color(self) -> Hsla {
        match self {
            Tone::Blue => theme::tone_blue(),
            Tone::Violet => theme::tone_violet(),
            Tone::Teal => theme::tone_teal(),
        }
    }
    fn tint(self) -> Hsla {
        match self {
            Tone::Blue => theme::hexa(0x7FB0EE24),
            Tone::Violet => theme::violet_tint(),
            Tone::Teal => theme::hexa(0x5CCB9A24),
        }
    }
}

/// A circular avatar with initials. `gradient` fills it with the brand
/// blue→violet gradient; otherwise it uses the `tone` tint. `online` adds a
/// presence dot.
pub fn avatar(initials: impl Into<SharedString>, tone: Tone, gradient: bool, online: bool, size: f32) -> impl IntoElement {
    let mut tile = h_flex()
        .size(px(size))
        .rounded_full()
        .items_center()
        .justify_center()
        .flex_none()
        .text_size(px(size * 0.4))
        .font_semibold()
        .font_family(theme::FONT_DISPLAY)
        .child(initials.into());
    if gradient {
        tile = tile.bg(theme::accent_grad()).text_color(theme::accent_contrast());
    } else {
        tile = tile.bg(tone.tint()).text_color(tone.color());
    }
    div()
        .relative()
        .size(px(size))
        .flex_none()
        .child(tile)
        .when(online, |d| {
            d.child(
                div()
                    .absolute()
                    .bottom_0()
                    .right_0()
                    .size(px(size * 0.3))
                    .rounded_full()
                    .bg(theme::success())
                    .border_2()
                    .border_color(theme::bg_panel()),
            )
        })
}

// ── ProviderCard ────────────────────────────────────────────────────────────
/// An AI-provider option card (onboarding provider grid).
pub fn provider_card(
    id: impl Into<ElementId>,
    mono: &'static str,
    mono_bg: u32,
    name: &'static str,
    by: &'static str,
    selected: bool,
    recommended: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    h_flex()
        .id(id)
        .w_full()
        .relative()
        .items_center()
        .gap(px(11.0))
        .p(px(14.0))
        .rounded(px(theme::RADIUS_LG))
        .border_1()
        .border_color(if selected { theme::accent_ring() } else { theme::line() })
        .bg(if selected { theme::accent_tint() } else { theme::bg_panel() })
        .cursor_pointer()
        .hover(|s| s.border_color(theme::line_bright()))
        .on_click(on_click)
        .child(
            div()
                .size(px(34.0))
                .rounded(px(theme::RADIUS_SM))
                .bg(theme::hex(mono_bg))
                .flex()
                .items_center()
                .justify_center()
                .text_color(theme::white(1.0))
                .font_bold()
                .font_family(theme::FONT_DISPLAY)
                .text_size(px(14.0))
                .child(mono),
        )
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .child(div().text_size(px(14.5)).font_semibold().text_color(theme::text_strong()).child(name))
                .child(div().text_size(px(12.0)).text_color(theme::text_muted()).child(by)),
        )
        .when(recommended, |d| {
            d.child(
                div()
                    .absolute()
                    .bottom(px(8.0))
                    .right(px(10.0))
                    .px(px(6.0))
                    .py(px(2.0))
                    .rounded(px(theme::RADIUS_XS))
                    .bg(theme::accent_tint())
                    .text_color(theme::accent())
                    .text_size(px(9.0))
                    .font_bold()
                    .child("PICK"),
            )
        })
}
