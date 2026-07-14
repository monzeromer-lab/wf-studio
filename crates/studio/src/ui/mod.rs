//! The Studio view tree. Each submodule renders one region of the mock; this
//! module wires them into the window and layers the popovers/toast on top.

mod canvas;
mod chat;
mod onboarding;
pub(crate) mod overlays;
mod sidebar;
mod title_bar;
mod toolbar;
mod widgets;

use gpui::{AnyElement, Context, Window, prelude::*, div};
use gpui_component::{h_flex, v_flex};

use crate::app::StudioApp;
use crate::state::Screen;
use crate::theme;

/// Root of the window: the title bar, then either onboarding or the studio.
pub fn render(app: &StudioApp, window: &mut Window, cx: &mut Context<StudioApp>) -> impl IntoElement {
    v_flex()
        .size_full()
        .bg(theme::window_bg())
        .text_color(theme::text())
        .font_family(theme::FONT_UI)
        .child(title_bar::render(app, window, cx))
        .child(match app.screen {
            Screen::Onboarding => onboarding::render(app, window, cx).into_any_element(),
            Screen::Studio => studio_body(app, window, cx).into_any_element(),
        })
        .when(app.show_skills, |this| this.child(overlays::skills(app, cx)))
        .when(app.toast, |this| this.child(overlays::toast(app, cx)))
        .into_any_element()
}

/// The three-column studio (chat · canvas · sidebar) plus the overlay layer.
fn studio_body(app: &StudioApp, window: &mut Window, cx: &mut Context<StudioApp>) -> AnyElement {
    let show_right = app.review_open || app.show_history || app.show_activity;

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
                .child(chat::render(app, window, cx))
                .child(div().flex_1().min_w_0().h_full().child(canvas::render(app, window, cx)))
                .when(show_right, |this| this.child(sidebar::render(app, window, cx))),
        )
        .into_any_element()
}
