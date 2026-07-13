use gpui::{Context, Window, div, prelude::*};
use gpui_component::{
    ActiveTheme,
    button::{Button, ButtonVariants},
    h_flex,
    input::Input,
};

use crate::app::StudioApp;

pub fn prompt_dock(
    app: &StudioApp,
    _window: &mut Window,
    cx: &mut Context<StudioApp>,
) -> impl IntoElement {
    h_flex()
        .items_center()
        .gap_2()
        .p_3()
        .border_t_1()
        .border_color(cx.theme().border)
        .child(div().flex_1().child(Input::new(&app.prompt_state)))
        .child(
            Button::new("generate")
                .primary()
                .label("إنشاء · Generate")
                .on_click(cx.listener(|this, _, _, cx| {
                    // M0: the prompt is not sent anywhere yet (AI lands in M1);
                    // Generate recompiles the sample document end-to-end.
                    this.recompile(cx);
                })),
        )
}
