use gpui::{Context, Window, div, prelude::*};
use gpui_component::{
    ActiveTheme, StyledExt,
    button::{Button, ButtonVariants},
    v_flex,
};

use crate::app::StudioApp;

/// M0: the preview lives in a separate native window (Wayland has no
/// embedded child-webview path in published crates — see M0_RESULTS).
/// This pane reflects its state and hosts the reopen control.
pub fn preview_area(
    app: &StudioApp,
    _window: &mut Window,
    cx: &mut Context<StudioApp>,
) -> impl IntoElement {
    let preview_open = app.preview.is_some();

    let (headline, detail) = if preview_open {
        (
            "المعاينة مفتوحة في نافذة منفصلة",
            "Your live preview is running in its own window",
        )
    } else {
        (
            "نافذة المعاينة مغلقة",
            "The preview window is closed — reopen it below",
        )
    };

    v_flex()
        .flex_1()
        .items_center()
        .justify_center()
        .bg(cx.theme().muted.opacity(0.3))
        .child(
            v_flex()
                .items_center()
                .gap_3()
                .p_8()
                .rounded_lg()
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().background)
                .child(div().text_lg().font_semibold().child(headline))
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(detail),
                )
                .when(!preview_open, |this| {
                    this.child(
                        Button::new("reopen-preview")
                            .primary()
                            .label("فتح المعاينة · Open preview")
                            .on_click(cx.listener(|app, _, _, cx| {
                                app.open_preview(cx);
                                // Fresh window needs content + navigation.
                                app.recompile(cx);
                            })),
                    )
                }),
        )
}
