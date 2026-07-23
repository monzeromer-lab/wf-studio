//! Sign-in screen: email + password, or Google / Apple SSO.

use gpui::{App, ClickEvent, Context, Window, div, prelude::*, px};
use gpui_component::{StyledExt, h_flex, input::Input, v_flex};

use crate::app::StudioApp;
use crate::theme;
use crate::ui::widgets::{Btn, brand_badge, icon};

pub fn render(app: &StudioApp, _window: &mut Window, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let ready = app.login_ready(cx);

    div()
        .flex_1()
        .min_h_0()
        .flex()
        .items_center()
        .justify_center()
        .p(px(40.0))
        .child(
            v_flex()
                .w_full()
                .max_w(px(360.0))
                .child(div().mb(px(22.0)).child(brand_badge(52.0, 26.0)))
                .child(
                    div()
                        .font_family(theme::FONT_DISPLAY)
                        .text_size(px(25.0))
                        .font_semibold()
                        .text_color(theme::text_strong())
                        .child("Sign in to WebFluent"),
                )
                .child(
                    div()
                        .mt(px(8.0))
                        .text_size(px(14.0))
                        .text_color(theme::text_muted())
                        .line_height(px(22.0))
                        .child("Build websites by conversation \u{2014} in Arabic or English."),
                )
                .child(
                    v_flex()
                        .mt(px(26.0))
                        .gap(px(10.0))
                        .child(input_row("user", Input::new(&app.login_email)))
                        .child(input_row("lock", Input::new(&app.login_pw))),
                )
                .child(
                    div().mt(px(16.0)).child(
                        Btn::primary("Sign in")
                            .full()
                            .loading(app.login_busy)
                            .disabled(!ready)
                            .render("signin", cx.listener(|a, _, _, cx| a.sign_in(cx))),
                    ),
                )
                .child(divider())
                .child(
                    h_flex()
                        .gap(px(10.0))
                        .child(sso_btn("sso-google", "google", "Google", cx.listener(|a, _, _, cx| a.sso_sign_in(cx))))
                        .child(sso_btn("sso-apple", "apple", "Apple", cx.listener(|a, _, _, cx| a.sso_sign_in(cx)))),
                )
                .child(
                    h_flex()
                        .mt(px(22.0))
                        .justify_center()
                        .gap(px(5.0))
                        .text_size(px(12.5))
                        .text_color(theme::text_caption())
                        .child("New to WebFluent?")
                        .child(
                            div()
                                .id("create-account")
                                .text_color(theme::accent())
                                .font_semibold()
                                .cursor_pointer()
                                .child("Create an account")
                                .on_click(cx.listener(|a, _, _, cx| a.sso_sign_in(cx))),
                        ),
                ),
        )
}

fn input_row(icon_name: &'static str, input: Input) -> impl IntoElement {
    h_flex()
        .h(px(46.0))
        .items_center()
        .gap(px(9.0))
        .px(px(13.0))
        .rounded(px(11.0))
        .border_1()
        .border_color(theme::line_strong())
        .bg(theme::bg_sunken())
        .child(icon(icon_name, 16.0, theme::icon_color()))
        .child(div().flex_1().min_w_0().child(input.appearance(false).text_size(px(14.0))))
}

fn divider() -> impl IntoElement {
    h_flex()
        .my(px(20.0))
        .items_center()
        .gap(px(12.0))
        .child(div().flex_1().h(px(1.0)).bg(theme::line()))
        .child(div().text_size(px(11.5)).text_color(theme::text_faint()).child("or continue with"))
        .child(div().flex_1().h(px(1.0)).bg(theme::line()))
}

fn sso_btn(
    id: &'static str,
    icon_name: &'static str,
    label: &'static str,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    h_flex()
        .id(id)
        .flex_1()
        .h(px(42.0))
        .items_center()
        .justify_center()
        .gap(px(8.0))
        .rounded(px(11.0))
        .border_1()
        .border_color(theme::line_strong())
        .bg(theme::bg_raised())
        .text_size(px(13.0))
        .font_semibold()
        .text_color(theme::text_soft())
        .cursor_pointer()
        .hover(|s| s.bg(theme::bg_hover()).text_color(theme::text_strong()))
        .child(icon(icon_name, 16.0, theme::text_soft()))
        .child(label)
        .on_click(on_click)
}
