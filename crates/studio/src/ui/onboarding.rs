//! Onboarding: choose a connection (provider key or ACP agent), connect it,
//! then pick a starting point. A three-step flow with a clickable rail.

use gpui::{App, ClickEvent, Context, Window, div, prelude::*, px};
use gpui_component::{StyledExt, h_flex, input::Input, v_flex};

use crate::app::StudioApp;
use crate::state::{ConnMode, PROVIDERS};
use crate::theme;
use crate::ui::widgets::{Btn, icon, provider_card};

pub fn render(app: &StudioApp, _window: &mut Window, cx: &mut Context<StudioApp>) -> impl IntoElement {
    h_flex().flex_1().min_h_0().child(rail(app, cx)).child(content(app, cx))
}

// ── left rail: step indicator ────────────────────────────────────────────────
fn rail(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let labels = ["Choose provider", "Connect key", "Pick a start"];
    v_flex()
        .w(px(296.0))
        .flex_none()
        .h_full()
        .p(px(32.0))
        .bg(theme::bg_base())
        .border_r_1()
        .border_color(theme::line_faint())
        .child(
            v_flex()
                .mt(px(8.0))
                .gap(px(22.0))
                .children(labels.iter().enumerate().map(|(i, label)| step_row(i as u8, label, app.ob_step, cx))),
        )
        .child(div().flex_1())
        .child(
            h_flex()
                .items_center()
                .gap(px(8.0))
                .px(px(13.0))
                .py(px(11.0))
                .rounded(px(theme::RADIUS_MD))
                .bg(theme::success_tint())
                .text_size(px(12.0))
                .font_semibold()
                .text_color(theme::success())
                .child(icon("shield", 15.0, theme::success()))
                .child("Local-first \u{b7} nothing leaves this device"),
        )
        .child(
            div()
                .mt(px(14.0))
                .text_size(px(12.0))
                .text_color(theme::text_faint())
                .line_height(px(19.0))
                .child("Bring your own key. Free while in alpha \u{2014} every project runs on your machine."),
        )
}

fn step_row(i: u8, label: &'static str, step: u8, cx: &mut Context<StudioApp>) -> impl IntoElement + use<> {
    let done = i < step;
    let current = i == step;
    let (badge_bg, badge_fg, border): (gpui::Hsla, gpui::Hsla, Option<gpui::Hsla>) = if current {
        (theme::accent(), theme::accent_contrast(), None)
    } else if done {
        (theme::success_tint(), theme::success(), None)
    } else {
        (theme::bg_raised(), theme::text_caption(), Some(theme::line()))
    };
    let mut badge = div()
        .size(px(28.0))
        .rounded(px(theme::RADIUS_SM))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(12.5))
        .font_bold()
        .bg(badge_bg)
        .text_color(badge_fg);
    if let Some(b) = border {
        badge = badge.border_1().border_color(b);
    }
    let badge = if done { badge.child(icon("check", 14.0, theme::success())) } else { badge.child(format!("{}", i + 1)) };

    h_flex()
        .id(("ob-step", i as usize))
        .items_center()
        .gap(px(13.0))
        .cursor_pointer()
        .child(badge)
        .child(
            div()
                .text_size(px(13.5))
                .when(current, |d| d.font_semibold())
                .when(!current, |d| d.font_medium())
                .text_color(if current {
                    theme::text_strong()
                } else if done {
                    theme::text_caption()
                } else {
                    theme::text_faint()
                })
                .child(label),
        )
        .on_click(cx.listener(move |a, _, _, cx| a.goto_step(i, cx)))
}

// ── right content ────────────────────────────────────────────────────────────
fn content(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let inner = match app.ob_step {
        0 => step_provider(app, cx).into_any_element(),
        1 => step_key(app, cx).into_any_element(),
        _ => step_start(cx).into_any_element(),
    };
    v_flex().id("ob-content").flex_1().h_full().min_w_0().flex().justify_center().p(px(56.0)).overflow_y_scroll().child(inner)
}

fn title(text: impl Into<gpui::SharedString>) -> impl IntoElement {
    div()
        .font_family(theme::FONT_DISPLAY)
        .text_size(px(27.0))
        .font_semibold()
        .text_color(theme::text_strong())
        .child(text.into())
}
fn subtitle(text: &'static str) -> impl IntoElement {
    div().mt(px(9.0)).text_size(px(15.0)).text_color(theme::text_muted()).line_height(px(24.0)).child(text)
}

fn step_provider(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let key = app.conn_mode == ConnMode::Key;
    let mut body = v_flex()
        .max_w(px(680.0))
        .child(title("How do you want to connect?"))
        .child(subtitle("Bring your own provider key, or connect an agent over ACP. You can switch anytime in Settings."))
        .child(conn_toggle(app, cx));
    if key {
        body = body.child(
            h_flex().mt(px(20.0)).flex_wrap().gap(px(12.0)).children(PROVIDERS.iter().enumerate().map(|(i, p)| {
                let id = p.id;
                div().w(px(210.0)).child(provider_card(
                    ("prov", i),
                    p.mono,
                    p.mono_bg,
                    p.name,
                    p.by,
                    app.provider == p.id,
                    p.recommended,
                    cx.listener(move |a, _, _, cx| a.pick_provider(id, cx)),
                ))
            })),
        );
    } else {
        body = body.child(acp_info());
    }
    body.child(
        h_flex()
            .mt(px(32.0))
            .gap(px(11.0))
            .child(Btn::secondary("Skip for now").render("ob-skip", cx.listener(|a, _, window, cx| a.skip_onboarding(window, cx))))
            .child(Btn::primary("Continue").icon_right("arrow-right").render("ob-next0", cx.listener(|a, _, _, cx| a.next_step(cx)))),
    )
}

fn conn_toggle(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    h_flex()
        .mt(px(22.0))
        .max_w(px(420.0))
        .gap(px(4.0))
        .p(px(4.0))
        .bg(theme::bg_sunken())
        .rounded(px(11.0))
        .child(mode_btn("cm-key", "Use your own key", app.conn_mode == ConnMode::Key, cx.listener(|a, _, _, cx| a.set_conn_mode(ConnMode::Key, cx))))
        .child(mode_btn("cm-acp", "Connect an agent \u{b7} ACP", app.conn_mode == ConnMode::Acp, cx.listener(|a, _, _, cx| a.set_conn_mode(ConnMode::Acp, cx))))
}

fn mode_btn(
    id: &'static str,
    label: &'static str,
    active: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let mut b = div()
        .id(id)
        .flex_1()
        .h(px(38.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(theme::RADIUS_SM))
        .text_size(px(13.0))
        .font_semibold()
        .cursor_pointer()
        .on_click(on_click);
    if active {
        b = b.bg(theme::accent()).text_color(theme::accent_contrast());
    } else {
        b = b.text_color(theme::text_soft()).hover(|s| s.text_color(theme::text_strong()));
    }
    b.child(label)
}

fn acp_info() -> impl IntoElement {
    h_flex()
        .mt(px(20.0))
        .gap(px(15.0))
        .items_start()
        .p(px(20.0))
        .rounded(px(theme::RADIUS_LG))
        .border_1()
        .border_color(theme::accent_ring())
        .bg(theme::accent_tint())
        .child(div().size(px(42.0)).flex_none().rounded(px(12.0)).bg(theme::bg_panel()).flex().items_center().justify_center().child(icon("link", 20.0, theme::accent())))
        .child(
            v_flex()
                .flex_1()
                .child(div().text_size(px(15.0)).font_semibold().text_color(theme::text_strong()).child("Connect any ACP agent"))
                .child(div().mt(px(5.0)).text_size(px(13.0)).text_color(theme::text_muted()).line_height(px(21.0)).child("Point WebFluent at an Agent Client Protocol endpoint \u{2014} a local run command or a WebSocket URL. Your model, credentials and context stay entirely with the agent.")),
        )
}

fn step_key(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let key = app.conn_mode == ConnMode::Key;
    let title_text = if key { "Connect your key" } else { "Connect your agent" };
    let mut body = v_flex().max_w(px(560.0)).child(title(title_text));
    if key {
        let valid = app.key_valid(cx);
        body = body
            .child(subtitle("Paste your API key. It\u{2019}s saved to your operating system\u{2019}s secure store and never leaves this device."))
            .child(field_row("lock", Input::new(&app.api_key).appearance(false).text_size(px(14.0))))
            .child(
                h_flex()
                    .mt(px(13.0))
                    .items_center()
                    .gap(px(8.0))
                    .text_size(px(13.0))
                    .text_color(theme::text_muted())
                    .child(icon("lock", 14.0, theme::success()))
                    .child("Keys are stored with your OS keychain \u{2014} not with us."),
            )
            .child(nav_row(!valid, cx));
    } else {
        let ready = app.acp_url.read(cx).value().trim().len() >= 6;
        body = body
            .child(subtitle("Give WebFluent your agent\u{2019}s endpoint. Credentials never pass through us \u{2014} they stay with the agent."))
            .child(field_row("link", Input::new(&app.acp_url).appearance(false).text_size(px(13.0))))
            .child(nav_row(!ready, cx));
    }
    body
}

fn field_row(icon_name: &'static str, input: Input) -> impl IntoElement {
    h_flex()
        .mt(px(26.0))
        .h(px(46.0))
        .items_center()
        .gap(px(9.0))
        .px(px(13.0))
        .rounded(px(11.0))
        .border_1()
        .border_color(theme::line_strong())
        .bg(theme::bg_sunken())
        .child(icon(icon_name, 16.0, theme::icon_color()))
        .child(div().flex_1().min_w_0().child(input))
}

fn nav_row(next_disabled: bool, cx: &mut Context<StudioApp>) -> impl IntoElement {
    h_flex()
        .mt(px(30.0))
        .gap(px(11.0))
        .child(Btn::secondary("Back").icon("arrow-left").render("ob-back1", cx.listener(|a, _, _, cx| a.prev_step(cx))))
        .child(Btn::primary("Continue").icon_right("arrow-right").disabled(next_disabled).render("ob-next1", cx.listener(|a, _, _, cx| a.next_step(cx))))
}

struct Starter {
    id: &'static str,
    title: &'static str,
    desc: &'static str,
    icon: &'static str,
    tile_bg: gpui::Hsla,
    tile_fg: gpui::Hsla,
    rec: bool,
}

fn step_start(cx: &mut Context<StudioApp>) -> impl IntoElement {
    let starters = [
        Starter { id: "blank", title: "Blank canvas", desc: "Start from an empty page", icon: "plus", tile_bg: theme::bg_hover(), tile_fg: theme::text_soft(), rec: false },
        Starter { id: "venue", title: "Event / venue", desc: "Rooftop, club, live shows", icon: "zap", tile_bg: theme::accent_tint(), tile_fg: theme::accent(), rec: true },
        Starter { id: "cafe", title: "Restaurant / caf\u{e9}", desc: "Menu, hours, location", icon: "sparkle", tile_bg: theme::violet_tint(), tile_fg: theme::violet_soft(), rec: false },
        Starter { id: "portfolio", title: "Portfolio", desc: "Show your work", icon: "image", tile_bg: theme::bg_hover(), tile_fg: theme::text_soft(), rec: false },
        Starter { id: "product", title: "Product landing", desc: "Launch something new", icon: "layers", tile_bg: theme::bg_hover(), tile_fg: theme::text_soft(), rec: false },
        Starter { id: "import", title: "Import a design", desc: "Figma, image or PDF", icon: "paperclip", tile_bg: theme::bg_hover(), tile_fg: theme::text_soft(), rec: false },
    ];
    v_flex()
        .max_w(px(720.0))
        .child(title("Pick a start"))
        .child(subtitle("Start blank, or open with a scaffold. You\u{2019}ll shape everything by conversation next."))
        .child(h_flex().mt(px(26.0)).flex_wrap().gap(px(12.0)).children(starters.into_iter().map(|s| starter_card(s, cx))))
        .child(h_flex().mt(px(30.0)).child(Btn::secondary("Back").icon("arrow-left").render("ob-back2", cx.listener(|a, _, _, cx| a.prev_step(cx)))))
}

fn starter_card(s: Starter, cx: &mut Context<StudioApp>) -> impl IntoElement + use<> {
    let id = s.id;
    v_flex()
        .id(s.id)
        .relative()
        .w(px(212.0))
        .p(px(16.0))
        .gap(px(10.0))
        .rounded(px(theme::RADIUS_LG))
        .border_1()
        .border_color(theme::line())
        .bg(theme::bg_panel())
        .cursor_pointer()
        .text_color(theme::text_body())
        .hover(|st| st.border_color(theme::line_bright()).bg(theme::bg_raised()))
        .on_click(cx.listener(move |a, _, window, cx| a.pick_starter(id, window, cx)))
        .child(div().size(px(36.0)).rounded(px(10.0)).bg(s.tile_bg).flex().items_center().justify_center().child(icon(s.icon, 18.0, s.tile_fg)))
        .child(div().text_size(px(14.0)).font_semibold().text_color(theme::text_strong()).child(s.title))
        .child(div().text_size(px(12.5)).text_color(theme::text_caption()).line_height(px(18.0)).child(s.desc))
        .when(s.rec, |d| {
            d.child(
                div()
                    .absolute()
                    .top(px(12.0))
                    .right(px(12.0))
                    .px(px(6.0))
                    .py(px(3.0))
                    .rounded(px(theme::RADIUS_XS))
                    .bg(theme::accent_tint())
                    .text_color(theme::accent())
                    .text_size(px(9.5))
                    .font_bold()
                    .child("PICK"),
            )
        })
}
