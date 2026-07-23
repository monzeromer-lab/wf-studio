//! Workspace toolbar: RTL/LTR direction, device size, undo/redo, the live
//! site URL, and Share / Publish.

use gpui::{App, ClickEvent, Context, Window, div, prelude::*, px};
use gpui_component::{StyledExt, h_flex};

use crate::app::StudioApp;
use crate::state::{Device, Dir, Modal};
use crate::theme;
use crate::ui::widgets::{Btn, icon};

pub fn render(app: &StudioApp, _window: &mut Window, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let rtl = app.dir == Dir::Rtl;
    let dev = app.device;

    h_flex()
        .flex_none()
        .h(px(46.0))
        .items_center()
        .gap(px(10.0))
        .px(px(14.0))
        .bg(theme::bg_toolbar())
        .border_b_1()
        .border_color(theme::line())
        .child(
            seg_group()
                .child(text_seg("seg-rtl", "RTL \u{b7} \u{639}\u{631}\u{628}\u{64a}", rtl, cx.listener(|a, _, _, cx| a.set_dir(Dir::Rtl, cx))))
                .child(text_seg("seg-ltr", "LTR \u{b7} EN", !rtl, cx.listener(|a, _, _, cx| a.set_dir(Dir::Ltr, cx)))),
        )
        .child(vsep())
        .child(
            seg_group()
                .child(icon_seg("seg-desk", "monitor", dev == Device::Desktop, cx.listener(|a, _, _, cx| a.set_device(Device::Desktop, cx))))
                .child(icon_seg("seg-tab", "tablet", dev == Device::Tablet, cx.listener(|a, _, _, cx| a.set_device(Device::Tablet, cx))))
                .child(icon_seg("seg-mob", "phone", dev == Device::Mobile, cx.listener(|a, _, _, cx| a.set_device(Device::Mobile, cx)))),
        )
        .child(vsep())
        .child(icon_btn("undo", "undo", app.can_undo(), cx.listener(|a, _, _, cx| a.undo(cx))))
        .child(icon_btn("redo", "redo", app.can_redo(), cx.listener(|a, _, _, cx| a.redo(cx))))
        .child(div().flex_1())
        .child(
            h_flex()
                .items_center()
                .gap(px(8.0))
                .h(px(30.0))
                .px(px(12.0))
                .rounded(px(theme::RADIUS_SM))
                .bg(theme::bg_sunken())
                .border_1()
                .border_color(theme::line_faint())
                .text_size(px(12.5))
                .text_color(theme::text_muted())
                .child(icon("check-circle", 14.0, theme::success()))
                .child(div().font_family(theme::FONT_MONO).text_color(theme::text_soft()).child("layali.webfluent.app"))
                .child(icon("refresh", 14.0, theme::icon_color())),
        )
        .child(Btn::secondary("Share").sm().icon("share").render("tb-share", cx.listener(|a, _, _, cx| a.open_modal(Modal::Share, cx))))
        .child(Btn::primary("Publish").sm().icon("cloud").render("tb-publish", cx.listener(|a, _, _, cx| a.open_modal(Modal::Publish, cx))))
}

fn seg_group() -> gpui::Div {
    div().flex().gap(px(2.0)).p(px(3.0)).bg(theme::bg_sunken()).rounded(px(theme::RADIUS_MD)).border_1().border_color(theme::line())
}

fn text_seg(
    id: &'static str,
    label: &'static str,
    active: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    seg_base(id, active, on_click).child(div().text_size(px(12.0)).font_semibold().child(label))
}

fn icon_seg(
    id: &'static str,
    name: &'static str,
    active: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let fg = if active { theme::text_strong() } else { theme::text_muted() };
    seg_base(id, active, on_click).child(icon(name, 15.0, fg))
}

fn seg_base(
    id: &'static str,
    active: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> gpui::Stateful<gpui::Div> {
    let mut b = div()
        .id(id)
        .h(px(26.0))
        .px(px(11.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(theme::RADIUS_SM))
        .cursor_pointer()
        .on_click(on_click);
    if active {
        b = b.bg(theme::bg_hover()).text_color(theme::text_strong());
    } else {
        b = b.text_color(theme::text_muted()).hover(|s| s.bg(theme::white(0.03)).text_color(theme::text_soft()));
    }
    b
}

fn icon_btn(
    id: &'static str,
    name: &'static str,
    enabled: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let fg = if enabled { theme::icon_color() } else { theme::text_faint() };
    div()
        .id(id)
        .size(px(30.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(theme::RADIUS_SM))
        .border_1()
        .border_color(theme::line())
        .text_color(fg)
        .cursor_pointer()
        .hover(|s| s.bg(theme::bg_hover()).text_color(theme::text_strong()))
        .child(icon(name, 15.0, fg))
        .on_click(on_click)
}

fn vsep() -> impl IntoElement {
    div().w(px(1.0)).h(px(20.0)).bg(theme::line())
}
