//! First-run onboarding (mock's `screen: 'onboarding'`): pick a provider →
//! paste a key → choose a starting point. Three steps with a progress rail.

use gpui::{Context, Window, div, prelude::*, px};
use gpui_component::{StyledExt, h_flex, input::Input, v_flex};

use crate::app::StudioApp;
use crate::state::{PROVIDERS, ProviderId, SAMPLES, Tested};
use crate::theme;
use crate::ui::widgets::{brand_badge, icon};

pub fn render(app: &StudioApp, window: &mut Window, cx: &mut Context<StudioApp>) -> impl IntoElement {
    h_flex()
        .flex_1()
        .min_h_0()
        .bg(theme::window_bg())
        .child(rail(app))
        .child(content(app, window, cx))
}

fn rail(app: &StudioApp) -> impl IntoElement {
    let steps = ["Choose provider", "Connect key", "Pick a start"];
    v_flex()
        .h_full()
        .flex_none()
        .w(px(300.0))
        .px(px(34.0))
        .py(px(40.0))
        .bg(theme::ob_rail())
        .border_r_1()
        .border_color(theme::white(0.06))
        .child(
            h_flex()
                .items_center()
                .gap(px(10.0))
                .font_family(theme::FONT_DISPLAY)
                .font_semibold()
                .text_size(px(17.0))
                .child(brand_badge(30.0, 17.0))
                .child("WebFluent"),
        )
        .child(
            v_flex().mt(px(44.0)).gap(px(22.0)).children(steps.into_iter().enumerate().map(|(i, s)| {
                let done = (i as u8) < app.ob_step;
                let cur = i as u8 == app.ob_step;
                let (dot_bg, dot_fg) = if cur {
                    (theme::accent(), theme::white(1.0))
                } else if done {
                    (theme::hexa(0x63c0882e), theme::success())
                } else {
                    (theme::elevated(), theme::faint())
                };
                h_flex()
                    .items_center()
                    .gap(px(13.0))
                    .child(
                        div()
                            .flex_none()
                            .size(px(28.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(8.0))
                            .bg(dot_bg)
                            .text_color(dot_fg)
                            .text_size(px(13.0))
                            .font_bold()
                            .child(if done { "\u{2713}".to_string() } else { (i + 1).to_string() }),
                    )
                    .child(
                        div()
                            .text_size(px(14.0))
                            .text_color(if cur { theme::text() } else if done { theme::muted() } else { theme::faint() })
                            .when(cur, |d| d.font_semibold())
                            .child(s),
                    )
            })),
        )
        .child(div().flex_1())
        .child(
            div()
                .text_size(px(12.0))
                .text_color(theme::faint())
                .line_height(px(19.0))
                .child("Bring your own key. Free while in alpha. Everything runs on your machine."),
        )
}

fn content(app: &StudioApp, _window: &mut Window, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let step = app.ob_step;
    let next_enabled = step != 1 || app.key_valid(cx);

    v_flex()
        .id("ob-content")
        .flex_1()
        .h_full()
        .min_w_0()
        .px(px(54.0))
        .py(px(46.0))
        .overflow_y_scroll()
        .child(match step {
            0 => step_provider(app, cx).into_any_element(),
            1 => step_key(app, cx).into_any_element(),
            _ => step_samples(cx).into_any_element(),
        })
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .mt(px(30.0))
                .child(
                    div()
                        .id("ob-back")
                        .px(px(20.0))
                        .py(px(11.0))
                        .rounded(px(10.0))
                        .border_1()
                        .border_color(theme::white(0.12))
                        .text_color(if step == 0 { theme::disabled() } else { theme::text_dim() })
                        .text_size(px(14.0))
                        .font_semibold()
                        .cursor_pointer()
                        .child("Back")
                        .on_click(cx.listener(|a, _, _, cx| a.ob_back(cx))),
                )
                .when(step < 2, |this| {
                    this.child(
                        div()
                            .id("ob-next")
                            .px(px(26.0))
                            .py(px(11.0))
                            .rounded(px(10.0))
                            .bg(if next_enabled { theme::accent() } else { theme::seg_active() })
                            .text_color(if next_enabled { theme::white(1.0) } else { theme::faint() })
                            .text_size(px(14.0))
                            .font_semibold()
                            .when(next_enabled, |d| d.cursor_pointer())
                            .child("Continue")
                            .on_click(cx.listener(|a, _, _, cx| a.ob_next(cx))),
                    )
                }),
        )
}

fn step_provider(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    v_flex()
        .max_w(px(660.0))
        .child(div().font_family(theme::FONT_DISPLAY).font_semibold().text_size(px(27.0)).child("Choose your AI provider"))
        .child(div().mt(px(9.0)).text_size(px(15.0)).text_color(theme::muted()).line_height(px(24.0)).child("WebFluent works with six providers. Pick one \u{2014} you can switch anytime in Settings."))
        .child(
            h_flex().mt(px(26.0)).flex_wrap().gap(px(12.0)).children(PROVIDERS.iter().map(|p| {
                let id = p.id;
                let active = app.provider == id;
                h_flex()
                    .id(("ob-prov", index(id)))
                    .relative()
                    .w(px(200.0))
                    .items_center()
                    .gap(px(11.0))
                    .p(px(14.0))
                    .rounded(px(13.0))
                    .border_1()
                    .border_color(if active { theme::accent() } else { theme::white(0.1) })
                    .bg(if active { theme::hexa(0xe2725b1a) } else { theme::panel() })
                    .cursor_pointer()
                    .on_click(cx.listener(move |a, _, _, cx| a.pick_provider(id, cx)))
                    .child(
                        div()
                            .size(px(34.0))
                            .rounded(px(9.0))
                            .bg(theme::hex(p.mono_bg))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(theme::white(1.0))
                            .font_bold()
                            .text_size(px(14.0))
                            .child(p.mono),
                    )
                    .child(
                        v_flex()
                            .child(div().text_size(px(14.5)).font_semibold().text_color(theme::text()).child(p.name))
                            .child(div().text_size(px(12.0)).text_color(theme::muted()).child(p.by)),
                    )
                    .when(p.recommended, |d| {
                        d.child(
                            div()
                                .absolute()
                                .bottom(px(8.0))
                                .right(px(10.0))
                                .px(px(6.0))
                                .py(px(2.0))
                                .rounded(px(5.0))
                                .bg(theme::hexa(0xe2725b24))
                                .text_color(theme::accent())
                                .text_size(px(9.0))
                                .font_bold()
                                .child("PICK"),
                        )
                    })
            })),
        )
}

fn step_key(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let name = app.provider().name;
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

    v_flex()
        .max_w(px(520.0))
        .child(div().font_family(theme::FONT_DISPLAY).font_semibold().text_size(px(27.0)).child(format!("Connect your {name} key")))
        .child(div().mt(px(9.0)).text_size(px(15.0)).text_color(theme::muted()).line_height(px(24.0)).child("Paste your API key. It\u{2019}s saved to your operating system\u{2019}s secure store and never leaves this device."))
        .child(
            h_flex()
                .mt(px(26.0))
                .items_center()
                .gap(px(10.0))
                .px(px(15.0))
                .py(px(13.0))
                .bg(theme::panel())
                .border_1()
                .border_color(key_border)
                .rounded(px(12.0))
                .child(icon("lock", 17.0, theme::muted()))
                .child(div().flex_1().min_w_0().child(Input::new(&app.api_key).appearance(false).text_size(px(14.0))))
                .child(
                    div()
                        .id("ob-test")
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
        .when(app.tested == Tested::Fail, |this| {
            this.child(
                h_flex()
                    .mt(px(13.0))
                    .items_center()
                    .gap(px(8.0))
                    .text_size(px(13.0))
                    .text_color(theme::danger())
                    .child(icon("alert-circle", 14.0, theme::danger()))
                    .child("That key doesn\u{2019}t look right. Paste the full key from your provider\u{2019}s dashboard."),
            )
        })
        .child(
            h_flex()
                .mt(px(13.0))
                .items_center()
                .gap(px(8.0))
                .text_size(px(13.0))
                .text_color(theme::muted())
                .child(icon("lock", 14.0, theme::success()))
                .child("Keys are stored with your OS keychain \u{2014} not with us."),
        )
}

fn step_samples(cx: &mut Context<StudioApp>) -> impl IntoElement {
    v_flex()
        .max_w(px(620.0))
        .child(div().font_family(theme::FONT_DISPLAY).font_semibold().text_size(px(27.0)).child("Pick a starting point"))
        .child(div().mt(px(9.0)).text_size(px(15.0)).text_color(theme::muted()).line_height(px(24.0)).child("We\u{2019}ll drop a prompt in for you \u{2014} edit it, or write your own once you\u{2019}re in."))
        .child(
            v_flex().mt(px(24.0)).gap(px(11.0)).children(SAMPLES.iter().enumerate().map(|(i, s)| {
                let prompt = s.prompt;
                h_flex()
                    .id(("sample", i))
                    .items_center()
                    .gap(px(15.0))
                    .px(px(17.0))
                    .py(px(15.0))
                    .rounded(px(13.0))
                    .border_1()
                    .border_color(theme::white(0.1))
                    .bg(theme::panel())
                    .cursor_pointer()
                    .hover(|st| st.border_color(theme::accent()))
                    .on_click(cx.listener(move |a, _, window, cx| a.pick_sample(prompt, window, cx)))
                    .child(
                        div()
                            .flex_none()
                            .size(px(40.0))
                            .rounded(px(11.0))
                            .bg(theme::hex(s.icon_bg.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(18.0))
                            .child(s.icon),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .child(div().text_size(px(15.0)).font_semibold().text_color(theme::text()).child(s.title))
                            .child(div().mt(px(2.0)).text_size(px(13.0)).text_color(theme::muted()).line_height(px(18.0)).child(s.desc)),
                    )
                    .child(icon("chevron-right", 18.0, theme::faint()))
            })),
        )
        .child(
            div()
                .id("start-blank")
                .mt(px(4.0))
                .flex()
                .justify_center()
                .py(px(12.0))
                .rounded(px(11.0))
                .border_1()
                .border_color(theme::white(0.16))
                .text_color(theme::muted())
                .text_size(px(13.5))
                .font_semibold()
                .cursor_pointer()
                .child("Start from a blank prompt instead")
                .on_click(cx.listener(|a, _, window, cx| a.start_blank(window, cx))),
        )
}

fn index(id: ProviderId) -> usize {
    PROVIDERS.iter().position(|p| p.id == id).unwrap_or(0)
}
