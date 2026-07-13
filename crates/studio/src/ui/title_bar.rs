//! Top title bar: brand, the compile-status pill (FR-13), and the
//! history/settings buttons. Uses gpui-component's `TitleBar` for the native
//! window-drag region.

use gpui::{Context, Hsla, Window, div, prelude::*, px};
use gpui_component::{StyledExt, TitleBar, h_flex};

use crate::app::StudioApp;
use crate::state::{Screen, Status};
use crate::theme;
use crate::ui::widgets::{dot, icon};

pub fn render(app: &StudioApp, _window: &mut Window, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let onboarding = app.screen == Screen::Onboarding;

    TitleBar::new()
        .bg(theme::panel())
        .border_0()
        .child(
        h_flex()
            
            .flex_1()
            .items_center()
            .gap(px(14.0))
            .px(px(15.0))
            .child(vsep())
            .child(
                h_flex()
                    .items_baseline()
                    .gap(px(9.0))
                    .child(
                        div()
                            .font_family(theme::FONT_DISPLAY)
                            .font_semibold()
                            .text_size(px(15.0))
                            .child("WebFluent Studio"),
                    )
                    .child(div().text_size(px(13.0)).text_color(theme::faint()).child("Yasmine Caf\u{e9}")),
            )
            .child(div().flex_1())
            .when(!onboarding, |this| {
                this.child(status_pill(app, cx))
                    .child(square_btn("hist", "history", app.show_history, cx.listener(|a, _, _, cx| a.toggle_history(cx))))
                    .child(square_btn("settings", "settings", app.show_settings, cx.listener(|a, _, _, cx| a.toggle_settings(cx))))
            }),
    )
}

fn vsep() -> impl IntoElement {
    div().w(px(1.0)).h(px(20.0)).bg(theme::white(0.09))
}

/// The status button: dot + label, opens the activity popover.
fn status_pill(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let (label, color) = app.status.label_color(app.generated);
    let (bg, border) = status_tones(app.status, app.busy);

    h_flex()
        .id("status")
        .items_center()
        .gap(px(8.0))
        .h(px(30.0))
        .px(px(12.0))
        .rounded(px(8.0))
        .border_1()
        .border_color(border)
        .bg(bg)
        .cursor_pointer()
        .child(dot(8.0, color))
        .child(div().text_size(px(12.5)).font_semibold().text_color(color).child(label))
        .on_click(cx.listener(|a, _, _, cx| a.toggle_activity(cx)))
}

fn status_tones(status: Status, busy: bool) -> (Hsla, Hsla) {
    match status {
        Status::Error => (theme::hexa(0xec6a5e24), theme::hexa(0xec6a5e66)),
        Status::Compiled => (theme::hexa(0x63c0881f), theme::hexa(0x63c0884d)),
        Status::Attention | Status::SelfHeal => (theme::hexa(0xe5a54b1f), theme::hexa(0xe5a54b4d)),
        _ if busy => (theme::hexa(0xe2725b1f), theme::hexa(0xe2725b4d)),
        _ => (theme::white(0.04), theme::white(0.08)),
    }
}

fn square_btn(
    id: &'static str,
    icon_name: &'static str,
    active: bool,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .size(px(30.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(8.0))
        .border_1()
        .border_color(theme::white(0.08))
        .bg(if active { theme::seg_active() } else { gpui::transparent_black() })
        .cursor_pointer()
        .hover(|s| s.bg(theme::seg_active()))
        .child(icon(icon_name, 16.0, theme::text_dim()))
        .on_click(on_click)
}
