//! Layered popovers over the studio: the Settings dropdown (provider/key/model),
//! the Skills popover, the Activity log (FR-21), and the needs-attention toast
//! (FR-22). Each dims-and-catches outside clicks with a full-bleed backdrop
//! sibling painted *under* the panel.

use gpui::{App, ClickEvent, Context, Window, div, prelude::*, px};
use gpui_component::{StyledExt, h_flex, input::Input, v_flex};

use crate::app::StudioApp;
use crate::state::{PROVIDERS, ProviderId, SKILLS, Tested};
use crate::theme;
use crate::ui::widgets::icon;

fn label(text: &'static str) -> impl IntoElement {
    div()
        .text_size(px(11.0))
        .font_bold()
        .text_color(theme::faint())
        .child(text.to_uppercase())
}

pub fn settings(app: &StudioApp, _window: &mut Window, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let key_ok = !app.key_text(cx).trim().is_empty();
    let (test_label, test_bg, test_fg) = match app.tested {
        Tested::Ok => ("\u{2713} Connected", theme::hexa(0x63c0882e), theme::success()),
        Tested::Fail => ("Retry", theme::hexa(0xec6a5e2e), theme::danger()),
        _ if key_ok => ("Test", theme::accent(), theme::white(1.0)),
        _ => ("Test", theme::seg_active(), theme::faint()),
    };
    let key_border = match app.tested {
        Tested::Fail => theme::danger(),
        Tested::Ok => theme::success(),
        _ if key_ok => theme::hexa(0xe2725b80),
        _ => theme::white(0.1),
    };

    let panel = div()
        .absolute()
        .top(px(4.0))
        .right(px(12.0))
        .w(px(362.0))
        .bg(theme::panel())
        .border_1()
        .border_color(theme::border_strong())
        .rounded(px(14.0))
        .shadow_lg()
        .overflow_hidden()
        .child(div().px(px(16.0)).py(px(14.0)).border_b_1().border_color(theme::hairline()).font_family(theme::FONT_DISPLAY).font_semibold().text_size(px(14.5)).child("Settings"))
        .child(
            v_flex()
                .id("settings-body")
                .px(px(16.0))
                .py(px(14.0))
                .max_h(px(560.0))
                .overflow_y_scroll()
                .child(label("AI Provider"))
                .child(h_flex().mt(px(9.0)).flex_wrap().gap(px(6.0)).children(PROVIDERS.iter().map(|p| provider_mini(app, p.id, cx))))
                .child(div().mt(px(16.0)).child(label("Model")))
                .child(v_flex().mt(px(8.0)).gap(px(6.0)).children(app.provider().models.iter().map(|m| model_row(app, m, cx))))
                .child(div().mt(px(16.0)).child(label("API Key")))
                .child(
                    h_flex()
                        .mt(px(8.0))
                        .items_center()
                        .gap(px(8.0))
                        .px(px(11.0))
                        .py(px(8.0))
                        .rounded(px(10.0))
                        .bg(theme::sunken())
                        .border_1()
                        .border_color(key_border)
                        .child(icon("lock", 15.0, theme::muted()))
                        .child(div().flex_1().min_w_0().child(Input::new(&app.api_key).appearance(false).text_size(px(13.0))))
                        .child(
                            div()
                                .id("test-conn")
                                .px(px(14.0))
                                .py(px(7.0))
                                .rounded(px(8.0))
                                .bg(test_bg)
                                .text_color(test_fg)
                                .text_size(px(12.5))
                                .font_semibold()
                                .cursor_pointer()
                                .child(test_label)
                                .on_click(cx.listener(|a, _, _, cx| a.test_conn(cx))),
                        ),
                )
                .child(
                    h_flex()
                        .mt(px(7.0))
                        .items_center()
                        .gap(px(6.0))
                        .text_size(px(11.5))
                        .text_color(theme::hex(0x7d8f79))
                        .child(div().size(px(6.0)).rounded_full().bg(theme::success()))
                        .child("Stored in your OS keychain \u{2014} never sent to us"),
                )
                .child(
                    h_flex()
                        .id("adv-toggle")
                        .mt(px(16.0))
                        .items_center()
                        .gap(px(8.0))
                        .py(px(9.0))
                        .text_color(theme::text_dim())
                        .text_size(px(13.0))
                        .font_semibold()
                        .cursor_pointer()
                        .child(icon("chevron-right", 14.0, theme::text_dim()))
                        .child("Advanced")
                        .on_click(cx.listener(|a, _, _, cx| a.toggle_advanced(cx))),
                )
                .when(app.show_advanced, |this| this.child(advanced(app, cx))),
        );
    let close = cx.listener(|a, _, _, cx| a.close_menus(cx));
    layer("settings-back", close, panel)
}

fn provider_mini(app: &StudioApp, id: ProviderId, cx: &mut Context<StudioApp>) -> impl IntoElement + use<> {
    let p = crate::state::provider(id);
    let active = app.provider == id;
    v_flex()
        .id(("prov", id_index(id)))
        .w(px(104.0))
        .items_center()
        .gap(px(5.0))
        .px(px(6.0))
        .py(px(10.0))
        .rounded(px(10.0))
        .border_1()
        .border_color(if active { theme::accent() } else { theme::white(0.1) })
        .bg(if active { theme::hexa(0xe2725b1f) } else { theme::elevated() })
        .cursor_pointer()
        .child(
            div()
                .size(px(24.0))
                .rounded(px(7.0))
                .bg(theme::hex(p.mono_bg))
                .flex()
                .items_center()
                .justify_center()
                .text_color(theme::white(1.0))
                .font_bold()
                .text_size(px(11.0))
                .child(p.mono),
        )
        .child(div().text_size(px(12.0)).font_semibold().text_color(theme::text()).child(p.name))
        .on_click(cx.listener(move |a, _, _, cx| a.pick_provider(id, cx)))
}

fn model_row(app: &StudioApp, m: &'static str, cx: &mut Context<StudioApp>) -> impl IntoElement + use<> {
    let active = app.model.as_ref() == m;
    h_flex()
        .id(("model", m.len()))
        .items_center()
        .justify_between()
        .w_full()
        .px(px(12.0))
        .py(px(10.0))
        .rounded(px(9.0))
        .border_1()
        .border_color(if active { theme::hexa(0xe2725b66) } else { theme::white(0.08) })
        .bg(if active { theme::hexa(0xe2725b1a) } else { theme::elevated() })
        .text_color(theme::text())
        .text_size(px(13.0))
        .font_medium()
        .cursor_pointer()
        .child(div().child(m))
        .when(active, |d| d.child(icon("check", 15.0, theme::accent())))
        .on_click(cx.listener(move |a, _, _, cx| a.set_model(m, cx)))
}

fn advanced(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement + use<> {
    v_flex()
        .child(setting_toggle("Context pruning", "Trims unrelated code \u{b7} saves ~60% tokens", app.pruning, cx.listener(|a, _, _, cx| a.set_pruning(cx))))
        .child(setting_toggle("Prompt caching", "Reuses the language spec across calls", app.caching, cx.listener(|a, _, _, cx| a.set_caching(cx))))
        .child(
            v_flex()
                .py(px(10.0))
                .border_t_1()
                .border_color(theme::white(0.05))
                .child(div().text_size(px(13.0)).text_color(theme::text_soft()).child("Self-healing attempts"))
                .child(div().mt(px(2.0)).mb(px(8.0)).text_size(px(11.0)).text_color(theme::faint()).child("How many times to auto-fix an error before asking you"))
                .child(
                    h_flex().gap(px(6.0)).children([1u8, 2, 3, 5].into_iter().map(|n| {
                        let active = app.heal_attempts == n;
                        div()
                            .id(("heal", n as usize))
                            .flex_1()
                            .flex()
                            .justify_center()
                            .py(px(7.0))
                            .rounded(px(8.0))
                            .border_1()
                            .border_color(if active { theme::hexa(0xe2725b80) } else { theme::white(0.1) })
                            .bg(if active { theme::hexa(0xe2725b24) } else { theme::elevated() })
                            .text_color(if active { theme::accent_soft() } else { theme::text_dim() })
                            .text_size(px(13.0))
                            .font_semibold()
                            .cursor_pointer()
                            .child(n.to_string())
                            .on_click(cx.listener(move |a, _, _, cx| a.set_heal_attempts(n, cx)))
                    })),
                ),
        )
}

fn setting_toggle(
    title: &'static str,
    desc: &'static str,
    on: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    h_flex()
        .items_center()
        .justify_between()
        .gap(px(12.0))
        .py(px(10.0))
        .border_t_1()
        .border_color(theme::white(0.05))
        .child(
            v_flex()
                .flex_1()
                .child(div().text_size(px(13.0)).text_color(theme::text_soft()).child(title))
                .child(div().mt(px(2.0)).text_size(px(11.0)).text_color(theme::faint()).child(desc)),
        )
        .child(
            h_flex()
                .id(title)
                .w(px(38.0))
                .h(px(22.0))
                .p(px(2.0))
                .rounded_full()
                .bg(if on { theme::accent() } else { theme::hex(0x3a332f) })
                .items_center()
                .when(on, |d| d.justify_end())
                .cursor_pointer()
                .child(div().size(px(18.0)).rounded_full().bg(theme::white(1.0)))
                .on_click(on_click),
        )
}

pub fn skills(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let panel = v_flex()
        .absolute()
        .bottom(px(96.0))
        .left(px(14.0))
        .w(px(330.0))
        .bg(theme::panel())
        .border_1()
        .border_color(theme::border_strong())
        .rounded(px(14.0))
        .shadow_lg()
        .overflow_hidden()
        .child(
            v_flex()
                .px(px(16.0))
                .py(px(13.0))
                .border_b_1()
                .border_color(theme::hairline())
                .child(div().font_family(theme::FONT_DISPLAY).font_semibold().text_size(px(14.5)).child("Skills"))
                .child(div().mt(px(3.0)).text_size(px(12.0)).text_color(theme::muted()).child("Capabilities the assistant applies to your build")),
        )
        .child(
            v_flex()
                .id("skills-body")
                .px(px(12.0))
                .py(px(10.0))
                .max_h(px(316.0))
                .overflow_y_scroll()
                .gap(px(6.0))
                .children(SKILLS.iter().map(|sk| {
                    let id = sk.id;
                    let active = app.active_skills.contains(&id);
                    h_flex()
                        .id(("skill", id_index_skill(id)))
                        .items_start()
                        .gap(px(11.0))
                        .w_full()
                        .p(px(11.0))
                        .rounded(px(10.0))
                        .border_1()
                        .border_color(if active { theme::hexa(0xd9a06666) } else { theme::white(0.08) })
                        .bg(if active { theme::hexa(0xd9a0661a) } else { theme::elevated() })
                        .cursor_pointer()
                        .on_click(cx.listener(move |a, _, _, cx| a.toggle_skill(id, cx)))
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
                                .border_color(if active { theme::gold() } else { theme::white(0.2) })
                                .bg(if active { theme::gold() } else { gpui::transparent_black() })
                                .when(active, |d| d.child(icon("check", 12.0, theme::window_bg()))),
                        )
                        .child(
                            v_flex()
                                .flex_1()
                                .child(div().text_size(px(13.5)).font_semibold().text_color(theme::text()).child(sk.name))
                                .child(div().mt(px(2.0)).text_size(px(11.5)).text_color(theme::muted()).child(sk.desc)),
                        )
                })),
        );
    let close = cx.listener(|a, _, _, cx| a.close_skills(cx));
    layer("skills-back", close, panel)
}

pub fn activity(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let (status_label, status_color) = app.status.label_color(app.generated);
    let panel = v_flex()
        .absolute()
        .top(px(4.0))
        .right(px(84.0))
        .w(px(348.0))
        .max_h(px(420.0))
        .bg(theme::panel())
        .border_1()
        .border_color(theme::border_strong())
        .rounded(px(14.0))
        .shadow_lg()
        .overflow_hidden()
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .px(px(16.0))
                .py(px(14.0))
                .border_b_1()
                .border_color(theme::hairline())
                .child(div().font_family(theme::FONT_DISPLAY).font_semibold().text_size(px(14.5)).child("Activity"))
                .child(div().text_size(px(12.0)).font_semibold().text_color(status_color).child(status_label)),
        )
        .child(
            v_flex()
                .id("activity-body")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .px(px(6.0))
                .py(px(8.0))
                .when(app.activity.is_empty(), |this| {
                    this.child(
                        div()
                            .py(px(34.0))
                            .px(px(16.0))
                            .text_center()
                            .text_size(px(13.0))
                            .text_color(theme::faint())
                            .line_height(px(21.0))
                            .child("Nothing yet. Generate a site and every compile step and auto-fix shows up here."),
                    )
                })
                .children(app.activity.iter().map(|a| {
                    h_flex()
                        .gap(px(11.0))
                        .px(px(11.0))
                        .py(px(9.0))
                        .items_start()
                        .child(div().flex_none().size(px(8.0)).mt(px(5.0)).rounded_full().bg(a.tone.dot()))
                        .child(div().flex_1().text_size(px(13.0)).text_color(theme::text_soft()).line_height(px(19.0)).child(a.text.clone()))
                })),
        );
    let close = cx.listener(|a, _, _, cx| a.close_menus(cx));
    layer("activity-back", close, panel)
}

pub fn toast(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    div()
        .absolute()
        .bottom(px(96.0))
        .left_0()
        .right_0()
        .flex()
        .justify_center()
        .child(
            h_flex()
                .items_center()
                .gap(px(13.0))
                .max_w(px(560.0))
                .px(px(15.0))
                .py(px(13.0))
                .bg(theme::toast_bg())
                .border_1()
                .border_color(theme::hexa(0xe5a54b80))
                .rounded(px(13.0))
                .shadow_lg()
                .child(icon("alert-triangle", 20.0, theme::warn()))
                .child(
                    v_flex()
                        .flex_1()
                        .child(div().text_size(px(13.5)).font_semibold().child("Couldn\u{2019}t add the order form automatically"))
                        .child(
                            div()
                                .mt(px(2.0))
                                .text_size(px(12.5))
                                .text_color(theme::hex(0xc9b79a))
                                .child(format!("Tried {} times \u{2014} it needs a payment detail from you. Your design was left untouched.", app.heal_attempts)),
                        ),
                )
                .child(
                    div()
                        .id("toast-ok")
                        .px(px(13.0))
                        .py(px(8.0))
                        .rounded(px(9.0))
                        .border_1()
                        .border_color(theme::hexa(0xe5a54b66))
                        .text_color(theme::warn())
                        .text_size(px(12.5))
                        .font_semibold()
                        .cursor_pointer()
                        .child("Got it")
                        .on_click(cx.listener(|a, _, _, cx| a.dismiss_toast(cx))),
                ),
        )
}

/// A dimmed backdrop that catches outside clicks, with the panel on top.
fn layer(
    id: &'static str,
    on_outside: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    panel: impl IntoElement,
) -> impl IntoElement {
    div()
        .absolute()
        .inset_0()
        .child(div().id(id).absolute().inset_0().on_click(on_outside))
        .child(panel)
}

fn id_index(id: ProviderId) -> usize {
    PROVIDERS.iter().position(|p| p.id == id).unwrap_or(0)
}
fn id_index_skill(id: crate::state::SkillId) -> usize {
    SKILLS.iter().position(|s| s.id == id).unwrap_or(0)
}
