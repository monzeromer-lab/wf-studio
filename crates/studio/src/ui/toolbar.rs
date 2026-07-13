//! Toolbar strip: RTL/LTR direction (FR-11), device size (FR-12), undo/redo
//! (FR-14), and the live preview URL.

use gpui::{App, ClickEvent, Context, Window, div, prelude::*, px};
use gpui_component::{StyledExt, h_flex};

use crate::app::StudioApp;
use crate::state::{Device, Dir};
use crate::theme;
use crate::ui::widgets::icon;

pub fn render(app: &StudioApp, _window: &mut Window, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let rtl = app.dir == Dir::Rtl;
    let dev = app.device;
    let preview_url = if rtl { "yasmine.cafe" } else { "yasmine.cafe/en" };

    h_flex()
        .flex_none()
        .h(px(44.0))
        .items_center()
        .gap(px(10.0))
        .px(px(14.0))
        .bg(theme::toolbar())
        .border_b_1()
        .border_color(theme::white(0.06))
        .child(
            seg_group()
                .child(seg("seg-rtl", text_seg("RTL \u{b7} \u{639}\u{631}\u{628}\u{64a}"), rtl, cx.listener(|a, _, _, cx| a.set_dir(Dir::Rtl, cx))))
                .child(seg("seg-ltr", text_seg("LTR \u{b7} EN"), !rtl, cx.listener(|a, _, _, cx| a.set_dir(Dir::Ltr, cx)))),
        )
        .child(
            seg_group()
                .child(seg("seg-desk", icon_seg("desktop", dev == Device::Desktop), dev == Device::Desktop, cx.listener(|a, _, _, cx| a.set_device(Device::Desktop, cx))))
                .child(seg("seg-tab", icon_seg("tablet", dev == Device::Tablet), dev == Device::Tablet, cx.listener(|a, _, _, cx| a.set_device(Device::Tablet, cx))))
                .child(seg("seg-mob", icon_seg("mobile", dev == Device::Mobile), dev == Device::Mobile, cx.listener(|a, _, _, cx| a.set_device(Device::Mobile, cx)))),
        )
        .child(div().w(px(1.0)).h(px(20.0)).bg(theme::white(0.08)))
        .child(
            h_flex()
                .gap(px(5.0))
                .child(nav_btn("undo", "undo", app.can_undo(), cx.listener(|a, _, _, cx| a.undo(cx))))
                .child(nav_btn("redo", "redo", app.can_redo(), cx.listener(|a, _, _, cx| a.redo(cx)))),
        )
        .child(div().flex_1())
        .child(
            h_flex()
                .items_center()
                .gap(px(7.0))
                .px(px(12.0))
                .py(px(4.0))
                .rounded(px(8.0))
                .bg(theme::sunken())
                .border_1()
                .border_color(theme::white(0.06))
                .child(icon("check-circle", 13.0, theme::success()))
                .child(div().text_size(px(12.5)).text_color(theme::muted()).child(preview_url)),
        )
}

fn seg_group() -> gpui::Div {
    div()
        .flex()
        .p(px(3.0))
        .bg(theme::sunken())
        .rounded(px(9.0))
        .border_1()
        .border_color(theme::white(0.06))
}

fn text_seg(label: &'static str) -> impl IntoElement {
    div().text_size(px(12.5)).font_semibold().child(label)
}

fn icon_seg(name: &'static str, active: bool) -> impl IntoElement {
    icon(name, 15.0, if active { theme::text() } else { theme::muted() })
}

fn seg<E: IntoElement>(
    id: &'static str,
    child: E,
    active: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let base = div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .gap(px(5.0))
        .px(px(12.0))
        .py(px(5.0))
        .rounded(px(7.0))
        .cursor_pointer()
        .child(child)
        .on_click(on_click);
    if active {
        base.bg(theme::seg_active()).text_color(theme::text()).shadow_sm()
    } else {
        base.text_color(theme::muted()).hover(|s| s.bg(theme::white(0.03)))
    }
}

fn nav_btn(
    id: &'static str,
    name: &'static str,
    enabled: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .size(px(30.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(8.0))
        .border_1()
        .border_color(theme::white(0.06))
        .bg(theme::sunken())
        .cursor_pointer()
        .child(icon(name, 15.0, if enabled { theme::text_dim() } else { theme::disabled() }))
        .on_click(on_click)
}
