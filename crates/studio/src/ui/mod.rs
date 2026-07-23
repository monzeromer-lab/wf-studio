//! The Studio view tree. Each submodule renders one region/screen of the mock;
//! this module wires them into the window and layers modals/popovers on top.

mod canvas;
mod chat;
mod ds_workspace;
mod home;
mod login;
mod modals;
mod onboarding;
mod sidebar;
mod title_bar;
mod toolbar;
mod widgets;

use gpui::{AnyElement, ClickEvent, Context, Window, div, prelude::*, px};
use gpui_component::{h_flex, v_flex};

use crate::app::StudioApp;
use crate::state::{RightMode, Screen, Status};
use crate::theme;
use crate::ui::widgets::icon;

/// Root of the window: the title bar, the active screen, then the modal layer.
pub fn render(app: &StudioApp, window: &mut Window, cx: &mut Context<StudioApp>) -> impl IntoElement {
    // The embedded preview webview paints above all GPUI content in this window,
    // so it may only be visible on the Workspace, with a compiled site, and with
    // nothing (busy overlay / error / modal) that must cover the canvas. Toggled
    // centrally here — canvas.rs only ever runs on the Workspace, so leaving it
    // to decide would strand the webview visible over Home/Login/etc.
    if let Some(preview) = &app.preview {
        let show = app.screen == Screen::Workspace
            && app.generated
            && !app.busy
            && app.status != Status::Error
            && app.modal.is_none();
        preview.update(cx, |w, _| if show { w.show() } else { w.hide() });
    }

    v_flex()
        .relative()
        .size_full()
        .bg(theme::bg_base())
        .text_color(theme::text_body())
        .font_family(theme::FONT_UI)
        .child(title_bar::render(app, window, cx))
        .child(match app.screen {
            Screen::Login => login::render(app, window, cx).into_any_element(),
            Screen::Home => home::render(app, window, cx).into_any_element(),
            Screen::Onboarding => onboarding::render(app, window, cx).into_any_element(),
            Screen::Workspace => studio_body(app, window, cx).into_any_element(),
            Screen::DsWorkspace => ds_workspace::render(app, window, cx).into_any_element(),
        })
        .when(app.modal.is_some(), |this| this.child(modals::render(app, window, cx)))
        .when(app.toast_note.is_some(), |this| this.child(toast_view(app, cx)))
        .into_any_element()
}

/// The bottom-center toast (`app.toast_note`).
fn toast_view(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let note = app.toast_note.clone().unwrap();
    let (ic, fg, tint, border) = note.tone.style();
    div().absolute().bottom(px(40.0)).left_0().right_0().flex().justify_center().child(
        h_flex()
            .occlude()
            .items_center()
            .gap(px(12.0))
            .max_w(px(440.0))
            .px(px(16.0))
            .py(px(13.0))
            .rounded(px(theme::RADIUS_MD))
            .bg(theme::bg_raised())
            .border_1()
            .border_color(border)
            .shadow(theme::shadow_pop())
            .child(div().size(px(26.0)).flex_none().rounded(px(8.0)).bg(tint).flex().items_center().justify_center().child(icon(ic, 15.0, fg)))
            .child(div().flex_1().text_size(px(13.0)).text_color(theme::text_soft()).line_height(px(19.0)).child(note.msg.clone()))
            .child(
                div()
                    .id("toast-x")
                    .size(px(24.0))
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(6.0))
                    .text_color(theme::icon_color())
                    .cursor_pointer()
                    .hover(|s| s.text_color(theme::text_strong()))
                    .child(icon("close", 14.0, theme::icon_color()))
                    .on_click(cx.listener(|a, _, _, cx| a.dismiss_note(cx))),
            ),
    )
}

/// The workspace: toolbar, a collapsible chat, the canvas, and a collapsible
/// context panel.
fn studio_body(app: &StudioApp, window: &mut Window, cx: &mut Context<StudioApp>) -> AnyElement {
    let panel_hidden = app.right_mode() == RightMode::Outline && !app.panel_open;

    let left = if app.chat_open {
        chat::render(app, window, cx).into_any_element()
    } else {
        reopen_tab("chat-tab", "sparkle", true, cx.listener(|a, _, _, cx| a.toggle_chat(cx))).into_any_element()
    };
    let right = if panel_hidden {
        reopen_tab("panel-tab", "layers", false, cx.listener(|a, _, _, cx| a.toggle_panel(cx))).into_any_element()
    } else {
        sidebar::render(app, window, cx).into_any_element()
    };

    v_flex()
        .relative()
        .flex_1()
        .min_h_0()
        .child(toolbar::render(app, window, cx))
        .child(
            h_flex()
                .flex_1()
                .min_h_0()
                .w_full()
                .child(left)
                .child(div().flex_1().min_w_0().h_full().child(canvas::render(app, window, cx)))
                .child(right),
        )
        .into_any_element()
}

/// A thin vertical tab that reopens a collapsed chat/context panel.
fn reopen_tab(id: &'static str, ic: &'static str, left_edge: bool, on_click: impl Fn(&ClickEvent, &mut Window, &mut gpui::App) + 'static) -> impl IntoElement {
    let mut t = v_flex()
        .id(id)
        .w(px(40.0))
        .flex_none()
        .h_full()
        .items_center()
        .pt(px(14.0))
        .bg(theme::bg_base())
        .cursor_pointer()
        .hover(|s| s.bg(theme::bg_hover()))
        .child(icon(ic, 18.0, if left_edge { theme::accent() } else { theme::icon_color() }))
        .on_click(on_click);
    t = if left_edge { t.border_r_1().border_color(theme::line()) } else { t.border_l_1().border_color(theme::line()) };
    t
}
