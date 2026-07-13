use gpui::{App, Context, Hsla, SharedString, Window, div, prelude::*};
use gpui_component::{
    ActiveTheme, StyledExt, TitleBar,
    button::{Button, ButtonVariants},
    h_flex,
};
use wf_core::CompileStatus;

use crate::app::StudioApp;

pub fn top_bar(
    app: &StudioApp,
    _window: &mut Window,
    cx: &mut Context<StudioApp>,
) -> impl IntoElement {
    TitleBar::new().child(
        h_flex()
            .flex_1()
            .items_center()
            .justify_between()
            .pr_2()
            .child(
                h_flex()
                    .items_center()
                    .gap_2()
                    .child(div().font_semibold().child("WebFluent Studio"))
                    .child(
                        // Spike B probe: Arabic must shape/join correctly in chrome.
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("استوديو ويب فلونت"),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .gap_2()
                    .child(status_pill(&app.status, cx))
                    .child(
                        Button::new("settings")
                            .ghost()
                            .label("الإعدادات · Settings")
                            .on_click(|_, _, _| {
                                // Placeholder: provider settings dropdown lands in M1.
                            }),
                    ),
            ),
    )
}

/// The "Compiled" badge (FR-13). Shows a status dot + human label —
/// diagnostics stay structured, never rendered as code (FR-6).
fn status_pill(status: &CompileStatus, cx: &App) -> impl IntoElement {
    let (color, label): (Hsla, SharedString) = match status {
        CompileStatus::Idle => (cx.theme().muted_foreground, "جاهز · Ready".into()),
        CompileStatus::Compiling => (cx.theme().info, "يُجمَّع… · Compiling…".into()),
        CompileStatus::Compiled { duration_ms } => (
            cx.theme().success,
            format!("مُجمَّع · Compiled in {duration_ms}ms").into(),
        ),
        CompileStatus::Failed { diagnostics } => (
            cx.theme().danger,
            format!("يحتاج انتباه · {} issue(s)", diagnostics.len()).into(),
        ),
    };

    h_flex()
        .items_center()
        .gap_2()
        .px_3()
        .py_1()
        .rounded_full()
        .border_1()
        .border_color(color.opacity(0.4))
        .child(div().w_2().h_2().rounded_full().bg(color))
        .child(div().text_xs().child(label))
}
