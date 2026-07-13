//! Left chat panel: assistant header, the message transcript, and the prompt
//! dock (attachments, skills, try-row, textarea, send).

use gpui::{Context, Hsla, Window, div, prelude::*, px};
use gpui_component::{StyledExt, h_flex, input::Input, v_flex};

use crate::app::StudioApp;
use crate::state::{Message, Role, Tone};
use crate::theme;
use crate::ui::widgets::{brand_badge, dot, icon};

pub fn render(app: &StudioApp, _window: &mut Window, cx: &mut Context<StudioApp>) -> impl IntoElement {
    v_flex()
        .h_full()
        .flex_none()
        .w(px(358.0))
        .bg(theme::panel())
        .border_r_1()
        .border_color(theme::hairline())
        .child(header(app))
        .child(messages(app))
        .child(dock(app, cx))
}

fn header(app: &StudioApp) -> impl IntoElement {
    h_flex()
        .flex_none()
        .px(px(16.0))
        .pt(px(14.0))
        .pb(px(13.0))
        .items_center()
        .gap(px(10.0))
        .border_b_1()
        .border_color(theme::hairline())
        .child(brand_badge(27.0, 15.0))
        .child(
            div()
                .flex_1()
                .font_family(theme::FONT_DISPLAY)
                .text_size(px(14.5))
                .font_semibold()
                .child("Assistant"),
        )
        .child(
            h_flex()
                .items_center()
                .gap(px(6.0))
                .child(dot(6.0, theme::success()))
                .child(div().text_size(px(11.5)).text_color(theme::muted()).child(app.provider().name)),
        )
}

fn messages(app: &StudioApp) -> impl IntoElement {
    let empty = app.messages.is_empty();
    let mut list = v_flex()
        .id("messages")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .px(px(14.0))
        .py(px(16.0))
        .gap(px(11.0));

    if empty {
        list = list.justify_center().child(
            div()
                .mx_auto()
                .max_w(px(280.0))
                .text_center()
                .text_size(px(13.0))
                .text_color(theme::faint())
                .line_height(px(21.0))
                .child("Tell me what to build or change \u{2014} in Arabic or English. Attach a photo or logo with the clip, and I\u{2019}ll fold it into the design. You review every change before it sticks."),
        );
    } else {
        list = list.children(app.messages.iter().map(message_row));
    }

    if app.busy {
        list = list.child(
            h_flex()
                .items_center()
                .gap(px(9.0))
                .p(px(2.0))
                .text_color(theme::hex(0xa89e96))
                .text_size(px(13.0))
                .child(icon("refresh", 15.0, app.status.label_color(app.generated).1))
                .child(div().child(app.sub_label.clone())),
        );
    }

    list
}

fn message_row(m: &Message) -> impl IntoElement {
    let (bg, fg, border) = bubble_tones(m);
    let mut bubble = v_flex()
        .max_w(px(272.0))
        .min_w_0()
        .px(px(12.0))
        .py(px(9.0))
        .rounded(px(13.0))
        .bg(bg)
        .text_color(fg)
        .text_size(px(13.0))
        .line_height(px(19.0));
    if let Some(border) = border {
        bubble = bubble.border_1().border_color(border);
    }
    if !m.attachments.is_empty() {
        bubble = bubble.child(
            h_flex().flex_wrap().gap(px(6.0)).mb(px(8.0)).children(m.attachments.iter().map(|name| {
                h_flex()
                    .items_center()
                    .gap(px(5.0))
                    .px(px(8.0))
                    .py(px(3.0))
                    .rounded(px(6.0))
                    .bg(theme::black(0.22))
                    .text_size(px(11.0))
                    .child(icon("paperclip", 11.0, fg))
                    .child(name.clone())
            })),
        );
    }
    bubble = bubble.child(div().w_full().min_w_0().child(m.text.clone()));

    h_flex()
        .w_full()
        .when(m.role == Role::User, |r| r.justify_end())
        .child(bubble)
}

fn bubble_tones(m: &Message) -> (Hsla, Hsla, Option<Hsla>) {
    if m.role == Role::User {
        return (theme::accent(), theme::white(1.0), None);
    }
    match m.tone {
        Tone::Warn => (theme::hexa(0xe5a54b21), theme::warn_soft(), Some(theme::hexa(0xe5a54b4d))),
        Tone::Err => (theme::hexa(0xec6a5e21), theme::hex(0xec8f83), Some(theme::hexa(0xec6a5e4d))),
        _ => (theme::bubble(), theme::text_soft(), None),
    }
}

fn dock(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let has_sel = app.generated && !app.sel.is_empty() && !app.busy && !app.review_open;
    let has_skills = !app.active_skills.is_empty();
    let has_atts = !app.attachments.is_empty();

    let dock_border = if !app.dock_hint.is_empty() {
        theme::hexa(0xec6a5e99)
    } else if app.busy {
        theme::hexa(0xe2725b66)
    } else {
        theme::white(0.1)
    };

    let send_bg = if app.busy { theme::hex(0x4a423d) } else { theme::accent() };

    v_flex()
        .flex_none()
        .px(px(14.0))
        .pt(px(12.0))
        .pb(px(14.0))
        .border_t_1()
        .border_color(theme::hairline())
        .when(has_sel, |this| this.child(try_row(app, cx)))
        .when(has_skills, |this| this.child(skill_chips(app, cx)))
        .when(has_atts, |this| this.child(attachment_chips(app, cx)))
        .child(
            v_flex()
                .gap(px(9.0))
                .bg(theme::sunken())
                .border_1()
                .border_color(dock_border)
                .rounded(px(13.0))
                .px(px(12.0))
                .py(px(11.0))
                .child(Input::new(&app.prompt).appearance(false).text_size(px(14.0)))
                .child(
                    h_flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(small_btn("attach", "paperclip", theme::panel(), theme::text_dim(), cx.listener(|a, _, _, cx| a.trigger_file(cx))))
                        .child(skills_btn(app, cx))
                        .child(div().flex_1())
                        .child(div().text_size(px(11.0)).text_color(theme::faint()).child(app.model.clone()))
                        .child(
                            div()
                                .id("send")
                                .size(px(32.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .rounded(px(8.0))
                                .bg(send_bg)
                                .cursor_pointer()
                                .child(icon(if app.busy { "stop" } else { "send" }, if app.busy { 13.0 } else { 16.0 }, theme::white(1.0)))
                                .on_click(cx.listener(|a, _, window, cx| a.send_or_cancel(window, cx))),
                        ),
                ),
        )
        .when(!app.dock_hint.is_empty(), |this| {
            this.child(
                h_flex()
                    .mt(px(9.0))
                    .items_center()
                    .gap(px(7.0))
                    .text_size(px(12.0))
                    .text_color(theme::accent_hover())
                    .child(icon("alert-circle", 12.0, theme::accent_hover()))
                    .child(div().child(app.dock_hint.clone())),
            )
        })
}

fn try_row(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    // Contextual "try" quick-prompts only make sense for a single, unambiguous
    // section, so only show them when exactly one is selected.
    let tries: Vec<&'static str> = match app.sel.as_slice() {
        [only] => crate::state::section(only.key).tries.to_vec(),
        _ => Vec::new(),
    };

    h_flex()
        .flex_wrap()
        .items_center()
        .gap(px(7.0))
        .mb(px(9.0))
        .children(app.sel.iter().enumerate().map(|(i, s)| {
            let key = s.key;
            let label = crate::state::section(key).label;
            h_flex()
                .id(("sel-chip", i))
                .items_center()
                .gap(px(6.0))
                .px(px(10.0))
                .py(px(4.0))
                .rounded_full()
                .bg(theme::hexa(0xe2725b24))
                .text_size(px(11.5))
                .font_semibold()
                .text_color(theme::accent_soft())
                .child(icon("plus", 11.0, theme::accent_soft()))
                .child(div().child(label))
                .child(
                    div()
                        .id(("clear-sel", i))
                        .cursor_pointer()
                        .opacity(0.7)
                        .ml(px(2.0))
                        .child("\u{2715}")
                        .on_click(cx.listener(move |a, _, _, cx| a.remove_selection(key, cx))),
                )
        }))
        .children(tries.into_iter().enumerate().map(|(i, t)| {
            div()
                .id(("try", i))
                .px(px(10.0))
                .py(px(4.0))
                .rounded_full()
                .border_1()
                .border_color(theme::white(0.12))
                .bg(theme::elevated())
                .text_size(px(11.5))
                .text_color(theme::text_dim())
                .cursor_pointer()
                .hover(|s| s.border_color(theme::white(0.2)))
                .child(t)
                .on_click(cx.listener(move |a, _, window, cx| a.pick_try(t, window, cx)))
        }))
}

fn skill_chips(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    h_flex()
        .flex_wrap()
        .gap(px(6.0))
        .mb(px(9.0))
        .children(app.active_skills.iter().map(|id| {
            let id = *id;
            let sk = crate::state::skill(id);
            h_flex()
                .items_center()
                .gap(px(6.0))
                .px(px(9.0))
                .py(px(5.0))
                .rounded(px(8.0))
                .bg(theme::hexa(0xd9a06621))
                .border_1()
                .border_color(theme::hexa(0xd9a06659))
                .text_size(px(11.5))
                .text_color(theme::gold_soft())
                .child(icon("skill", 11.0, theme::gold_soft()))
                .child(div().child(sk.name))
                .child(
                    div()
                        .id(("rm-skill", id as usize))
                        .cursor_pointer()
                        .opacity(0.6)
                        .child("\u{2715}")
                        .on_click(cx.listener(move |a, _, _, cx| a.remove_skill(id, cx))),
                )
        }))
}

fn attachment_chips(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    h_flex()
        .flex_wrap()
        .gap(px(6.0))
        .mb(px(9.0))
        .children(app.attachments.iter().map(|at| {
            let id = at.id;
            h_flex()
                .items_center()
                .gap(px(6.0))
                .px(px(9.0))
                .py(px(5.0))
                .rounded(px(8.0))
                .bg(theme::elevated())
                .border_1()
                .border_color(theme::white(0.1))
                .text_size(px(11.5))
                .text_color(theme::text_soft())
                .child(icon("paperclip", 11.0, theme::accent()))
                .child(div().child(at.name.clone()))
                .child(
                    div()
                        .id(("rm-att", id as usize))
                        .cursor_pointer()
                        .opacity(0.6)
                        .child("\u{2715}")
                        .on_click(cx.listener(move |a, _, _, cx| a.remove_attachment(id, cx))),
                )
        }))
}

fn skills_btn(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let active = !app.active_skills.is_empty();
    let bg = if app.show_skills { theme::seg_active() } else { theme::panel() };
    let fg = if active { theme::gold_soft() } else { theme::text_dim() };
    let border = if active { theme::hexa(0xd9a06666) } else { theme::white(0.12) };
    div()
        .id("skills")
        .size(px(30.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(8.0))
        .border_1()
        .border_color(border)
        .bg(bg)
        .cursor_pointer()
        .child(icon("skill", 15.0, fg))
        .on_click(cx.listener(|a, _, _, cx| a.toggle_skills_menu(cx)))
}

fn small_btn(
    id: &'static str,
    icon_name: &'static str,
    bg: Hsla,
    fg: Hsla,
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
        .border_color(theme::white(0.12))
        .bg(bg)
        .cursor_pointer()
        .child(icon(icon_name, 15.0, fg))
        .on_click(on_click)
}
