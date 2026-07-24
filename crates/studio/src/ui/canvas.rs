//! Center canvas. The generated website renders in the **embedded webview**
//! (built as a child of the gpui window in `crate::app`), sized to the device
//! frame. Because webkit paints above GPUI content regardless of order, its
//! frame is only laid out when nothing must cover the canvas; the actual
//! show()/hide() is centralized in [`crate::ui::render`].

use gpui::{Context, SharedString, Window, div, prelude::*, px};
use gpui_component::{StyledExt, h_flex, v_flex};

use crate::app::StudioApp;
use crate::state::{Device, Dir};
use crate::theme;
use crate::ui::widgets::icon;

pub fn render(app: &StudioApp, _window: &mut Window, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let show_empty = !app.generated && !app.busy;
    let show_preview = app.generated && !app.busy && app.modal.is_none();

    div()
        .relative()
        .size_full()
        .bg(theme::bg_sunken())
        .when(show_preview, |d| d.child(preview_frame(app)))
        .when(show_empty, |d| d.child(empty_state(app, cx)))
        .when(app.busy, |d| d.child(compiling_overlay(app)))
        .when(show_preview && app.review_open, |d| d.child(review_labels(app)))
}

fn preview_frame(app: &StudioApp) -> impl IntoElement {
    let max_w = match app.device {
        Device::Desktop => 1200.0,
        Device::Tablet => 788.0,
        Device::Mobile => 392.0,
    };
    div()
        .absolute()
        .inset_0()
        .flex()
        .justify_center()
        .overflow_hidden()
        .p(px(22.0))
        .child(
            div()
                .w_full()
                .max_w(px(max_w))
                .h_full()
                .rounded(px(theme::RADIUS_LG))
                .overflow_hidden()
                .border_1()
                .border_color(theme::line())
                .bg(theme::site_bg())
                .shadow(theme::shadow_pop())
                .children(app.preview.as_ref().cloned()),
        )
}

fn empty_state(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let rtl = app.dir == Dir::Rtl;
    let sugg: [&str; 3] = if rtl {
        [
            "\u{645}\u{642}\u{647}\u{649} \u{633}\u{637}\u{62d} \u{644}\u{644}\u{645}\u{648}\u{633}\u{64a}\u{642}\u{649} \u{627}\u{644}\u{62d}\u{64a}\u{651}\u{629} \u{641}\u{64a} \u{627}\u{644}\u{631}\u{64a}\u{627}\u{636}",
            "\u{635}\u{641}\u{62d}\u{629} \u{645}\u{642}\u{647}\u{649} \u{645}\u{639} \u{642}\u{627}\u{626}\u{645}\u{629} \u{648}\u{645}\u{648}\u{627}\u{639}\u{64a}\u{62f}",
            "\u{628}\u{648}\u{631}\u{62a}\u{641}\u{648}\u{644}\u{64a}\u{648} \u{644}\u{645}\u{635}\u{648}\u{651}\u{631}",
        ]
    } else {
        ["A rooftop live-music venue in Riyadh", "A caf\u{e9} landing with menu & hours", "A photographer\u{2019}s portfolio"]
    };

    div().absolute().inset_0().flex().items_center().justify_center().p(px(40.0)).child(
        v_flex()
            .items_center()
            .max_w(px(460.0))
            .child(div().size(px(64.0)).rounded(px(18.0)).bg(theme::accent_grad()).shadow(theme::glow_violet()).flex().items_center().justify_center().child(icon("sparkle", 26.0, theme::accent_contrast())))
            .child(div().mt(px(20.0)).font_family(theme::FONT_DISPLAY).text_size(px(21.0)).font_semibold().text_color(theme::text_strong()).child("A blank canvas, ready when you are"))
            .child(div().mt(px(8.0)).text_size(px(14.0)).text_color(theme::text_muted()).text_center().line_height(px(22.0)).child("Describe your site in the assistant, or start with one of these \u{2014} the preview builds here live."))
            .child(v_flex().mt(px(22.0)).w_full().gap(px(9.0)).children(sugg.into_iter().enumerate().map(|(i, s)| suggestion_row(i, s, cx)))),
    )
}

fn suggestion_row(i: usize, text: &str, cx: &mut Context<StudioApp>) -> impl IntoElement + use<> {
    let owned: SharedString = text.to_string().into();
    let for_click = owned.clone();
    h_flex()
        .id(("cansugg", i))
        .items_center()
        .gap(px(11.0))
        .p(px(13.0))
        .rounded(px(theme::RADIUS_MD))
        .border_1()
        .border_color(theme::line())
        .bg(theme::bg_panel())
        .text_size(px(13.5))
        .text_color(theme::text_soft())
        .cursor_pointer()
        .hover(|s| s.border_color(theme::accent_ring()).bg(theme::accent_tint()).text_color(theme::text_strong()))
        .child(icon("sparkle", 15.0, theme::accent()))
        .child(div().flex_1().child(owned))
        .child(icon("arrow-right", 15.0, theme::text_faint()))
        .on_click(cx.listener(move |a, _, window, cx| a.run_suggestion(&for_click, window, cx)))
}

fn compiling_overlay(app: &StudioApp) -> impl IntoElement {
    div().absolute().inset_0().flex().items_center().justify_center().bg(theme::hexa(0x070809b8)).child(
        v_flex()
            .items_center()
            .child(div().size(px(54.0)).rounded(px(16.0)).bg(theme::accent_grad()).shadow(theme::glow_violet()).flex().items_center().justify_center().child(icon("sparkle", 26.0, theme::accent_contrast())))
            .child(div().mt(px(16.0)).text_size(px(14.0)).font_semibold().text_color(theme::text_strong()).child(SharedString::from(app.compile_text())))
            .child(div().mt(px(6.0)).font_family(theme::FONT_MONO).text_size(px(12.0)).text_color(theme::text_muted()).child(SharedString::from(app.compile_sub()))),
    )
}

/// BEFORE / AFTER labels over the preview during review. The real wipe is the
/// cursor-driven `clip-path` split inside the diff shell (§4.1, `wf_preview::DIFF_SHELL`),
/// which also bakes in its own labels; these GPUI tags are the fallback for when
/// the webview isn't embedded (they don't composite over the child X11 surface).
fn review_labels(_app: &StudioApp) -> impl IntoElement {
    div()
        .absolute()
        .inset_0()
        .child(div().absolute().top(px(34.0)).left(px(36.0)).px(px(9.0)).py(px(4.0)).rounded(px(theme::RADIUS_XS)).bg(theme::black(0.7)).text_size(px(10.5)).font_bold().text_color(theme::text_muted()).child("BEFORE"))
        .child(div().absolute().top(px(34.0)).right(px(36.0)).px(px(9.0)).py(px(4.0)).rounded(px(theme::RADIUS_XS)).bg(theme::accent_tint()).text_size(px(10.5)).font_bold().text_color(theme::accent()).child("AFTER"))
}
