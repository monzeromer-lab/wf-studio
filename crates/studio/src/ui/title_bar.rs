//! Top title bar. Always shows the brand; the right-hand cluster varies per
//! screen (Home, Design-system workspace, Workspace). Uses gpui-component's
//! `TitleBar` for the native window-drag region + window controls.

use gpui::{App, ClickEvent, Context, Hsla, Window, div, prelude::*, px};
use gpui_component::{StyledExt, TitleBar, h_flex};

use crate::app::StudioApp;
use crate::state::{Modal, Screen, Status};
use crate::theme;
use crate::ui::widgets::{avatar, dot, icon};

pub fn render(app: &StudioApp, _window: &mut Window, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let in_project = matches!(app.screen, Screen::Workspace | Screen::DsWorkspace);
    let proj_name = app.current_project().map(|p| p.name.clone());

    let mut left = h_flex().items_center().gap(px(9.0)).min_w_0();
    if in_project {
        left = left
            .child(
                h_flex()
                    .id("back-projects")
                    .items_center()
                    .gap(px(6.0))
                    .h(px(28.0))
                    .pl(px(8.0))
                    .pr(px(10.0))
                    .rounded(px(theme::RADIUS_SM))
                    .border_1()
                    .border_color(theme::line())
                    .text_size(px(12.5))
                    .font_semibold()
                    .text_color(theme::text_soft())
                    .cursor_pointer()
                    .hover(|s| s.bg(theme::bg_hover()).text_color(theme::text_strong()))
                    .child(icon("arrow-left", 15.0, theme::text_soft()))
                    .child("Projects")
                    .on_click(cx.listener(|a, _, _, cx| a.request_exit(cx))),
            )
            .child(vsep());
    }
    left = left
        .child(crate::ui::widgets::brand_badge(26.0, 15.0))
        .child(div().font_family(theme::FONT_DISPLAY).font_semibold().text_size(px(14.0)).text_color(theme::text_strong()).child("WebFluent Studio"));
    if let Some(name) = proj_name {
        left = left
            .child(div().text_size(px(13.0)).text_color(theme::text_faint()).child("\u{b7}"))
            .child(div().text_size(px(13.0)).text_color(theme::text_muted()).child(name));
    }

    TitleBar::new().bg(theme::bg_toolbar()).border_0().child(
        h_flex()
            .flex_1()
            .items_center()
            .px(px(6.0))
            .child(left)
            .child(div().flex_1())
            .child(right_cluster(app, cx)),
    )
}

fn right_cluster(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let mut row = h_flex().items_center().gap(px(8.0));
    // No studio backend/account yet — surface that we're running locally, beside
    // the compile-status/activity log.
    if app.offline {
        row = row.child(offline_pill());
    }
    match app.screen {
        Screen::Home => {
            row = row
                .child(icon_btn("tb-settings", "settings", cx.listener(|a, _, _, cx| a.open_modal(Modal::Settings, cx))))
                .child(profile(cx));
        }
        Screen::DsWorkspace => {
            row = row
                .child(icon_btn("tb-share", "share", cx.listener(|a, _, _, cx| a.open_modal(Modal::Share, cx))))
                .child(icon_btn("tb-settings", "settings", cx.listener(|a, _, _, cx| a.open_modal(Modal::Settings, cx))))
                .child(profile(cx));
        }
        Screen::Workspace => {
            row = row
                .child(status_pill(app, cx))
                .child(icon_btn("tb-history", "clock", cx.listener(|a, _, _, cx| a.open_modal(Modal::History, cx))))
                .child(icon_btn("tb-settings", "settings", cx.listener(|a, _, _, cx| a.open_modal(Modal::Settings, cx))))
                .child(vsep())
                .child(profile(cx));
        }
        Screen::Login | Screen::Onboarding => {}
    }
    row
}

/// Local/offline indicator — the studio has no backend or account yet.
fn offline_pill() -> impl IntoElement {
    h_flex()
        .items_center()
        .gap(px(6.0))
        .px(px(10.0))
        .h(px(26.0))
        .rounded_full()
        .bg(theme::bg_raised())
        .border_1()
        .border_color(theme::line())
        .child(div().size(px(6.0)).rounded_full().bg(theme::warning()))
        .child(div().text_size(px(11.5)).font_semibold().text_color(theme::text_muted()).child("Offline"))
}

fn profile(cx: &mut Context<StudioApp>) -> impl IntoElement {
    div()
        .id("tb-profile")
        .cursor_pointer()
        .child(avatar("RS", crate::ui::widgets::Tone::Violet, true, false, 30.0))
        .on_click(cx.listener(|a, _, _, cx| a.open_profile(cx)))
}

fn vsep() -> impl IntoElement {
    div().w(px(1.0)).h(px(20.0)).bg(theme::line()).mx(px(2.0))
}

fn icon_btn(
    id: &'static str,
    icon_name: &'static str,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .size(px(30.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(theme::RADIUS_SM))
        .border_1()
        .border_color(theme::line())
        .text_color(theme::icon_color())
        .cursor_pointer()
        .hover(|s| s.bg(theme::bg_hover()).text_color(theme::text_strong()))
        .child(icon(icon_name, 16.0, theme::icon_color()))
        .on_click(on_click)
}

/// Compile-status pill (opens the Activity log for now).
fn status_pill(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let (label, color) = app.status.label_color(app.generated);
    let (bg, border) = status_tones(app.status, app.busy);
    h_flex()
        .id("tb-status")
        .items_center()
        .gap(px(8.0))
        .h(px(30.0))
        .px(px(12.0))
        .rounded(px(theme::RADIUS_SM))
        .border_1()
        .border_color(border)
        .bg(bg)
        .cursor_pointer()
        .child(dot(8.0, color))
        .child(div().text_size(px(12.5)).font_semibold().text_color(color).child(label))
        .on_click(cx.listener(|a, _, _, cx| a.open_modal(Modal::Compile, cx)))
}

fn status_tones(status: Status, busy: bool) -> (Hsla, Hsla) {
    match status {
        Status::Error => (theme::danger_tint(), theme::hexa(0xEF7A8566)),
        Status::Compiled => (theme::success_tint(), theme::hexa(0x5CCB9A59)),
        _ if busy => (theme::accent_tint(), theme::accent_ring()),
        _ => (theme::white(0.04), theme::line_strong()),
    }
}
