//! Center canvas. The generated website renders in an **embedded webview**
//! (built as a child of the gpui window in `crate::app`) sized to the device
//! frame. Because webkit paints above GPUI content, the webview is hidden
//! whenever a native overlay (empty state, busy, error) must draw over the
//! canvas.

use gpui::{Context, Window, div, prelude::*, px};
use gpui_component::{StyledExt, h_flex, v_flex};

use crate::app::StudioApp;
use crate::state::{Device, Status};
use crate::theme;
use crate::ui::widgets::{brand_badge, icon};

pub fn render(app: &StudioApp, _window: &mut Window, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let show_empty = !app.generated && !app.busy && app.status != Status::Error;
    // The webview renders above GPUI *no matter where it sits in the element
    // tree* (it's a real, separate child window, not GPUI-painted content),
    // so it must be hidden for any overlay that needs to cover it — not just
    // the canvas's own busy/error/empty states, but the settings/activity/
    // skills popovers and the toast too, all of which are plain GPUI content
    // rendered elsewhere in the tree.
    let overlay_open = app.show_settings || app.show_activity || app.show_skills || app.toast;
    let show_preview = app.generated && !app.busy && app.status != Status::Error && !overlay_open;

    if let Some(preview) = &app.preview {
        preview.update(cx, |w, _| {
            if show_preview {
                w.show();
            } else {
                w.hide();
            }
        });
    }

    div()
        .relative()
        .size_full()
        .bg(theme::canvas())
        .when(show_preview, |d| d.child(preview_frame(app)))
        .when(show_empty, |d| d.child(empty_state(app, cx)))
        .when(app.busy, |d| d.child(busy_overlay(app)))
        .when(app.status == Status::Error, |d| d.child(error_overlay(app, cx)))
}

/// The device-sized frame that hosts the preview webview.
fn preview_frame(app: &StudioApp) -> impl IntoElement {
    let (frame_w, radius) = if app.review_open {
        (1400.0, 0.0)
    } else {
        match app.device {
            Device::Desktop => (1120.0, 10.0),
            Device::Tablet => (768.0, 10.0),
            Device::Mobile => (390.0, 24.0),
        }
    };
    let pad = if app.review_open {
        0.0
    } else if app.device == Device::Desktop {
        26.0
    } else {
        30.0
    };

    div()
        .absolute()
        .inset_0()
        .p(px(pad))
        .flex()
        .justify_center()
        .child(
            div()
                .w_full()
                .max_w(px(frame_w))
                .h_full()
                .rounded(px(radius))
                .overflow_hidden()
                .bg(theme::white(1.0))
                .children(app.preview.as_ref().cloned()),
        )
}

fn empty_state(_app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    div()
        .absolute()
        .inset_0()
        .items_center()
        .justify_center()
        .flex()
        .child(
            v_flex()
                .items_center()
                .max_w(px(420.0))
                .child(brand_badge(60.0, 30.0))
                .child(
                    div()
                        .mt(px(20.0))
                        .font_family(theme::FONT_DISPLAY)
                        .font_semibold()
                        .text_size(px(21.0))
                        .text_center()
                        .child("Describe your website to begin"),
                )
                .child(
                    div()
                        .mt(px(8.0))
                        .text_size(px(14.5))
                        .text_color(theme::muted())
                        .text_center()
                        .line_height(px(23.0))
                        .child("Use the chat on the left \u{2014} in Arabic or English. WebFluent builds a live, editable site. No code, ever."),
                )
                .child(
                    h_flex()
                        .mt(px(22.0))
                        .gap(px(9.0))
                        .flex_wrap()
                        .justify_center()
                        .children(crate::state::STARTERS.iter().enumerate().map(|(i, s)| {
                            let prompt = s.prompt;
                            div()
                                .id(("starter", i))
                                .px(px(15.0))
                                .py(px(8.0))
                                .rounded_full()
                                .border_1()
                                .border_color(theme::white(0.14))
                                .bg(theme::panel())
                                .text_size(px(13.0))
                                .font_medium()
                                .text_color(theme::text_soft())
                                .cursor_pointer()
                                .hover(|st| st.border_color(theme::accent()))
                                .child(s.chip)
                                .on_click(cx.listener(move |a, _, window, cx| a.start_with(prompt, window, cx)))
                        })),
                ),
        )
}

fn busy_overlay(app: &StudioApp) -> impl IntoElement {
    let color = app.status.label_color(app.generated).1;
    div()
        .absolute()
        .inset_0()
        .bg(theme::hexa(0x14110f8c))
        .flex()
        .items_center()
        .justify_center()
        .child(
            h_flex()
                .items_center()
                .gap(px(13.0))
                .px(px(22.0))
                .py(px(14.0))
                .bg(theme::panel())
                .border_1()
                .border_color(theme::border_strong())
                .rounded(px(13.0))
                .shadow_lg()
                .child(icon("refresh", 20.0, color))
                .child(div().text_size(px(14.0)).font_medium().child(app.sub_label.clone())),
        )
}

fn error_overlay(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let title = format!("Couldn\u{2019}t reach {}", app.provider().name);
    div()
        .absolute()
        .inset_0()
        .bg(theme::hexa(0x14110fd1))
        .flex()
        .items_center()
        .justify_center()
        .child(
            v_flex()
                .items_center()
                .max_w(px(430.0))
                .px(px(24.0))
                .child(
                    div()
                        .size(px(60.0))
                        .rounded(px(16.0))
                        .bg(theme::hexa(0xec6a5e29))
                        .border_1()
                        .border_color(theme::hexa(0xec6a5e66))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(icon("wifi-off", 28.0, theme::danger())),
                )
                .child(
                    div()
                        .mt(px(20.0))
                        .font_family(theme::FONT_DISPLAY)
                        .font_semibold()
                        .text_size(px(21.0))
                        .text_center()
                        .child(title),
                )
                .child(
                    div()
                        .mt(px(9.0))
                        .text_size(px(14.5))
                        .text_color(theme::hex(0xa89e96))
                        .text_center()
                        .line_height(px(23.0))
                        .child("The request timed out with no response. Check your internet connection or your API key, then try again \u{2014} your project is safe."),
                )
                .child(
                    h_flex()
                        .mt(px(24.0))
                        .gap(px(11.0))
                        .child(
                            h_flex()
                                .id("err-retry")
                                .items_center()
                                .gap(px(8.0))
                                .px(px(22.0))
                                .py(px(11.0))
                                .rounded(px(11.0))
                                .bg(theme::accent())
                                .text_color(theme::white(1.0))
                                .font_semibold()
                                .text_size(px(14.0))
                                .cursor_pointer()
                                .child(icon("refresh", 15.0, theme::white(1.0)))
                                .child("Try again")
                                .on_click(cx.listener(|a, _, window, cx| a.retry(window, cx))),
                        )
                        .child(
                            div()
                                .id("err-settings")
                                .px(px(22.0))
                                .py(px(11.0))
                                .rounded(px(11.0))
                                .border_1()
                                .border_color(theme::white(0.14))
                                .text_color(theme::text_dim())
                                .font_semibold()
                                .text_size(px(14.0))
                                .cursor_pointer()
                                .child("Check settings")
                                .on_click(cx.listener(|a, _, _, cx| a.open_settings(cx))),
                        ),
                ),
        )
}
