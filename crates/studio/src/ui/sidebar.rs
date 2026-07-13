//! Right sidebar: the Visual Diff Review panel (FR-7) or the Version History
//! panel (FR-14), whichever is open.

use gpui::{Context, Window, div, prelude::*, px};
use gpui_component::{StyledExt, h_flex, v_flex};

use crate::app::StudioApp;
use crate::state::{Checkpoint, Chip};
use crate::theme;
use crate::ui::widgets::icon;

pub fn render(app: &StudioApp, _window: &mut Window, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let mut panel = v_flex()
        .flex_none()
        .h_full()
        .w(px(366.0))
        .bg(theme::panel())
        .border_l_1()
        .border_color(theme::hairline());

    if app.review_open {
        panel = panel.child(review(app, cx));
    } else if app.show_history {
        panel = panel.child(history(app, cx));
    }
    panel
}

fn review(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let count = app.chips.len();
    let accepted = app.accepted_count();
    let apply_enabled = accepted > 0;
    let plural = if accepted == 1 { "" } else { "s" };

    v_flex()
        .flex_1()
        .min_h_0()
        .child(
            v_flex()
                .flex_none()
                .px(px(18.0))
                .pt(px(18.0))
                .pb(px(14.0))
                .border_b_1()
                .border_color(theme::hairline())
                .child(
                    h_flex()
                        .items_center()
                        .justify_between()
                        .child(div().font_family(theme::FONT_DISPLAY).font_semibold().text_size(px(16.0)).child("Review changes"))
                        .child(div().text_size(px(12.0)).text_color(theme::muted()).child(format!("{count} proposed"))),
                )
                .child(
                    div()
                        .mt(px(6.0))
                        .text_size(px(13.0))
                        .text_color(theme::muted())
                        .line_height(px(19.0))
                        .child("Drag the slider on the preview to compare. Keep or drop each change."),
                ),
        )
        .child(
            h_flex()
                .flex_none()
                .items_center()
                .gap(px(10.0))
                .px(px(18.0))
                .py(px(10.0))
                .border_b_1()
                .border_color(theme::hairline())
                .child(div().text_size(px(12.0)).text_color(theme::muted()).child(format!("{accepted} of {count} kept")))
                .child(div().flex_1())
                .child(mini_btn("keep-all", "Keep all", cx.listener(|a, _, _, cx| a.keep_all(cx))))
                .child(mini_btn("clear-all", "Clear all", cx.listener(|a, _, _, cx| a.clear_all(cx)))),
        )
        .when(count > 8, |this| {
            this.child(
                div()
                    .flex_none()
                    .mx(px(12.0))
                    .mt(px(12.0))
                    .px(px(12.0))
                    .py(px(9.0))
                    .rounded(px(9.0))
                    .bg(theme::hexa(0xe5a54b1a))
                    .border_1()
                    .border_color(theme::hexa(0xe5a54b47))
                    .text_size(px(12.0))
                    .text_color(theme::warn_soft())
                    .line_height(px(17.0))
                    .child("This is a large edit \u{2014} the changes are grouped by section. Review each carefully before applying."),
            )
        })
        .child(
            v_flex()
                .id("chip-list")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .p(px(12.0))
                .gap(px(7.0))
                .children(app.chips.iter().map(|c| chip_row(c, cx))),
        )
        .child(
            h_flex()
                .flex_none()
                .gap(px(9.0))
                .p(px(14.0))
                .border_t_1()
                .border_color(theme::hairline())
                .child(
                    div()
                        .id("discard")
                        .flex_none()
                        .px(px(15.0))
                        .py(px(11.0))
                        .rounded(px(10.0))
                        .border_1()
                        .border_color(theme::white(0.14))
                        .text_color(theme::text_dim())
                        .font_semibold()
                        .text_size(px(13.5))
                        .cursor_pointer()
                        .child("Discard")
                        .on_click(cx.listener(|a, _, _, cx| a.reset_proposal(cx))),
                )
                .child(
                    div()
                        .id("apply")
                        .flex_1()
                        .flex()
                        .justify_center()
                        .px(px(15.0))
                        .py(px(11.0))
                        .rounded(px(10.0))
                        .bg(if apply_enabled { theme::accent() } else { theme::hex(0x3a332f) })
                        .text_color(theme::white(1.0))
                        .font_semibold()
                        .text_size(px(13.5))
                        .when(apply_enabled, |d| d.cursor_pointer())
                        .child(format!("Apply {accepted} change{plural}"))
                        .on_click(cx.listener(|a, _, _, cx| a.apply_accepted(cx))),
                ),
        )
}

fn chip_row(c: &Chip, cx: &mut Context<StudioApp>) -> impl IntoElement + use<> {
    let id = c.id;
    let (kind_fg, kind_bg) = theme::chip_kind(c.kind);
    let border = if c.accepted { theme::hexa(0xe2725b59) } else { theme::white(0.07) };
    let bg = if c.accepted { theme::hexa(0xe2725b14) } else { theme::elevated() };

    h_flex()
        .id(("chip", id as usize))
        .items_start()
        .gap(px(12.0))
        .p(px(12.0))
        .rounded(px(11.0))
        .border_1()
        .border_color(border)
        .bg(bg)
        .cursor_pointer()
        .on_click(cx.listener(move |a, _, _, cx| a.toggle_chip(id, cx)))
        .child(
            div()
                .flex_none()
                .size(px(20.0))
                .mt(px(1.0))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(6.0))
                .border_1()
                .border_color(if c.accepted { theme::accent() } else { theme::white(0.2) })
                .bg(if c.accepted { theme::accent() } else { gpui::transparent_black() })
                .when(c.accepted, |d| d.child(icon("check", 13.0, theme::white(1.0)))),
        )
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .items_start()
                .gap(px(5.0))
                .child(div().text_size(px(13.5)).font_medium().text_color(theme::text()).line_height(px(19.0)).child(c.label.clone()))
                .child(
                    div()
                        .px(px(7.0))
                        .py(px(2.0))
                        .rounded(px(5.0))
                        .bg(kind_bg)
                        .text_color(kind_fg)
                        .text_size(px(10.5))
                        .font_bold()
                        .child(c.kind.label().to_uppercase()),
                ),
        )
}

fn mini_btn(
    id: &'static str,
    label: &'static str,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .px(px(11.0))
        .py(px(5.0))
        .rounded(px(7.0))
        .border_1()
        .border_color(theme::white(0.12))
        .bg(theme::elevated())
        .text_color(theme::text_dim())
        .text_size(px(12.0))
        .font_semibold()
        .cursor_pointer()
        .hover(|s| s.border_color(theme::white(0.2)))
        .child(label)
        .on_click(on_click)
}

fn history(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    v_flex()
        .flex_1()
        .min_h_0()
        .child(
            h_flex()
                .flex_none()
                .items_center()
                .justify_between()
                .px(px(18.0))
                .pt(px(18.0))
                .pb(px(14.0))
                .border_b_1()
                .border_color(theme::hairline())
                .child(div().font_family(theme::FONT_DISPLAY).font_semibold().text_size(px(16.0)).child("Version history"))
                .child(
                    div()
                        .id("close-history")
                        .size(px(26.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(7.0))
                        .text_color(theme::muted())
                        .cursor_pointer()
                        .hover(|s| s.bg(theme::white(0.05)))
                        .child(icon("close", 15.0, theme::muted()))
                        .on_click(cx.listener(|a, _, _, cx| a.toggle_history(cx))),
                ),
        )
        .child(
            v_flex()
                .id("history-list")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .px(px(12.0))
                .pt(px(8.0))
                .pb(px(16.0))
                .when(app.history.is_empty(), |this| {
                    this.child(
                        div()
                            .py(px(40.0))
                            .px(px(16.0))
                            .text_center()
                            .text_size(px(13.0))
                            .text_color(theme::faint())
                            .line_height(px(21.0))
                            .child("No changes yet. Every edit you keep is saved here automatically."),
                    )
                })
                .children(app.history.iter().enumerate().map(|(i, h)| history_row(i, h, cx))),
        )
}

fn history_row(i: usize, h: &Checkpoint, cx: &mut Context<StudioApp>) -> impl IntoElement + use<> {
    let dot_color = if h.current { theme::success() } else { theme::muted() };
    h_flex()
        .gap(px(12.0))
        .px(px(2.0))
        .py(px(4.0))
        .items_start()
        .child(
            v_flex()
                .flex_none()
                .items_center()
                .child(div().size(px(11.0)).rounded_full().bg(dot_color).border_2().border_color(theme::panel()))
                .child(div().flex_1().w(px(2.0)).min_h(px(18.0)).my(px(3.0)).bg(theme::white(0.09))),
        )
        .child(
            v_flex()
                .flex_1()
                .pb(px(14.0))
                .gap(px(2.0))
                .items_start()
                .child(div().text_size(px(13.5)).font_medium().text_color(theme::text()).child(h.title.clone()))
                .child(div().text_size(px(11.5)).text_color(theme::faint()).child(h.time.clone()))
                .when(!h.current, |this| {
                    this.child(
                        div()
                            .id(("restore", i))
                            .mt(px(7.0))
                            .px(px(11.0))
                            .py(px(5.0))
                            .rounded(px(7.0))
                            .border_1()
                            .border_color(theme::white(0.14))
                            .bg(theme::raised())
                            .text_color(theme::accent())
                            .text_size(px(11.5))
                            .font_semibold()
                            .cursor_pointer()
                            .child("Restore this version")
                            .on_click(cx.listener(move |a, _, _, cx| a.restore(i, cx))),
                    )
                })
                .when(h.current, |this| {
                    this.child(
                        div()
                            .mt(px(7.0))
                            .px(px(9.0))
                            .py(px(3.0))
                            .rounded(px(6.0))
                            .bg(theme::hexa(0x63c08826))
                            .text_color(theme::success())
                            .text_size(px(11.0))
                            .font_semibold()
                            .child("Current version"),
                    )
                }),
        )
}
