//! Right context panel. Shows one of six views depending on workspace state:
//! Review (diff), Inspector (single selection), Multi (many), Outline (idle),
//! Start (nothing built), or Working (compiling).

use gpui::{App, ClickEvent, Context, Hsla, SharedString, Window, div, prelude::*, px};
use gpui_component::{StyledExt, h_flex, v_flex};

use crate::app::StudioApp;
use crate::state::{Align, BlockType, ChipKind, Dir, ElKind, OUTLINE_GROUPS, RightMode, element, review_items};
use crate::theme;
use crate::ui::widgets::{Btn, icon};

const COLOR_SWATCHES: &[(&str, u32)] = &[("#F4F6FB", 0xF4F6FB), ("#93C0F2", 0x93C0F2), ("#8A6DF2", 0x8A6DF2), ("#5CCB9A", 0x5CCB9A), ("#E9BE6A", 0xE9BE6A), ("#A3AAB8", 0xA3AAB8)];
const BG_SWATCHES: &[(&str, Option<u32>)] = &[("transparent", None), ("#14161C", Some(0x14161C)), ("#1B1E25", Some(0x1B1E25)), ("#93C0F2", Some(0x93C0F2)), ("#8A6DF2", Some(0x8A6DF2))];

pub fn render(app: &StudioApp, _window: &mut Window, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let panel = v_flex().flex_none().h_full().w(px(312.0)).min_h_0().bg(theme::bg_panel()).border_l_1().border_color(theme::line());
    match app.right_mode() {
        RightMode::Review => panel.child(review(app, cx)),
        RightMode::Inspector => panel.child(inspector(app, cx)),
        RightMode::Multi => panel.child(multi(app, cx)),
        RightMode::Outline => panel.child(outline(app, cx)),
        RightMode::Working => panel.child(working(app)),
        RightMode::Start => panel.child(start()),
    }
}

fn head(ic: &'static str, title: SharedString) -> gpui::Div {
    h_flex()
        .flex_none()
        .h(px(48.0))
        .items_center()
        .gap(px(9.0))
        .px(px(16.0))
        .border_b_1()
        .border_color(theme::line_faint())
        .child(icon(ic, 16.0, theme::accent()))
        .child(div().flex_1().font_semibold().text_size(px(13.5)).text_color(theme::text_strong()).child(title))
}

fn close_x(id: &'static str, on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> impl IntoElement {
    div()
        .id(id)
        .size(px(26.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(7.0))
        .text_color(theme::icon_color())
        .cursor_pointer()
        .hover(|s| s.bg(theme::bg_hover()).text_color(theme::text_strong()))
        .child(icon("close", 15.0, theme::icon_color()))
        .on_click(on_click)
}

fn section_label(text: &'static str) -> impl IntoElement {
    div().text_size(px(11.5)).font_bold().text_color(theme::text_caption()).mb(px(10.0)).child(text)
}

// ── Review ───────────────────────────────────────────────────────────────────
fn review(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let rtl = app.dir == Dir::Rtl;
    let items = review_items(rtl);
    let kept = app.kept_count();
    v_flex()
        .flex_1()
        .min_h_0()
        .child(head("eye", "Review changes".into()).child(div().text_size(px(11.5)).text_color(theme::text_caption()).child(format!("{kept} kept"))))
        .child(
            v_flex()
                .flex_1()
                .min_h_0()
                .id("rv-body")
                .overflow_y_scroll()
                .px(px(16.0))
                .py(px(14.0))
                .child(div().text_size(px(12.5)).text_color(theme::text_muted()).line_height(px(19.0)).mb(px(14.0)).child("Drag the divider on the preview to compare. Keep or drop each change, then apply."))
                .child(
                    h_flex()
                        .gap(px(8.0))
                        .mb(px(12.0))
                        .child(mini_btn("keep-all", "Keep all", cx.listener(|a, _, _, cx| a.review_keep_all(cx))))
                        .child(mini_btn("clear-all", "Clear all", cx.listener(|a, _, _, cx| a.review_clear_all(cx)))),
                )
                .child(v_flex().gap(px(8.0)).children(items.into_iter().enumerate().map(|(i, (kind, label))| review_row(i, kind, label, app.keeps[i], cx)))),
        )
        .child(
            h_flex()
                .w_full()
                .flex_none()
                .gap(px(10.0))
                .p(px(16.0))
                .border_t_1()
                .border_color(theme::line_faint())
                .child(Btn::secondary("Discard").grow().render("rv-discard", cx.listener(|a, _, _, cx| a.discard_review(cx))))
                .child(Btn::primary(format!("Apply {kept}")).grow().icon("check").render("rv-apply", cx.listener(|a, _, _, cx| a.apply_review(cx)))),
        )
}

fn review_row(i: usize, kind: ChipKind, label: &'static str, on: bool, cx: &mut Context<StudioApp>) -> impl IntoElement + use<> {
    let (tag_fg, tag_bg) = theme::chip_kind(kind);
    h_flex()
        .id(("rvrow", i))
        .items_center()
        .gap(px(10.0))
        .px(px(12.0))
        .py(px(11.0))
        .rounded(px(theme::RADIUS_MD))
        .border_1()
        .border_color(if on { theme::accent_ring() } else { theme::line() })
        .bg(if on { theme::accent_tint() } else { theme::bg_raised() })
        .cursor_pointer()
        .on_click(cx.listener(move |a, _, _, cx| a.toggle_keep(i, cx)))
        .child(check_box(on))
        .child(div().flex_1().text_size(px(12.5)).text_color(theme::text_soft()).child(label))
        .child(div().px(px(7.0)).py(px(3.0)).rounded(px(theme::RADIUS_XS)).bg(tag_bg).text_color(tag_fg).text_size(px(9.5)).font_bold().child(kind.label().to_uppercase()))
}

// ── Inspector (single selection) ─────────────────────────────────────────────
fn inspector(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let key = app.selection[0].clone();
    let kind = app.sel_kind().unwrap_or(ElKind::Text);
    let edit = app.edit_for(&key);
    let is_text = kind == ElKind::Text;
    let is_btn = kind == ElKind::Button;
    let is_img = kind == ElKind::Image;

    let mut body = v_flex()
        .flex_1()
        .min_h_0()
        .id("insp-body")
        .overflow_y_scroll()
        .p(px(16.0))
        .gap(px(20.0))
        .child(div().text_size(px(12.0)).text_color(theme::text_caption()).line_height(px(18.0)).child("Every control edits the live site instantly \u{2014} no prompt, no review step."));

    if is_text || is_btn {
        body = body.child(color_control(&edit, cx));
    }
    if is_text {
        let cur = edit.size.unwrap_or(default_size(&key));
        let min = if key.as_ref() == "heading" { 24.0 } else { 10.0 };
        body = body
            .child(stepper_control("Text size", cur, min, 72.0, 2.0, |a, v, cx| a.set_size(v, cx), cx))
            .child(weight_control(&edit, &key, cx))
            .child(align_control(&edit, cx));
    }
    if is_btn || is_img {
        body = body.child(bg_control(&edit, cx));
        let cur = edit.radius.unwrap_or(if is_btn { 40.0 } else { 18.0 });
        body = body.child(stepper_control("Corner radius", cur, 0.0, 40.0, 2.0, |a, v, cx| a.set_radius(v, cx), cx));
    }
    body = body.child(reset_btn(cx));

    v_flex()
        .flex_1()
        .min_h_0()
        .child(head("sliders", app.sel_label(&key)).child(close_x("insp-close", cx.listener(|a, _, _, cx| a.deselect(cx)))))
        .child(body)
}

// ── Multi-select ─────────────────────────────────────────────────────────────
fn multi(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let edit = app.edit_for(app.selection.first().map(|s| s.as_ref()).unwrap_or(""));
    v_flex()
        .flex_1()
        .min_h_0()
        .child(head("sliders", format!("{} elements selected", app.selection.len()).into()).child(close_x("multi-close", cx.listener(|a, _, _, cx| a.deselect(cx)))))
        .child(
            v_flex()
                .flex_1()
                .min_h_0()
                .id("multi-body")
                .overflow_y_scroll()
                .p(px(16.0))
                .gap(px(20.0))
                .child(div().text_size(px(12.0)).text_color(theme::text_caption()).line_height(px(18.0)).child("Changes apply to every selected element at once. Shift-click the preview to add or remove."))
                .child(color_control(&edit, cx))
                .child(align_control(&edit, cx))
                .child(reset_btn(cx)),
        )
}

// ── Outline (idle, built) ────────────────────────────────────────────────────
fn outline(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    v_flex()
        .flex_1()
        .min_h_0()
        .child(head("layers", "Page outline".into()).child(
            div()
                .id("outline-collapse")
                .size(px(26.0))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(7.0))
                .text_color(theme::icon_color())
                .cursor_pointer()
                .hover(|s| s.bg(theme::bg_hover()).text_color(theme::text_strong()))
                .child(icon("panel-close", 16.0, theme::icon_color()))
                .on_click(cx.listener(|a, _, _, cx| a.toggle_panel(cx))),
        ))
        .child(
            v_flex()
                .flex_1()
                .min_h_0()
                .id("outline-body")
                .overflow_y_scroll()
                .px(px(16.0))
                .py(px(14.0))
                .child(div().text_size(px(12.0)).text_color(theme::text_caption()).line_height(px(18.0)).mb(px(12.0)).child("Click any element to edit it. Shift-click to select several at once."))
                .child(v_flex().gap(px(2.0)).children(OUTLINE_GROUPS.iter().map(|(id, label, keys)| outline_group(app, id, label, keys, cx))))
                .child(
                    v_flex()
                        .mt(px(16.0))
                        .pt(px(14.0))
                        .border_t_1()
                        .border_color(theme::line_faint())
                        .child(section_label("ADD TO PAGE"))
                        .child(
                            h_flex()
                                .gap(px(8.0))
                                .child(add_btn("add-text", "type", "Text", BlockType::Text, cx))
                                .child(add_btn("add-image", "image", "Image", BlockType::Image, cx))
                                .child(add_btn("add-button", "plus", "Button", BlockType::Button, cx)),
                        ),
                ),
        )
}

fn outline_group(app: &StudioApp, id: &'static str, label: &'static str, keys: &'static [&'static str], cx: &mut Context<StudioApp>) -> impl IntoElement + use<> {
    let open = app.outline_open(id);
    let mut group = v_flex().child(
        h_flex()
            .id(SharedString::from(format!("olg-{id}")))
            .items_center()
            .gap(px(8.0))
            .px(px(10.0))
            .py(px(9.0))
            .rounded(px(theme::RADIUS_MD))
            .text_color(theme::text_soft())
            .cursor_pointer()
            .hover(|s| s.bg(theme::bg_raised()).text_color(theme::text_strong()))
            .child(icon(if open { "chevron-down" } else { "chevron-right" }, 14.0, theme::text_caption()))
            .child(div().flex_1().text_size(px(12.5)).font_semibold().child(label))
            .child(div().text_size(px(10.5)).text_color(theme::text_faint()).child(format!("{}", keys.len())))
            .on_click(cx.listener(move |a, _, _, cx| a.toggle_outline_group(id, cx))),
    );
    if open {
        group = group.child(v_flex().children(keys.iter().map(|k| {
            let key = *k;
            let meta = element(key);
            let (ic, lbl) = meta.map(|m| (m.icon, m.label)).unwrap_or(("target", key));
            h_flex()
                .id(SharedString::from(format!("olf-{key}")))
                .items_center()
                .gap(px(10.0))
                .pl(px(32.0))
                .pr(px(12.0))
                .py(px(8.0))
                .rounded(px(theme::RADIUS_MD))
                .text_size(px(12.5))
                .text_color(theme::text_muted())
                .cursor_pointer()
                .hover(|s| s.bg(theme::bg_raised()).text_color(theme::text_strong()))
                .child(icon(ic, 14.0, theme::text_caption()))
                .child(lbl)
                .on_click(cx.listener(move |a, ev: &gpui::ClickEvent, _, cx| a.select_el(key, ev.modifiers().shift, cx)))
        })));
    }
    group
}

fn add_btn(id: &'static str, ic: &'static str, label: &'static str, kind: BlockType, cx: &mut Context<StudioApp>) -> impl IntoElement {
    v_flex()
        .id(id)
        .flex_1()
        .h(px(60.0))
        .items_center()
        .justify_center()
        .gap(px(6.0))
        .rounded(px(theme::RADIUS_MD))
        .border_1()
        .border_dashed()
        .border_color(theme::line_bright())
        .text_size(px(11.5))
        .font_semibold()
        .text_color(theme::text_muted())
        .cursor_pointer()
        .hover(|s| s.border_color(theme::accent_ring()).text_color(theme::text_strong()))
        .child(icon(ic, 16.0, theme::text_muted()))
        .child(label)
        .on_click(cx.listener(move |a, _, _, cx| a.add_block(kind, cx)))
}

// ── Start / Working ──────────────────────────────────────────────────────────
fn start() -> impl IntoElement {
    v_flex()
        .flex_1()
        .items_center()
        .justify_center()
        .p(px(32.0))
        .gap(px(12.0))
        .child(div().size(px(44.0)).rounded(px(12.0)).bg(theme::bg_raised()).flex().items_center().justify_center().child(icon("layers", 20.0, theme::text_caption())))
        .child(div().text_size(px(13.5)).font_semibold().text_color(theme::text_soft()).child("Nothing to inspect yet"))
        .child(div().text_size(px(12.5)).text_color(theme::text_caption()).text_center().line_height(px(19.0)).child("Once you build a page, review changes and the inspector show up here."))
}

fn working(app: &StudioApp) -> impl IntoElement {
    v_flex()
        .flex_1()
        .items_center()
        .justify_center()
        .p(px(32.0))
        .gap(px(12.0))
        .child(icon("loader", 30.0, theme::accent()))
        .child(div().text_size(px(13.5)).font_semibold().text_color(theme::text_soft()).child("Building your page\u{2026}"))
        .child(div().text_size(px(12.5)).text_color(theme::text_caption()).text_center().line_height(px(19.0)).child(SharedString::from(app.compile_text())))
}

// ── controls ─────────────────────────────────────────────────────────────────
fn color_control(edit: &crate::state::ElEdit, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let current = edit.color.clone();
    v_flex().child(section_label("TEXT COLOR")).child(h_flex().flex_wrap().gap(px(8.0)).children(COLOR_SWATCHES.iter().map(|(val, rgb)| {
        let value: SharedString = (*val).into();
        let on = current.as_ref().map(|c| c.as_ref() == *val).unwrap_or(false);
        swatch(val, theme::hex(*rgb), false, on, cx.listener(move |a, _, _, cx| a.set_color(value.clone(), cx)))
    })))
}

fn bg_control(edit: &crate::state::ElEdit, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let current = edit.bg.clone();
    v_flex().child(section_label("BACKGROUND")).child(h_flex().flex_wrap().gap(px(8.0)).children(BG_SWATCHES.iter().map(|(val, rgb)| {
        let value: SharedString = (*val).into();
        let on = current.as_ref().map(|c| c.as_ref() == *val).unwrap_or(false);
        let (color, transparent) = match rgb {
            Some(v) => (theme::hex(*v), false),
            None => (theme::bg_sunken(), true),
        };
        swatch(val, color, transparent, on, cx.listener(move |a, _, _, cx| a.set_bg(value.clone(), cx)))
    })))
}

fn swatch(id: &'static str, color: Hsla, transparent: bool, active: bool, on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> impl IntoElement {
    let mut b = div().id(id).size(px(26.0)).rounded(px(theme::RADIUS_XS)).bg(color).cursor_pointer().border_1().border_color(if active { theme::accent() } else { theme::line_bright() }).on_click(on_click);
    if active {
        b = b.border_2().border_color(theme::accent());
    }
    if transparent {
        b = b.flex().items_center().justify_center().child(icon("close", 14.0, theme::text_caption()));
    }
    b
}

fn weight_control(edit: &crate::state::ElEdit, key: &str, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let cur = edit.weight.unwrap_or(default_weight(key));
    v_flex().child(section_label("WEIGHT")).child(
        h_flex()
            .gap(px(4.0))
            .p(px(3.0))
            .rounded(px(theme::RADIUS_MD))
            .bg(theme::bg_sunken())
            .border_1()
            .border_color(theme::line())
            .child(seg_btn("w-reg", "Regular", cur == 400, cx.listener(|a, _, _, cx| a.set_weight(400, cx))))
            .child(seg_btn("w-med", "Medium", cur == 600, cx.listener(|a, _, _, cx| a.set_weight(600, cx))))
            .child(seg_btn("w-bold", "Bold", cur == 800, cx.listener(|a, _, _, cx| a.set_weight(800, cx)))),
    )
}

fn align_control(edit: &crate::state::ElEdit, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let cur = edit.align.unwrap_or(Align::Start);
    v_flex().child(section_label("ALIGNMENT")).child(
        h_flex()
            .gap(px(4.0))
            .p(px(3.0))
            .rounded(px(theme::RADIUS_MD))
            .bg(theme::bg_sunken())
            .border_1()
            .border_color(theme::line())
            .child(seg_btn("a-start", "Start", cur == Align::Start, cx.listener(|a, _, _, cx| a.set_align(Align::Start, cx))))
            .child(seg_btn("a-center", "Center", cur == Align::Center, cx.listener(|a, _, _, cx| a.set_align(Align::Center, cx))))
            .child(seg_btn("a-end", "End", cur == Align::End, cx.listener(|a, _, _, cx| a.set_align(Align::End, cx)))),
    )
}

fn seg_btn(id: &'static str, label: &'static str, active: bool, on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> impl IntoElement {
    let mut b = div().id(id).flex_1().h(px(28.0)).flex().items_center().justify_center().rounded(px(theme::RADIUS_SM)).text_size(px(12.0)).font_semibold().cursor_pointer().on_click(on_click);
    if active {
        b = b.bg(theme::bg_hover()).text_color(theme::text_strong());
    } else {
        b = b.text_color(theme::text_muted());
    }
    b.child(label)
}

/// A labeled value with − / + steppers and a filled progress track (a
/// functional stand-in for a drag slider).
fn stepper_control(
    label: &'static str,
    value: f32,
    min: f32,
    max: f32,
    step: f32,
    set: impl Fn(&mut StudioApp, f32, &mut Context<StudioApp>) + Copy + 'static,
    cx: &mut Context<StudioApp>,
) -> impl IntoElement {
    let frac = ((value - min) / (max - min)).clamp(0.0, 1.0);
    let dec = (value - step).max(min);
    let inc = (value + step).min(max);
    v_flex()
        .gap(px(8.0))
        .child(
            h_flex()
                .items_center()
                .child(div().flex_1().text_size(px(11.5)).font_bold().text_color(theme::text_caption()).child(label.to_uppercase()))
                .child(div().text_size(px(12.5)).font_semibold().text_color(theme::text_strong()).child(format!("{}px", value as i32))),
        )
        .child(
            h_flex()
                .items_center()
                .gap(px(8.0))
                .child(stepper_btn("st-dec", "minus", cx.listener(move |a, _, _, cx| set(a, dec, cx))))
                .child(
                    div()
                        .flex_1()
                        .h(px(6.0))
                        .rounded_full()
                        .bg(theme::bg_hover())
                        .child(div().h_full().w(gpui::relative(frac)).rounded_full().bg(theme::accent_grad())),
                )
                .child(stepper_btn("st-inc", "plus", cx.listener(move |a, _, _, cx| set(a, inc, cx)))),
        )
}

fn stepper_btn(id: &'static str, ic: &'static str, on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> impl IntoElement {
    div()
        .id(id)
        .size(px(26.0))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(theme::RADIUS_XS))
        .bg(theme::bg_raised())
        .text_color(theme::text_soft())
        .cursor_pointer()
        .hover(|s| s.text_color(theme::text_strong()))
        .child(icon(ic, 14.0, theme::text_soft()))
        .on_click(on_click)
}

fn reset_btn(cx: &mut Context<StudioApp>) -> impl IntoElement {
    h_flex()
        .id("reset-style")
        .mt(px(4.0))
        .h(px(34.0))
        .items_center()
        .justify_center()
        .gap(px(8.0))
        .rounded(px(theme::RADIUS_SM))
        .border_1()
        .border_color(theme::line())
        .text_size(px(12.5))
        .font_semibold()
        .text_color(theme::text_muted())
        .cursor_pointer()
        .hover(|s| s.bg(theme::bg_hover()).text_color(theme::text_strong()))
        .child(icon("reset", 14.0, theme::text_muted()))
        .child("Reset styling")
        .on_click(cx.listener(|a, _, _, cx| a.reset_style(cx)))
}

fn mini_btn(id: &'static str, label: &'static str, on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> impl IntoElement {
    div()
        .id(id)
        .flex_1()
        .h(px(30.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(theme::RADIUS_SM))
        .border_1()
        .border_color(theme::line())
        .bg(theme::bg_raised())
        .text_size(px(12.0))
        .font_semibold()
        .text_color(theme::text_soft())
        .cursor_pointer()
        .hover(|s| s.text_color(theme::text_strong()))
        .child(label)
        .on_click(on_click)
}

fn check_box(on: bool) -> impl IntoElement {
    let mut b = div().size(px(18.0)).flex_none().flex().items_center().justify_center().rounded(px(theme::RADIUS_XS)).border_1();
    if on {
        b = b.bg(theme::accent()).border_color(theme::accent()).child(icon("check", 12.0, theme::accent_contrast()));
    } else {
        b = b.border_color(theme::line_bright());
    }
    b
}

fn default_size(key: &str) -> f32 {
    match key {
        "heading" => 44.0,
        "sub" => 17.0,
        "nav" => 14.0,
        "eyebrow" => 12.0,
        "brand" => 18.0,
        "lineupTitle" => 16.0,
        "footer" => 13.0,
        k if k.starts_with("add") => 16.0,
        _ => 20.0,
    }
}
fn default_weight(key: &str) -> u16 {
    match key {
        "heading" => 800,
        "sub" => 400,
        _ => 600,
    }
}
