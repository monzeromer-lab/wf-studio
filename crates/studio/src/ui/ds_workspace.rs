//! Design-system workspace: an assistant chat, a specimen canvas with three
//! tabs (Foundations / Components / Preview), and a context-sensitive inspector.
//! A native GPUI port of the `dsWorkspace` screen in `WebFluent Studio.dc.html`.

use gpui::{AnyElement, App, ClickEvent, Context, ElementId, SharedString, Window, div, prelude::*, px, relative};
use gpui_component::{StyledExt, h_flex, input::Input, v_flex};

use crate::app::StudioApp;
use crate::state::*;
use crate::theme;
use crate::ui::widgets::{dot, icon};

const DS_SUGGESTIONS: &[&str] = &[
    "Shift the accent to a warmer coral",
    "Add a large display type token",
    "Generate a pricing-table component",
];

pub fn render(app: &StudioApp, _window: &mut Window, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let left = if app.chat_open { ds_chat(app, cx).into_any_element() } else { chat_tab(cx).into_any_element() };
    h_flex().flex_1().min_h_0().w_full().child(left).child(canvas(app, cx)).child(inspector(app, cx))
}

// ── left: assistant chat ────────────────────────────────────────────────────
fn chat_tab(cx: &mut Context<StudioApp>) -> impl IntoElement {
    v_flex()
        .id("ds-chat-tab")
        .w(px(40.0))
        .flex_none()
        .h_full()
        .items_center()
        .pt(px(14.0))
        .bg(theme::bg_base())
        .border_r_1()
        .border_color(theme::line())
        .cursor_pointer()
        .hover(|s| s.bg(theme::bg_hover()))
        .child(icon("sparkle", 18.0, theme::accent()))
        .on_click(cx.listener(|a, _, _, cx| a.toggle_chat(cx)))
}

fn ds_chat(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    v_flex()
        .flex_none()
        .h_full()
        .w(px(360.0))
        .min_h_0()
        .bg(theme::bg_base())
        .border_r_1()
        .border_color(theme::line())
        .child(
            h_flex()
                .flex_none()
                .h(px(48.0))
                .items_center()
                .gap(px(9.0))
                .px(px(16.0))
                .border_b_1()
                .border_color(theme::line_faint())
                .child(div().size(px(24.0)).rounded(px(7.0)).bg(theme::accent_grad()).flex().items_center().justify_center().child(icon("sparkle", 14.0, theme::accent_contrast())))
                .child(div().font_semibold().text_size(px(13.5)).text_color(theme::text_strong()).child("Assistant"))
                .child(div().flex_1())
                .child(h_flex().items_center().gap(px(6.0)).text_size(px(12.0)).text_color(theme::text_muted()).child(dot(6.0, theme::success())).child(app.provider().name))
                .child(
                    div()
                        .id("ds-collapse")
                        .size(px(26.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(7.0))
                        .text_color(theme::icon_color())
                        .cursor_pointer()
                        .hover(|s| s.bg(theme::bg_hover()).text_color(theme::text_strong()))
                        .child(icon("panel-close", 16.0, theme::icon_color()))
                        .on_click(cx.listener(|a, _, _, cx| a.toggle_chat(cx))),
                ),
        )
        .child(
            v_flex()
                .id("ds-messages")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .p(px(16.0))
                .gap(px(14.0))
                .children(app.messages.iter().enumerate().map(|(i, m)| message_row(i, m)))
                .child(v_flex().gap(px(8.0)).children(DS_SUGGESTIONS.iter().enumerate().map(|(i, s)| suggestion(i, s, cx)))),
        )
        .child(
            v_flex().flex_none().px(px(16.0)).pt(px(14.0)).pb(px(16.0)).child(
                v_flex()
                    .rounded(px(theme::RADIUS_XL))
                    .border_1()
                    .border_color(theme::line_strong())
                    .bg(theme::bg_sunken())
                    .px(px(14.0))
                    .py(px(12.0))
                    .gap(px(10.0))
                    .child(Input::new(&app.prompt).appearance(false).text_size(px(13.5)))
                    .child(
                        h_flex()
                            .items_center()
                            .gap(px(7.0))
                            .child(div().flex_1().text_size(px(11.5)).text_color(theme::text_caption()).child(SharedString::from(app.chat_model_label())))
                            .child(
                                div()
                                    .id("ds-send")
                                    .size(px(34.0))
                                    .flex_none()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_full()
                                    .bg(theme::accent())
                                    .text_color(theme::accent_contrast())
                                    .shadow(theme::glow_accent())
                                    .cursor_pointer()
                                    .hover(|s| s.bg(theme::accent_hover()))
                                    .child(icon("send", 16.0, theme::accent_contrast()))
                                    .on_click(cx.listener(|a, _, window, cx| a.ds_send(window, cx))),
                            ),
                    ),
            ),
        )
}

fn suggestion(i: usize, text: &'static str, cx: &mut Context<StudioApp>) -> impl IntoElement + use<> {
    h_flex()
        .id(("ds-sugg", i))
        .items_center()
        .gap(px(9.0))
        .p(px(11.0))
        .rounded(px(theme::RADIUS_MD))
        .border_1()
        .border_color(theme::line())
        .bg(theme::bg_panel())
        .text_size(px(13.0))
        .text_color(theme::text_soft())
        .cursor_pointer()
        .hover(|s| s.border_color(theme::accent_ring()).bg(theme::accent_tint()).text_color(theme::text_strong()))
        .child(icon("sparkle", 14.0, theme::accent()))
        .child(div().flex_1().child(text))
        .on_click(cx.listener(move |a, _, _, cx| a.ds_run(text, cx)))
}

fn message_row(i: usize, m: &Message) -> impl IntoElement + use<> {
    let user = m.role == Role::User;
    let mut bubble = div().id(("ds-msg", i)).max_w(px(282.0)).px(px(14.0)).py(px(11.0)).text_size(px(13.0)).line_height(px(20.0)).child(m.text.clone());
    if user {
        bubble = bubble.bg(theme::accent_grad_soft()).text_color(theme::white(1.0)).rounded(px(14.0)).rounded_br(px(4.0));
    } else {
        bubble = bubble.bg(theme::bg_panel()).border_1().border_color(theme::line()).text_color(theme::text_soft()).rounded(px(14.0)).rounded_bl(px(4.0));
    }
    h_flex().w_full().when(user, |r| r.justify_end()).child(bubble)
}

// ── center: specimen canvas ─────────────────────────────────────────────────
fn canvas(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let content = match app.ds_tab {
        DsTab::Foundations => foundations(app, cx).into_any_element(),
        DsTab::Components => components(app, cx).into_any_element(),
        DsTab::Preview => preview(app, cx).into_any_element(),
    };
    v_flex().flex_1().min_w_0().min_h_0().h_full().child(canvas_header(app, cx)).child(
        div()
            .id("ds-canvas")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .px(px(40.0))
            .py(px(30.0))
            .bg(theme::bg_sunken())
            .child(v_flex().max_w(px(860.0)).mx_auto().child(content)),
    )
}

fn canvas_header(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let name = app.current_project().map(|p| p.name.clone()).unwrap_or_else(|| "Studio DS".into());
    let mut tabs = h_flex().gap(px(3.0)).p(px(3.0)).rounded_full().bg(theme::bg_sunken()).border_1().border_color(theme::line());
    for (i, t) in DsTab::ALL.iter().enumerate() {
        let t = *t;
        let active = app.ds_tab == t;
        tabs = tabs.child(
            h_flex()
                .id(("ds-tab", i))
                .px(px(15.0))
                .py(px(6.0))
                .rounded_full()
                .cursor_pointer()
                .text_size(px(12.5))
                .font_semibold()
                .when(active, |e| e.bg(theme::accent()).text_color(theme::accent_contrast()))
                .when(!active, |e| e.text_color(theme::text_muted()).hover(|s| s.text_color(theme::text_strong())))
                .child(t.label())
                .on_click(cx.listener(move |a, _, _, cx| a.set_ds_tab(t, cx))),
        );
    }
    h_flex()
        .flex_none()
        .h(px(54.0))
        .items_center()
        .gap(px(14.0))
        .px(px(24.0))
        .border_b_1()
        .border_color(theme::line_faint())
        .bg(theme::bg_base())
        .child(div().size(px(30.0)).rounded(px(9.0)).bg(theme::accent_grad()).flex().items_center().justify_center().flex_none().child(icon("boxes", 16.0, theme::accent_contrast())))
        .child(
            v_flex()
                .min_w_0()
                .child(div().font_family(theme::FONT_DISPLAY).text_size(px(15.0)).font_semibold().text_color(theme::text_strong()).child(name))
                .child(div().text_size(px(11.5)).text_color(theme::text_dim()).child("Design system")),
        )
        .child(div().flex_1())
        .child(tabs)
}

fn kicker(text: impl Into<SharedString>) -> gpui::Div {
    div().text_size(px(11.5)).font_bold().text_color(theme::text_dim()).child(text.into())
}

// ── Foundations tab ─────────────────────────────────────────────────────────
fn foundations(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let mut col = v_flex();
    // colors
    col = col.child(
        h_flex()
            .items_center()
            .justify_between()
            .mb(px(14.0))
            .child(kicker(format!("COLOR \u{b7} {} TOKENS", app.ds_colors.len())))
            .child(add_token_btn(cx)),
    );
    for (gname, idxs) in ds_color_groups(&app.ds_colors) {
        col = col.child(div().mt(px(14.0)).mb(px(9.0)).text_size(px(11.0)).font_semibold().text_color(theme::text_faint()).child(gname));
        let mut grid = h_flex().flex_wrap().gap(px(11.0));
        for i in idxs {
            grid = grid.child(color_card(app, i, cx));
        }
        col = col.child(grid);
    }
    // type
    col = col.child(kicker(format!("TYPE SCALE \u{b7} {} STYLES", app.ds_types.len())).mt(px(30.0)).mb(px(12.0)));
    let mut types = v_flex().gap(px(9.0));
    for i in 0..app.ds_types.len() {
        types = types.child(type_row(app, i, cx));
    }
    col = col.child(types);
    // radii
    col = col.child(kicker("RADII").mt(px(30.0)).mb(px(12.0)));
    col = col.child(
        h_flex().flex_wrap().gap(px(16.0)).children(DS_RADII.iter().map(|r| {
            v_flex()
                .items_center()
                .child(div().size(px(46.0)).rounded(px(r.val)).bg(theme::bg_raised()).border_1().border_color(theme::line_bright()))
                .child(div().mt(px(8.0)).text_size(px(11.5)).font_semibold().text_color(theme::text_soft()).child(r.name))
                .child(div().text_size(px(10.5)).font_family(theme::FONT_MONO).text_color(theme::text_dim()).child(format!("{}px", r.val as i32)))
        })),
    );
    col.child(div().h(px(20.0)))
}

fn add_token_btn(cx: &mut Context<StudioApp>) -> impl IntoElement {
    h_flex()
        .id("ds-addtoken")
        .items_center()
        .gap(px(6.0))
        .px(px(12.0))
        .py(px(6.0))
        .rounded_full()
        .border_1()
        .border_dashed()
        .border_color(theme::line_bright())
        .text_size(px(12.0))
        .font_semibold()
        .text_color(theme::text_muted())
        .cursor_pointer()
        .hover(|s| s.border_color(theme::accent_ring()).text_color(theme::text_strong()))
        .child(icon("plus", 14.0, theme::text_muted()))
        .child("Add token")
        .on_click(cx.listener(|a, _, _, cx| a.ds_add_color(cx)))
}

fn color_card(app: &StudioApp, i: usize, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let c = &app.ds_colors[i];
    let selected = app.ds_sel.as_ref().is_some_and(|s| s.kind == DsSelKind::Color && s.id == c.id);
    let id = c.id.clone();
    v_flex()
        .id(("ds-color", i))
        .w(px(158.0))
        .rounded(px(theme::RADIUS_MD))
        .overflow_hidden()
        .cursor_pointer()
        .border_1()
        .border_color(if selected { theme::accent_ring() } else { theme::line() })
        .bg(theme::bg_panel())
        .hover(|s| s.border_color(theme::accent_ring()))
        .child(div().h(px(58.0)).w_full().rounded_tl(px(theme::RADIUS_MD)).rounded_tr(px(theme::RADIUS_MD)).bg(theme::hex(c.val)))
        .child(
            v_flex()
                .px(px(12.0))
                .py(px(9.0))
                .child(div().text_size(px(12.5)).font_semibold().text_color(theme::text_strong()).child(c.name.clone()))
                .child(div().text_size(px(11.0)).text_color(theme::text_dim()).child(c.role.clone()))
                .child(div().mt(px(4.0)).font_family(theme::FONT_MONO).text_size(px(10.5)).text_color(theme::text_faint()).child(format!("#{:06X}", c.val))),
        )
        .on_click(cx.listener(move |a, _, _, cx| a.ds_select(DsSelKind::Color, id.clone(), cx)))
}

fn type_row(app: &StudioApp, i: usize, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let t = &app.ds_types[i];
    let selected = app.ds_sel.as_ref().is_some_and(|s| s.kind == DsSelKind::Type && s.id == t.id);
    let id = t.id.clone();
    h_flex()
        .id(("ds-type", i))
        .items_center()
        .gap(px(16.0))
        .px(px(18.0))
        .py(px(16.0))
        .rounded(px(theme::RADIUS_MD))
        .cursor_pointer()
        .border_1()
        .border_color(if selected { theme::accent_ring() } else { theme::line() })
        .bg(if selected { theme::accent_tint() } else { theme::bg_panel() })
        .hover(|s| s.border_color(theme::accent_ring()))
        .child(div().flex_1().min_w_0().text_size(px(t.size.min(40.0))).font_weight(gpui::FontWeight(t.weight as f32)).font_family(t.font()).text_color(theme::text_strong()).child(t.sample.clone()))
        .child(
            v_flex()
                .items_end()
                .flex_none()
                .child(div().text_size(px(12.5)).font_semibold().text_color(theme::text_soft()).child(t.name.clone()))
                .child(div().mt(px(2.0)).text_size(px(11.0)).font_family(theme::FONT_MONO).text_color(theme::text_dim()).child(t.meta_text())),
        )
        .on_click(cx.listener(move |a, _, _, cx| a.ds_select(DsSelKind::Type, id.clone(), cx)))
}

// ── Components tab ──────────────────────────────────────────────────────────
fn components(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let (ready, total) = ds_comp_counts();
    let mut col = v_flex();
    col = col.child(kicker(format!("COMPONENTS \u{b7} {ready} READY \u{b7} {total} IN CATALOG")).mb(px(4.0)));
    col = col.child(
        div()
            .text_size(px(12.5))
            .text_color(theme::text_muted())
            .line_height(px(19.0))
            .mb(px(18.0))
            .child("A full catalog across every category. Ready components render live from your system; the rest can be generated \u{2014} ask the assistant or click one."),
    );
    for cat in DS_CATALOG {
        col = col.child(cat_header(cat));
        let mut grid = h_flex().flex_wrap().gap(px(12.0));
        for c in cat.items {
            grid = grid.child(comp_card(app, *c, cx));
        }
        col = col.child(grid);
    }
    col.child(div().h(px(20.0)))
}

fn cat_header(cat: &DsCompCat) -> impl IntoElement {
    h_flex()
        .items_center()
        .gap(px(10.0))
        .mt(px(20.0))
        .mb(px(11.0))
        .child(div().text_size(px(12.0)).font_bold().text_color(theme::text_soft()).child(cat.cat))
        .child(div().text_size(px(11.0)).font_family(theme::FONT_MONO).text_color(theme::text_faint()).child(format!("{}", cat.items.len())))
        .child(div().flex_1().h(px(1.0)).bg(theme::line_faint()))
}

fn comp_card(app: &StudioApp, c: DsComp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let ready = c.ready();
    let selected = app.ds_sel.as_ref().is_some_and(|s| s.kind == DsSelKind::Comp && s.id.as_ref() == c.id);
    let (status_label, status_color) = if ready { ("Live", theme::success()) } else { ("Plan", theme::text_dim()) };
    v_flex()
        .id(c.id)
        .w(px(232.0))
        .min_h(px(118.0))
        .p(px(15.0))
        .rounded(px(theme::RADIUS_LG))
        .cursor_pointer()
        .border_1()
        .border_color(if selected { theme::accent_ring() } else { theme::line() })
        .bg(if ready { theme::bg_panel() } else { theme::bg_raised() })
        .hover(|s| s.border_color(theme::accent_ring()))
        .child(
            h_flex()
                .items_center()
                .gap(px(8.0))
                .mb(px(12.0))
                .child(div().flex_1().min_w_0().text_size(px(12.5)).font_semibold().text_color(theme::text_strong()).child(c.label))
                .child(h_flex().items_center().gap(px(5.0)).text_size(px(10.0)).font_semibold().text_color(status_color).child(dot(5.0, status_color)).child(status_label)),
        )
        .child(h_flex().flex_1().items_center().overflow_hidden().child(specimen(c.kind, &app.ds_demo, false, cx)))
        .on_click(cx.listener(move |a, _, _, cx| a.ds_select(DsSelKind::Comp, c.id, cx)))
}

// ── live specimens ──────────────────────────────────────────────────────────
fn specimen(kind: DsCompKind, demo: &DsDemo, interactive: bool, cx: &mut Context<StudioApp>) -> AnyElement {
    match kind {
        DsCompKind::Button => button_view(demo).into_any_element(),
        DsCompKind::IconButton => h_flex().gap(px(8.0)).child(icon_btn("plus")).child(icon_btn("share")).into_any_element(),
        DsCompKind::Input => input_view(demo.input_ph.clone(), 36.0).into_any_element(),
        DsCompKind::Textarea => input_view("Message\u{2026}".into(), 52.0).into_any_element(),
        DsCompKind::Chip => chip_view(demo.chip_kind, demo.chip_label.clone()).into_any_element(),
        DsCompKind::Toggle => toggle_row(demo, interactive, cx),
        DsCompKind::Slider => slider_track(demo.slider).into_any_element(),
        DsCompKind::Status => status_view(demo.status_tone).into_any_element(),
        DsCompKind::Avatar => avatar_row(demo).into_any_element(),
        DsCompKind::Card => card_view().into_any_element(),
        DsCompKind::Seg => seg_view(demo.tabs_active, interactive, cx),
        other => schematic(other).into_any_element(),
    }
}

fn button_view(demo: &DsDemo) -> impl IntoElement {
    let (bg, fg, border) = demo.button_variant.style();
    let (h, pad, ts) = demo.button_size.metrics();
    let mut b = h_flex()
        .h(px(h))
        .px(px(pad))
        .items_center()
        .justify_center()
        .rounded(px(theme::RADIUS_SM))
        .text_size(px(ts))
        .font_semibold()
        .text_color(fg)
        .flex_none()
        .child(demo.button_label.clone());
    if let Some(bg) = bg {
        b = b.bg(bg);
    }
    if let Some(bc) = border {
        b = b.border_1().border_color(bc);
    }
    b
}

fn icon_btn(name: &'static str) -> impl IntoElement {
    div()
        .size(px(34.0))
        .rounded(px(theme::RADIUS_SM))
        .bg(theme::bg_raised())
        .border_1()
        .border_color(theme::line())
        .flex()
        .items_center()
        .justify_center()
        .flex_none()
        .child(icon(name, 15.0, theme::text_soft()))
}

fn input_view(ph: SharedString, height: f32) -> impl IntoElement {
    h_flex()
        .w_full()
        .h(px(height))
        .items_center()
        .px(px(12.0))
        .rounded(px(theme::RADIUS_SM))
        .bg(theme::bg_sunken())
        .border_1()
        .border_color(theme::line_strong())
        .child(div().text_size(px(13.0)).text_color(theme::text_faint()).child(ph))
}

fn chip_view(kind: DsChipKind, label: SharedString) -> impl IntoElement {
    let (fg, bg) = kind.colors();
    h_flex().items_center().h(px(24.0)).px(px(10.0)).rounded_full().bg(bg).text_color(fg).text_size(px(11.5)).font_semibold().flex_none().child(label)
}

fn status_view(tone: DsStatusTone) -> impl IntoElement {
    let c = tone.color();
    h_flex()
        .items_center()
        .gap(px(6.0))
        .h(px(24.0))
        .px(px(10.0))
        .rounded_full()
        .bg(theme::bg_raised())
        .border_1()
        .border_color(theme::line())
        .text_size(px(11.5))
        .font_semibold()
        .text_color(c)
        .flex_none()
        .child(dot(5.0, c))
        .child(tone.label())
}

fn avatar_row(demo: &DsDemo) -> impl IntoElement {
    let (fg, bg) = demo.avatar_tone.colors();
    h_flex()
        .gap(px(7.0))
        .items_center()
        .child(
            h_flex()
                .size(px(34.0))
                .rounded_full()
                .items_center()
                .justify_center()
                .flex_none()
                .bg(bg)
                .text_color(fg)
                .text_size(px(13.0))
                .font_semibold()
                .font_family(theme::FONT_DISPLAY)
                .child(demo.avatar_initials.clone()),
        )
        .child(div().text_size(px(11.5)).text_color(theme::text_dim()).child(demo.avatar_initials.clone()))
}

fn switch(on: bool) -> gpui::Div {
    let mut t = h_flex()
        .w(px(44.0))
        .h(px(24.0))
        .rounded_full()
        .p(px(3.0))
        .items_center()
        .flex_none()
        .bg(if on { theme::accent() } else { theme::line_strong() })
        .child(div().size(px(18.0)).rounded_full().bg(theme::white(1.0)));
    if on {
        t = t.justify_end();
    }
    t
}

fn toggle_row(demo: &DsDemo, interactive: bool, cx: &mut Context<StudioApp>) -> AnyElement {
    let on = demo.toggle;
    let sw = if interactive {
        switch(on).id("ds-switch").cursor_pointer().on_click(cx.listener(|a, _, _, cx| a.toggle_ds_demo(cx))).into_any_element()
    } else {
        switch(on).into_any_element()
    };
    h_flex()
        .gap(px(12.0))
        .items_center()
        .child(sw)
        .child(div().text_size(px(11.5)).text_color(theme::text_dim()).child(if on { "On" } else { "Off" }))
        .into_any_element()
}

fn seg_view(active: u8, interactive: bool, cx: &mut Context<StudioApp>) -> AnyElement {
    let mut row = h_flex().gap(px(3.0)).p(px(3.0)).rounded_full().bg(theme::bg_sunken()).border_1().border_color(theme::line()).flex_none();
    for (i, lbl) in ["Preview", "Code", "Docs"].iter().enumerate() {
        let on = active as usize == i;
        let pill = div()
            .px(px(12.0))
            .py(px(5.0))
            .rounded_full()
            .text_size(px(11.0))
            .font_semibold()
            .when(on, |e| e.bg(theme::accent()).text_color(theme::accent_contrast()))
            .when(!on, |e| e.text_color(theme::text_muted()))
            .child(*lbl);
        if interactive {
            row = row.child(pill.id(("ds-seg", i)).cursor_pointer().on_click(cx.listener(move |a, _, _, cx| a.set_ds_tabs_active(i as u8, cx))));
        } else {
            row = row.child(pill);
        }
    }
    row.into_any_element()
}

fn card_view() -> impl IntoElement {
    v_flex()
        .w_full()
        .rounded(px(theme::RADIUS_MD))
        .border_1()
        .border_color(theme::line())
        .bg(theme::bg_raised())
        .p(px(11.0))
        .child(div().h(px(7.0)).w(px(90.0)).rounded(px(4.0)).bg(theme::accent()))
        .child(div().mt(px(8.0)).h(px(6.0)).w_full().rounded(px(4.0)).bg(theme::line_bright()))
        .child(div().mt(px(5.0)).h(px(6.0)).w(px(120.0)).rounded(px(4.0)).bg(theme::line_bright()))
}

fn slider_track(value: u8) -> impl IntoElement {
    div()
        .w_full()
        .h(px(6.0))
        .rounded_full()
        .bg(theme::bg_sunken())
        .overflow_hidden()
        .child(div().h_full().w(relative(value as f32 / 100.0)).bg(theme::accent_grad()))
}

fn schematic(_kind: DsCompKind) -> impl IntoElement {
    v_flex()
        .w_full()
        .gap(px(6.0))
        .child(div().h(px(10.0)).w(px(64.0)).rounded(px(4.0)).bg(theme::line_bright()))
        .child(div().h(px(9.0)).w_full().rounded(px(5.0)).bg(theme::bg_raised()).border_1().border_color(theme::line()))
        .child(div().h(px(9.0)).w(relative(0.7)).rounded(px(5.0)).bg(theme::bg_raised()).border_1().border_color(theme::line()))
}

// ── Preview tab ─────────────────────────────────────────────────────────────
fn preview(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let rtl = app.ds_rtl;
    let (eyebrow, head, body, cta, cta2, chips): (&str, &str, &str, &str, &str, [&str; 3]) = if rtl {
        (
            "\u{645}\u{628}\u{627}\u{634}\u{631} \u{627}\u{644}\u{644}\u{64a}\u{644}\u{629} \u{b7} \u{627}\u{644}\u{631}\u{64a}\u{627}\u{636}",
            "\u{644}\u{64a}\u{627}\u{644}\u{64d} \u{644}\u{627} \u{62a}\u{64f}\u{646}\u{633}\u{649} \u{641}\u{648}\u{642} \u{627}\u{644}\u{631}\u{64a}\u{627}\u{636}",
            "\u{645}\u{648}\u{633}\u{64a}\u{642}\u{649} \u{62d}\u{64a}\u{651}\u{629}\u{60c} \u{648}\u{639}\u{634}\u{627}\u{621} \u{62a}\u{62d}\u{62a} \u{627}\u{644}\u{646}\u{62c}\u{648}\u{645} \u{62d}\u{62a}\u{649} \u{627}\u{644}\u{641}\u{62c}\u{631}.",
            "\u{627}\u{62d}\u{62c}\u{632} \u{637}\u{627}\u{648}\u{644}\u{62a}\u{643}",
            "\u{634}\u{627}\u{647}\u{62f} \u{627}\u{644}\u{628}\u{631}\u{646}\u{627}\u{645}\u{62c}",
            ["\u{633}\u{637}\u{62d}", "\u{645}\u{648}\u{633}\u{64a}\u{642}\u{649} \u{62d}\u{64a}\u{651}\u{629}", "\u{62d}\u{62a}\u{649} \u{662}\u{635}"],
        )
    } else {
        (
            "LIVE TONIGHT \u{b7} RIYADH",
            "Unforgettable nights above Riyadh",
            "Live music, dinner under the stars, and a rooftop that runs till dawn \u{2014} every night from 9pm.",
            "Reserve a table",
            "See the lineup",
            ["Rooftop", "Live music", "Open till 2am"],
        )
    };
    let mut hero = v_flex().px(px(44.0)).py(px(40.0)).bg(theme::bg_raised());
    if rtl {
        hero = hero.items_end();
    }
    hero = hero
        .child(div().text_size(px(12.0)).font_bold().text_color(theme::violet_soft()).child(eyebrow.to_string()))
        .child(div().mt(px(12.0)).font_family(theme::FONT_DISPLAY).text_size(px(40.0)).font_bold().text_color(theme::text_strong()).line_height(px(42.0)).child(head.to_string()))
        .child(div().mt(px(14.0)).max_w(px(460.0)).text_size(px(16.0)).text_color(theme::text_muted()).line_height(px(24.0)).child(body.to_string()))
        .child(h_flex().gap(px(10.0)).mt(px(24.0)).child(pv_btn(cta.into(), true)).child(pv_btn(cta2.into(), false)));

    v_flex()
        .child(h_flex().items_center().justify_between().mb(px(16.0)).child(kicker("IN CONTEXT")).child(rtl_toggle(rtl, cx)))
        .child(
            v_flex()
                .rounded(px(theme::RADIUS_XL))
                .overflow_hidden()
                .border_1()
                .border_color(theme::line())
                .bg(theme::hex(0x070809))
                .child(hero)
                .child(
                    h_flex()
                        .flex_wrap()
                        .gap(px(10.0))
                        .px(px(44.0))
                        .py(px(24.0))
                        .border_t_1()
                        .border_color(theme::line_faint())
                        .child(chip_view(DsChipKind::Accent, chips[0].into()))
                        .child(chip_view(DsChipKind::Skill, chips[1].into()))
                        .child(chip_view(DsChipKind::Success, chips[2].into())),
                ),
        )
        .child(div().mt(px(14.0)).text_size(px(12.0)).text_color(theme::text_dim()).child("A live composition using your tokens & components \u{2014} toggle direction to preview RTL."))
}

fn pv_btn(label: SharedString, primary: bool) -> impl IntoElement {
    let mut b = h_flex().h(px(40.0)).px(px(18.0)).items_center().justify_center().rounded(px(theme::RADIUS_SM)).text_size(px(14.0)).font_semibold().flex_none().child(label);
    if primary {
        b = b.bg(theme::accent()).text_color(theme::accent_contrast());
    } else {
        b = b.text_color(theme::text_soft()).border_1().border_color(theme::line_strong());
    }
    b
}

fn rtl_toggle(on: bool, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let fg = if on { theme::accent() } else { theme::text_soft() };
    h_flex()
        .id("ds-rtl")
        .items_center()
        .gap(px(7.0))
        .px(px(13.0))
        .py(px(6.0))
        .rounded_full()
        .cursor_pointer()
        .border_1()
        .border_color(if on { theme::accent_ring() } else { theme::line() })
        .bg(if on { theme::accent_tint() } else { theme::bg_panel() })
        .text_color(fg)
        .text_size(px(12.0))
        .font_semibold()
        .child(icon("globe", 14.0, fg))
        .child(if on { "Arabic (RTL)" } else { "English (LTR)" })
        .on_click(cx.listener(|a, _, _, cx| a.toggle_ds_rtl(cx)))
}

// ── right: inspector ────────────────────────────────────────────────────────
fn inspector(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let body = if let Some(c) = app.ds_selected_color() {
        insp_color(c, cx).into_any_element()
    } else if let Some(t) = app.ds_selected_type() {
        insp_type(t, cx).into_any_element()
    } else if let Some(c) = app.ds_selected_comp() {
        insp_comp(app, c, cx).into_any_element()
    } else {
        insp_overview(app).into_any_element()
    };
    v_flex()
        .w(px(300.0))
        .flex_none()
        .h_full()
        .min_h_0()
        .bg(theme::bg_panel())
        .border_l_1()
        .border_color(theme::line())
        .child(div().id("ds-insp").flex_1().min_h_0().overflow_y_scroll().p(px(20.0)).child(body))
}

fn insp_header(mark: AnyElement, title: SharedString, cx: &mut Context<StudioApp>) -> impl IntoElement {
    h_flex()
        .items_center()
        .gap(px(9.0))
        .pb(px(14.0))
        .border_b_1()
        .border_color(theme::line_faint())
        .child(mark)
        .child(div().flex_1().text_size(px(13.0)).font_semibold().text_color(theme::text_strong()).child(title))
        .child(
            div()
                .id("ds-insp-close")
                .size(px(24.0))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(6.0))
                .text_color(theme::icon_color())
                .cursor_pointer()
                .hover(|s| s.bg(theme::bg_hover()).text_color(theme::text_strong()))
                .child(icon("close", 14.0, theme::icon_color()))
                .on_click(cx.listener(|a, _, _, cx| a.ds_clear_sel(cx))),
        )
}

fn insp_label(text: &'static str) -> impl IntoElement {
    div().mt(px(18.0)).mb(px(8.0)).text_size(px(11.0)).font_bold().text_color(theme::text_dim()).child(text)
}

fn field_display(value: SharedString) -> impl IntoElement {
    div()
        .w_full()
        .px(px(12.0))
        .py(px(9.0))
        .rounded(px(theme::RADIUS_SM))
        .bg(theme::bg_sunken())
        .border_1()
        .border_color(theme::line_strong())
        .text_size(px(13.0))
        .text_color(theme::text_strong())
        .child(value)
}

fn insp_overview(app: &StudioApp) -> impl IntoElement {
    let (colors, types, comps, radii) = app.ds_summary();
    v_flex()
        .child(div().font_family(theme::FONT_DISPLAY).text_size(px(16.0)).font_semibold().text_color(theme::text_strong()).child("System overview"))
        .child(div().mt(px(6.0)).text_size(px(12.5)).text_color(theme::text_muted()).line_height(px(19.0)).child("Click any token or component to edit it \u{2014} just like editing a website."))
        .child(
            h_flex()
                .flex_wrap()
                .gap(px(10.0))
                .mt(px(20.0))
                .child(stat_tile(colors, "Color tokens"))
                .child(stat_tile(types, "Type styles"))
                .child(stat_tile(comps, "Components"))
                .child(stat_tile(radii, "Radii")),
        )
        .child(
            h_flex()
                .gap(px(10.0))
                .items_start()
                .mt(px(18.0))
                .p(px(14.0))
                .rounded(px(theme::RADIUS_MD))
                .border_1()
                .border_color(theme::line())
                .bg(theme::bg_raised())
                .child(icon("boxes", 16.0, theme::accent()))
                .child(div().flex_1().text_size(px(12.0)).text_color(theme::text_muted()).line_height(px(18.0)).child("Applied to 3 websites. Changes here restyle them on next build.")),
        )
}

fn stat_tile(n: usize, label: &'static str) -> impl IntoElement {
    v_flex()
        .w(px(124.0))
        .p(px(16.0))
        .rounded(px(theme::RADIUS_MD))
        .border_1()
        .border_color(theme::line())
        .bg(theme::bg_raised())
        .child(div().font_family(theme::FONT_DISPLAY).text_size(px(24.0)).font_bold().text_color(theme::text_strong()).child(format!("{n}")))
        .child(div().mt(px(2.0)).text_size(px(11.5)).text_color(theme::text_dim()).child(label))
}

fn insp_color(c: &DsColorToken, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let mark = div().size(px(26.0)).rounded(px(8.0)).bg(theme::hex(c.val)).border_1().border_color(theme::line_bright()).flex_none().into_any_element();
    let id = c.id.clone();
    v_flex()
        .child(insp_header(mark, "Color token".into(), cx))
        .child(insp_label("NAME"))
        .child(field_display(c.name.clone()))
        .child(insp_label("VALUE"))
        .child(
            h_flex()
                .h(px(64.0))
                .items_end()
                .p(px(8.0))
                .rounded(px(theme::RADIUS_MD))
                .bg(theme::hex(c.val))
                .border_1()
                .border_color(theme::line_bright())
                .child(div().px(px(7.0)).py(px(2.0)).rounded(px(5.0)).bg(theme::white(0.55)).font_family(theme::FONT_MONO).text_size(px(12.0)).text_color(theme::hexa(0x08131FB3)).child(format!("#{:06X}", c.val))),
        )
        .child(swatch_picker(id, c.val, cx))
        .child(insp_label("ROLE"))
        .child(div().text_size(px(12.5)).text_color(theme::text_soft()).line_height(px(19.0)).px(px(13.0)).py(px(11.0)).rounded(px(theme::RADIUS_SM)).bg(theme::bg_raised()).border_1().border_color(theme::line()).child(c.role.clone()))
}

fn swatch_picker(color_id: SharedString, current: u32, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let mut row = h_flex().flex_wrap().gap(px(8.0)).mt(px(12.0));
    for (i, hex) in DS_SWATCHES.iter().enumerate() {
        let hexv = *hex;
        let sel = hexv == current;
        let cid = color_id.clone();
        row = row.child(
            div()
                .id(("ds-swatch", i))
                .size(px(30.0))
                .rounded(px(8.0))
                .bg(theme::hex(hexv))
                .border_1()
                .border_color(if sel { theme::accent_ring() } else { theme::line_bright() })
                .cursor_pointer()
                .on_click(cx.listener(move |a, _, _, cx| a.set_ds_color_val(cid.as_ref(), hexv, cx))),
        );
    }
    row
}

fn insp_type(t: &DsTypeToken, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let mark = icon("type", 16.0, theme::accent()).into_any_element();
    let id = t.id.clone();
    let size_frac = ((t.size - 10.0) / 62.0).clamp(0.0, 1.0);
    let track_frac = ((t.tracking + 4.0) / 16.0).clamp(0.0, 1.0);
    v_flex()
        .child(insp_header(mark, format!("{} style", t.name).into(), cx))
        .child(
            div()
                .mt(px(16.0))
                .p(px(16.0))
                .min_h(px(70.0))
                .rounded(px(theme::RADIUS_MD))
                .bg(theme::bg_sunken())
                .border_1()
                .border_color(theme::line())
                .flex()
                .items_center()
                .overflow_hidden()
                .child(div().text_size(px(t.size.min(48.0))).font_weight(gpui::FontWeight(t.weight as f32)).font_family(t.font()).text_color(theme::text_strong()).child(t.sample.clone())),
        )
        .child(insp_label("FAMILY"))
        .child(field_display(t.family.clone()))
        .child(stepper_row("SIZE", format!("{}px", t.size as i32), size_frac, id.clone(), StepTarget::Size, cx))
        .child(insp_label("WEIGHT"))
        .child(weight_row(t.weight, id.clone(), cx))
        .child(stepper_row("TRACKING", format!("{}", t.tracking as i32), track_frac, id, StepTarget::Tracking, cx))
}

#[derive(Clone, Copy)]
enum StepTarget {
    Size,
    Tracking,
    Slider,
}

fn stepper_row(label: &'static str, value: String, frac: f32, id: SharedString, target: StepTarget, cx: &mut Context<StudioApp>) -> impl IntoElement {
    v_flex()
        .mt(px(18.0))
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .mb(px(8.0))
                .child(div().text_size(px(11.0)).font_bold().text_color(theme::text_dim()).child(label))
                .child(div().text_size(px(11.0)).font_family(theme::FONT_MONO).text_color(theme::text_soft()).child(value)),
        )
        .child(
            h_flex()
                .gap(px(6.0))
                .items_center()
                .child(step_btn("minus", id.clone(), target, false, cx))
                .child(div().flex_1().h(px(6.0)).rounded_full().bg(theme::bg_sunken()).overflow_hidden().child(div().h_full().w(relative(frac)).bg(theme::accent_grad())))
                .child(step_btn("plus", id, target, true, cx)),
        )
}

fn step_btn(glyph: &'static str, tid: SharedString, target: StepTarget, up: bool, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let key = match target {
        StepTarget::Size => "ds-step-size",
        StepTarget::Tracking => "ds-step-track",
        StepTarget::Slider => "ds-step-slider",
    };
    div()
        .id((key, up as usize))
        .size(px(30.0))
        .rounded(px(8.0))
        .bg(theme::bg_sunken())
        .border_1()
        .border_color(theme::line_strong())
        .flex()
        .items_center()
        .justify_center()
        .flex_none()
        .cursor_pointer()
        .hover(|s| s.bg(theme::bg_hover()))
        .child(icon(glyph, 14.0, theme::text_soft()))
        .on_click(cx.listener(move |a, _, _, cx| match target {
            StepTarget::Size => a.bump_ds_type_size(tid.as_ref(), if up { 2.0 } else { -2.0 }, cx),
            StepTarget::Tracking => a.bump_ds_type_tracking(tid.as_ref(), if up { 1.0 } else { -1.0 }, cx),
            StepTarget::Slider => a.bump_ds_slider(if up { 5 } else { -5 }, cx),
        }))
}

fn weight_row(current: u16, id: SharedString, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let mut row = h_flex().gap(px(5.0)).p(px(4.0)).rounded(px(theme::RADIUS_SM)).bg(theme::bg_sunken());
    for (i, (w, lbl)) in DS_WEIGHTS.iter().enumerate() {
        let w = *w;
        let sel = w == current;
        let idc = id.clone();
        row = row.child(
            h_flex()
                .id(("ds-w", i))
                .flex_1()
                .justify_center()
                .py(px(7.0))
                .rounded(px(7.0))
                .cursor_pointer()
                .text_size(px(11.0))
                .font_semibold()
                .when(sel, |e| e.bg(theme::accent()).text_color(theme::accent_contrast()))
                .when(!sel, |e| e.text_color(theme::text_soft()))
                .child(*lbl)
                .on_click(cx.listener(move |a, _, _, cx| a.set_ds_type_weight(idc.as_ref(), w, cx))),
        );
    }
    row
}

fn insp_comp(app: &StudioApp, c: &'static DsComp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let mark = icon("boxes", 16.0, theme::accent()).into_any_element();
    let mut v = v_flex().child(insp_header(mark, c.label.into(), cx));
    if c.ready() {
        v = v
            .child(insp_label("LIVE PREVIEW"))
            .child(
                div()
                    .min_h(px(66.0))
                    .mt(px(4.0))
                    .rounded(px(theme::RADIUS_MD))
                    .bg(theme::bg_sunken())
                    .border_1()
                    .border_color(theme::line())
                    .flex()
                    .items_center()
                    .justify_center()
                    .p(px(16.0))
                    .child(specimen(c.kind, &app.ds_demo, true, cx)),
            )
            .child(comp_controls(app, c.kind, cx));
    } else {
        v = v
            .child(
                h_flex()
                    .mt(px(16.0))
                    .gap(px(10.0))
                    .items_start()
                    .p(px(14.0))
                    .rounded(px(theme::RADIUS_MD))
                    .border_1()
                    .border_dashed()
                    .border_color(theme::line_bright())
                    .bg(theme::bg_raised())
                    .child(icon("sparkle", 16.0, theme::accent()))
                    .child(div().flex_1().text_size(px(12.5)).text_color(theme::text_muted()).line_height(px(19.0)).child(format!("{} \u{2014} not generated yet. Ask the assistant to build it and it\u{2019}ll become a live, editable specimen.", c.label))),
            )
            .child(
                h_flex()
                    .id("ds-generate")
                    .w_full()
                    .mt(px(12.0))
                    .py(px(11.0))
                    .rounded(px(9.0))
                    .items_center()
                    .justify_center()
                    .bg(theme::accent())
                    .text_color(theme::accent_contrast())
                    .text_size(px(13.0))
                    .font_bold()
                    .cursor_pointer()
                    .shadow(theme::glow_accent())
                    .hover(|s| s.bg(theme::accent_hover()))
                    .child(format!("Generate {}", c.label))
                    .on_click(cx.listener(|a, _, _, cx| a.ds_generate_sel(cx))),
            )
            .child(insp_label("PLANNED VARIANTS"))
            .child(planned_variants(c.kind));
    }
    v
}

fn planned_variants(kind: DsCompKind) -> impl IntoElement {
    let mut col = v_flex().gap(px(7.0));
    for v in ds_planned_variants(kind) {
        col = col.child(
            h_flex()
                .items_center()
                .gap(px(9.0))
                .px(px(12.0))
                .py(px(10.0))
                .rounded(px(theme::RADIUS_SM))
                .bg(theme::bg_raised())
                .border_1()
                .border_color(theme::line())
                .text_size(px(12.5))
                .text_color(theme::text_soft())
                .child(dot(6.0, theme::text_dim()))
                .child(*v),
        );
    }
    col
}

fn comp_controls(app: &StudioApp, kind: DsCompKind, cx: &mut Context<StudioApp>) -> AnyElement {
    match kind {
        DsCompKind::Button => v_flex()
            .child(insp_label("VARIANT"))
            .child(variant_grid(app, cx))
            .child(insp_label("SIZE"))
            .child(size_row(app, cx))
            .child(insp_label("LABEL"))
            .child(field_display(app.ds_demo.button_label.clone()))
            .into_any_element(),
        DsCompKind::Chip => v_flex()
            .child(insp_label("KIND"))
            .child(chip_grid(app, cx))
            .child(insp_label("LABEL"))
            .child(field_display(app.ds_demo.chip_label.clone()))
            .into_any_element(),
        DsCompKind::Status => v_flex().child(insp_label("TONE")).child(status_grid(app, cx)).into_any_element(),
        DsCompKind::Avatar => v_flex()
            .child(insp_label("TONE"))
            .child(avatar_grid(app, cx))
            .child(insp_label("INITIALS"))
            .child(field_display(app.ds_demo.avatar_initials.clone()))
            .into_any_element(),
        DsCompKind::Slider => stepper_row("VALUE", format!("{}", app.ds_demo.slider), app.ds_demo.slider as f32 / 100.0, "slider".into(), StepTarget::Slider, cx).into_any_element(),
        DsCompKind::Toggle => div().mt(px(16.0)).text_size(px(12.5)).text_color(theme::text_muted()).line_height(px(19.0)).child("Flip the switch above \u{2014} state persists and drives the in-canvas specimen too.").into_any_element(),
        DsCompKind::Input | DsCompKind::Textarea => v_flex().child(insp_label("PLACEHOLDER")).child(field_display(app.ds_demo.input_ph.clone())).into_any_element(),
        _ => div().into_any_element(),
    }
}

fn opt_button(id: impl Into<ElementId>, label: &'static str, selected: bool, on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> AnyElement {
    h_flex()
        .id(id)
        .flex_1()
        .justify_center()
        .py(px(8.0))
        .rounded(px(8.0))
        .border_1()
        .cursor_pointer()
        .text_size(px(11.5))
        .font_semibold()
        .border_color(if selected { theme::accent_ring() } else { theme::line() })
        .bg(if selected { theme::accent_tint() } else { theme::bg_sunken() })
        .text_color(if selected { theme::accent() } else { theme::text_soft() })
        .child(label)
        .on_click(on_click)
        .into_any_element()
}

fn two_col(items: Vec<AnyElement>) -> impl IntoElement {
    let mut rows: Vec<AnyElement> = Vec::new();
    let mut it = items.into_iter();
    while let Some(a) = it.next() {
        let row = if let Some(b) = it.next() {
            h_flex().gap(px(5.0)).child(a).child(b)
        } else {
            h_flex().gap(px(5.0)).child(a).child(div().flex_1())
        };
        rows.push(row.into_any_element());
    }
    v_flex().gap(px(5.0)).children(rows)
}

fn variant_grid(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let cur = app.ds_demo.button_variant;
    let mut opts = Vec::new();
    for (i, v) in DsBtnVariant::ALL.iter().enumerate() {
        let v = *v;
        opts.push(opt_button(("ds-var", i), v.label(), v == cur, cx.listener(move |a, _, _, cx| a.set_ds_btn_variant(v, cx))));
    }
    two_col(opts)
}

fn size_row(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let cur = app.ds_demo.button_size;
    let mut row = h_flex().gap(px(5.0)).p(px(4.0)).rounded(px(8.0)).bg(theme::bg_sunken());
    for (i, s) in DsBtnSize::ALL.iter().enumerate() {
        let s = *s;
        let sel = s == cur;
        row = row.child(
            h_flex()
                .id(("ds-size", i))
                .flex_1()
                .justify_center()
                .py(px(7.0))
                .rounded(px(6.0))
                .cursor_pointer()
                .text_size(px(11.5))
                .font_semibold()
                .when(sel, |e| e.bg(theme::accent()).text_color(theme::accent_contrast()))
                .when(!sel, |e| e.text_color(theme::text_soft()))
                .child(s.label())
                .on_click(cx.listener(move |a, _, _, cx| a.set_ds_btn_size(s, cx))),
        );
    }
    row
}

fn chip_grid(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let cur = app.ds_demo.chip_kind;
    let mut opts = Vec::new();
    for (i, k) in DsChipKind::ALL.iter().enumerate() {
        let k = *k;
        opts.push(opt_button(("ds-chipk", i), k.label(), k == cur, cx.listener(move |a, _, _, cx| a.set_ds_chip_kind(k, cx))));
    }
    two_col(opts)
}

fn status_grid(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let cur = app.ds_demo.status_tone;
    let mut opts = Vec::new();
    for (i, t) in DsStatusTone::ALL.iter().enumerate() {
        let t = *t;
        opts.push(opt_button(("ds-status", i), t.label(), t == cur, cx.listener(move |a, _, _, cx| a.set_ds_status_tone(t, cx))));
    }
    two_col(opts)
}

fn avatar_grid(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let cur = app.ds_demo.avatar_tone;
    let mut opts = Vec::new();
    for (i, t) in DsAvatarTone::ALL.iter().enumerate() {
        let t = *t;
        opts.push(opt_button(("ds-avt", i), t.label(), t == cur, cx.listener(move |a, _, _, cx| a.set_ds_avatar_tone(t, cx))));
    }
    two_col(opts)
}
