//! Left assistant panel: header, message transcript (with an empty-state and a
//! compiling indicator), the selection chips, and the composer — whose buttons
//! open the attach / skills / design-system / API / model popovers.

use gpui::{AnyElement, Context, SharedString, Window, div, prelude::*, px, relative};
use gpui_component::{StyledExt, h_flex, input::Input, v_flex};

use crate::app::StudioApp;
use crate::state::{ChatMenu, Dir, Effort, Message, Permission, ProjectKind, Role, SKILL_NAMES, method_colors};
use crate::theme;
use crate::ui::widgets::icon;

pub fn render(app: &StudioApp, _window: &mut Window, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let menu_open = app.chat_menu.is_some() || app.ds_picker_open || app.api_panel_open;
    v_flex()
        .flex_none()
        .h_full()
        .w(px(360.0))
        .min_h_0()
        .bg(theme::bg_base())
        .border_r_1()
        .border_color(theme::line())
        .child(header(app, cx))
        // messages region: the popovers live *here* (below the header/toolbar)
        // so they can't cover the toolbar, and are height-capped to it + scroll.
        .child(
            v_flex()
                .relative()
                .flex_1()
                .min_h_0()
                .child(messages(app, cx))
                .when(menu_open, |r| {
                    r.child(div().id("menu-backdrop").absolute().inset_0().on_click(cx.listener(|a, _, _, cx| a.close_composer_menus(cx))))
                        .child(popover(app, cx))
                }),
        )
        .when(app.generated && !app.selection.is_empty(), |this| this.child(selection_chips(app, cx)))
        .child(composer(app, cx))
}

/// The active composer popover (attach / skills / model / DS / API), or nothing.
fn popover(app: &StudioApp, cx: &mut Context<StudioApp>) -> AnyElement {
    if app.chat_menu == Some(ChatMenu::Attach) {
        attach_menu(cx).into_any_element()
    } else if app.chat_menu == Some(ChatMenu::Skills) {
        skills_menu(app, cx).into_any_element()
    } else if app.chat_menu == Some(ChatMenu::Model) {
        model_menu(app, cx).into_any_element()
    } else if app.ds_picker_open {
        ds_menu(app, cx).into_any_element()
    } else if app.api_panel_open {
        api_panel(app, cx).into_any_element()
    } else {
        div().into_any_element()
    }
}

fn header(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
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
        .child(
            h_flex()
                .items_center()
                .gap(px(6.0))
                .text_size(px(12.0))
                .text_color(theme::text_muted())
                .child(div().size(px(6.0)).rounded_full().bg(theme::success()))
                .child(app.provider().name),
        )
        .child(
            div()
                .id("chat-collapse")
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
        )
}

fn messages(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let mut list = v_flex().id("messages").flex_1().min_h_0().overflow_y_scroll().p(px(16.0)).gap(px(14.0));

    if app.messages.is_empty() {
        let rtl = app.dir == Dir::Rtl;
        let sugg: [&str; 3] = if rtl {
            [
                "\u{645}\u{642}\u{647}\u{649} \u{633}\u{637}\u{62d} \u{644}\u{644}\u{645}\u{648}\u{633}\u{64a}\u{642}\u{649} \u{627}\u{644}\u{62d}\u{64a}\u{651}\u{629} \u{641}\u{64a} \u{627}\u{644}\u{631}\u{64a}\u{627}\u{636}\u{60c} \u{628}\u{627}\u{644}\u{639}\u{631}\u{628}\u{64a}\u{629}",
                "\u{635}\u{641}\u{62d}\u{629} \u{645}\u{642}\u{647}\u{649} \u{645}\u{639} \u{642}\u{627}\u{626}\u{645}\u{629} \u{648}\u{645}\u{648}\u{627}\u{639}\u{64a}\u{62f}",
                "\u{628}\u{648}\u{631}\u{62a}\u{641}\u{648}\u{644}\u{64a}\u{648} \u{644}\u{645}\u{635}\u{648}\u{651}\u{631} \u{641}\u{648}\u{62a}\u{648}\u{63a}\u{631}\u{627}\u{641}\u{64a}",
            ]
        } else {
            ["A rooftop live-music venue in Riyadh, Arabic-first", "A caf\u{e9} landing with menu & hours", "A photographer\u{2019}s portfolio"]
        };
        list = list.child(
            v_flex()
                .gap(px(12.0))
                .child(div().text_size(px(13.5)).line_height(px(22.0)).text_color(theme::text_soft()).child("Hi Rana \u{1f44b} Tell me what you want to build \u{2014} in Arabic or English. Try one of these:"))
                .child(v_flex().gap(px(8.0)).children(sugg.into_iter().enumerate().map(|(i, s)| suggestion_btn(i, s, cx)))),
        );
    } else {
        list = list.children(app.messages.iter().enumerate().map(|(i, m)| message_row(i, m)));
    }

    if app.busy {
        list = list.child(
            h_flex()
                .items_center()
                .gap(px(10.0))
                .p(px(14.0))
                .rounded(px(theme::RADIUS_MD))
                .bg(theme::bg_panel())
                .border_1()
                .border_color(theme::line())
                .text_size(px(13.0))
                .text_color(theme::text_soft())
                .child(icon("loader", 15.0, theme::accent()))
                .child(SharedString::from(app.compile_text())),
        );
    }
    list
}

fn suggestion_btn(i: usize, text: &str, cx: &mut Context<StudioApp>) -> impl IntoElement + use<> {
    let owned: SharedString = text.to_string().into();
    let for_click = owned.clone();
    h_flex()
        .id(("sugg", i))
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
        .child(div().child(owned))
        .on_click(cx.listener(move |a, _, window, cx| a.run_suggestion(&for_click, window, cx)))
}

fn message_row(i: usize, m: &Message) -> impl IntoElement + use<> {
    let user = m.role == Role::User;
    let mut bubble = div()
        .id(("msg", i))
        .max_w(px(282.0))
        .px(px(14.0))
        .py(px(11.0))
        .text_size(px(13.0))
        .line_height(px(20.0))
        .child(m.text.clone());
    if user {
        bubble = bubble.bg(theme::accent_grad_soft()).text_color(theme::white(1.0)).rounded(px(14.0)).rounded_br(px(4.0)).shadow(theme::glow_violet());
    } else {
        bubble = bubble.bg(theme::bg_panel()).border_1().border_color(theme::line()).text_color(theme::text_soft()).rounded(px(14.0)).rounded_bl(px(4.0));
    }
    h_flex().w_full().when(user, |r| r.justify_end()).child(bubble)
}

fn selection_chips(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let multi = app.selection.len() > 1;
    let count_label = if multi { format!("{} elements", app.selection.len()) } else { "1 element".to_string() };
    v_flex()
        .flex_none()
        .px(px(16.0))
        .pb(px(10.0))
        .child(
            h_flex()
                .items_center()
                .gap(px(6.0))
                .mb(px(7.0))
                .child(icon("target", 13.0, theme::accent()))
                .child(div().text_size(px(11.0)).font_bold().text_color(theme::text_caption()).child(format!("EDITING {}", count_label.to_uppercase())))
                .child(div().flex_1())
                .when(multi, |r| {
                    r.child(
                        div()
                            .id("sel-clear")
                            .text_size(px(11.0))
                            .font_semibold()
                            .text_color(theme::text_muted())
                            .cursor_pointer()
                            .hover(|s| s.text_color(theme::text_strong()))
                            .child("Clear")
                            .on_click(cx.listener(|a, _, _, cx| a.deselect(cx))),
                    )
                }),
        )
        .child(
            h_flex().flex_wrap().gap(px(6.0)).children(app.selection.iter().enumerate().map(|(i, key)| {
                let k = key.clone();
                let label = app.sel_label(key);
                let ic = app.sel_icon(key);
                h_flex()
                    .id(("selchip", i))
                    .items_center()
                    .gap(px(6.0))
                    .px(px(9.0))
                    .py(px(5.0))
                    .rounded_full()
                    .bg(theme::accent_tint())
                    .border_1()
                    .border_color(theme::accent_ring())
                    .text_size(px(11.5))
                    .font_semibold()
                    .text_color(theme::accent())
                    .child(icon(ic, 12.0, theme::accent()))
                    .child(div().child(label))
                    .child(
                        div()
                            .id(("selx", i))
                            .cursor_pointer()
                            .child(icon("close", 12.0, theme::accent()))
                            .on_click(cx.listener(move |a, _, _, cx| a.remove_from_selection(k.as_ref(), cx))),
                    )
            })),
        )
}

fn composer(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let skill_count = app.skills.len();
    let ds_label = app.applied_ds_name();
    let api_connected = app.api_spec.is_some();
    let api_label: SharedString = match &app.api_spec {
        Some(s) => format!("{}/{} endpoints", app.api_bound_count(), s.endpoints.len()).into(),
        None => "Connect API".into(),
    };
    let model_label: SharedString = format!("{} \u{b7} {}", app.chat_model_label(), app.effort.label()).into();

    let dock = v_flex()
        .rounded(px(theme::RADIUS_XL))
        .border_1()
        .border_color(theme::line_strong())
        .bg(theme::bg_sunken())
        .px(px(14.0))
        .py(px(12.0))
        .gap(px(10.0))
        .child(Input::new(&app.prompt).appearance(false).text_size(px(13.5)))
        // tools row: attach / skills / design system / API (wraps if needed)
        .child(
            h_flex()
                .w_full()
                .flex_wrap()
                .items_center()
                .gap(px(6.0))
                .child(round_btn("cp-attach", "paperclip", cx.listener(|a, _, _, cx| a.toggle_chat_menu(ChatMenu::Attach, cx))))
                .child(pill_btn("cp-skills", "grid", &format!("{skill_count}"), false, cx.listener(|a, _, _, cx| a.toggle_chat_menu(ChatMenu::Skills, cx))))
                .child(pill_btn_bordered("cp-ds", "boxes", ds_label, cx.listener(|a, _, _, cx| a.toggle_ds_picker(cx))))
                .child(pill_btn_bordered_state("cp-api", "plug", api_label, api_connected, cx.listener(|a, _, _, cx| a.toggle_api_panel(cx)))),
        )
        // action row: model selector on the left, send pinned bottom-right
        .child(
            h_flex()
                .w_full()
                .items_center()
                .gap(px(8.0))
                .child(pill_btn_bordered("cp-model", "cpu", model_label, cx.listener(|a, _, _, cx| a.toggle_chat_menu(ChatMenu::Model, cx))))
                .child(div().flex_1())
                .child(
                    div()
                        .id("cp-send")
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
                        .on_click(cx.listener(|a, _, window, cx| a.send_prompt(window, cx))),
                ),
        );

    v_flex().flex_none().px(px(16.0)).pt(px(14.0)).pb(px(16.0)).child(dock)
}

// ── composer buttons ─────────────────────────────────────────────────────────
fn round_btn(id: &'static str, ic: &'static str, on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut gpui::App) + 'static) -> impl IntoElement {
    div()
        .id(id)
        .size(px(32.0))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .bg(theme::bg_hover())
        .text_color(theme::text_soft())
        .cursor_pointer()
        .hover(|s| s.text_color(theme::text_strong()))
        .child(icon(ic, 15.0, theme::text_soft()))
        .on_click(on_click)
}

fn pill_btn(id: &'static str, ic: &'static str, label: &str, _active: bool, on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut gpui::App) + 'static) -> impl IntoElement {
    h_flex()
        .id(id)
        .flex_none()
        .h(px(32.0))
        .items_center()
        .gap(px(6.0))
        .px(px(11.0))
        .rounded_full()
        .bg(theme::bg_hover())
        .text_size(px(12.0))
        .font_semibold()
        .text_color(theme::text_soft())
        .cursor_pointer()
        .hover(|s| s.text_color(theme::text_strong()))
        .child(icon(ic, 15.0, theme::text_soft()))
        .child(label.to_string())
        .on_click(on_click)
}

fn pill_btn_bordered(id: &'static str, ic: &'static str, label: SharedString, on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut gpui::App) + 'static) -> impl IntoElement {
    h_flex()
        .id(id)
        .flex_shrink()
        .min_w_0()
        .max_w(px(150.0))
        .h(px(32.0))
        .items_center()
        .gap(px(5.0))
        .px(px(10.0))
        .rounded_full()
        .border_1()
        .border_color(theme::line())
        .text_size(px(11.5))
        .font_semibold()
        .text_color(theme::text_soft())
        .cursor_pointer()
        .hover(|s| s.bg(theme::bg_hover()).text_color(theme::text_strong()))
        .child(icon(ic, 14.0, theme::accent()))
        .child(div().min_w_0().overflow_hidden().child(label))
        .child(icon("chevron-down", 14.0, theme::text_muted()))
        .on_click(on_click)
}

fn pill_btn_bordered_state(id: &'static str, ic: &'static str, label: SharedString, active: bool, on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut gpui::App) + 'static) -> impl IntoElement {
    let (border, bg, ic_color) = if active {
        (theme::accent_ring(), theme::accent_tint(), theme::success())
    } else {
        (theme::line(), gpui::transparent_black(), theme::icon_color())
    };
    h_flex()
        .id(id)
        .flex_shrink()
        .min_w_0()
        .max_w(px(160.0))
        .h(px(32.0))
        .items_center()
        .gap(px(5.0))
        .px(px(10.0))
        .rounded_full()
        .border_1()
        .border_color(border)
        .bg(bg)
        .text_size(px(11.5))
        .font_semibold()
        .text_color(theme::text_soft())
        .cursor_pointer()
        .hover(|s| s.text_color(theme::text_strong()))
        .child(icon(ic, 14.0, ic_color))
        .child(div().min_w_0().overflow_hidden().child(label))
        .child(icon("chevron-down", 14.0, theme::text_muted()))
        .on_click(on_click)
}

// ── popovers ─────────────────────────────────────────────────────────────────
fn menu_shell(min_w: f32) -> gpui::Stateful<gpui::Div> {
    div()
        .id("cmenu")
        .absolute()
        .bottom(px(8.0))
        .left(px(8.0))
        .min_w(px(min_w))
        .max_h(relative(1.0))
        .overflow_y_scroll()
        .bg(theme::bg_raised())
        .border_1()
        .border_color(theme::line_strong())
        .rounded(px(theme::RADIUS_LG))
        .shadow(theme::shadow_pop())
        .p(px(8.0))
}

fn menu_label(text: &'static str) -> impl IntoElement {
    div().text_size(px(10.5)).font_bold().text_color(theme::text_caption()).px(px(8.0)).pt(px(2.0)).pb(px(8.0)).child(text)
}

fn attach_menu(cx: &mut Context<StudioApp>) -> impl IntoElement {
    let items = [("Upload an image", "image"), ("Import from Figma", "grid"), ("Attach a PDF", "type"), ("Paste a URL", "link")];
    menu_shell(240.0)
        .w(px(240.0))
        .children(items.into_iter().enumerate().map(|(i, (label, ic))| menu_row(("att", i), ic, label, cx.listener(|a, _, _, cx| a.close_chat_menu(cx)))))
        .child(menu_row(("att", 9usize), "plug", "Attach OpenAPI spec", cx.listener(|a, _, _, cx| a.attach_openapi(cx))))
}

fn skills_menu(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    menu_shell(262.0).w(px(262.0)).child(menu_label("SKILLS TO APPLY")).children(SKILL_NAMES.iter().enumerate().map(|(i, name)| {
        let on = app.skills.contains(&i);
        h_flex()
            .id(("skrow", i))
            .items_center()
            .gap(px(11.0))
            .w_full()
            .px(px(8.0))
            .py(px(9.0))
            .rounded(px(8.0))
            .cursor_pointer()
            .hover(|s| s.bg(theme::bg_hover()))
            .on_click(cx.listener(move |a, _, _, cx| a.toggle_skill_idx(i, cx)))
            .child(check_box(on))
            .child(div().text_size(px(13.0)).text_color(theme::text_soft()).child(*name))
    }))
}

fn model_menu(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    div()
        .id("cmenu")
        .absolute()
        .bottom(px(8.0))
        .right(px(8.0))
        .w(px(306.0))
        .max_h(relative(1.0))
        .overflow_y_scroll()
        .bg(theme::bg_raised())
        .border_1()
        .border_color(theme::line_strong())
        .rounded(px(theme::RADIUS_LG))
        .shadow(theme::shadow_pop())
        .p(px(10.0))
        .child(menu_label("MODEL"))
        .child(v_flex().gap(px(2.0)).children(app.provider_models().into_iter().enumerate().map(|(mi, (id, name, active))| {
            h_flex()
                .id(("model", mi))
                .items_center()
                .gap(px(10.0))
                .w_full()
                .px(px(10.0))
                .py(px(9.0))
                .rounded(px(9.0))
                .cursor_pointer()
                .when(active, |d| d.bg(theme::accent_tint()))
                .hover(|s| s.bg(theme::bg_hover()))
                .on_click(cx.listener(move |a, _, _, cx| a.set_chat_model(&id, cx)))
                .child(div().flex_1().text_size(px(13.0)).font_semibold().text_color(theme::text_strong()).child(name))
                .when(active, |d| d.child(icon("check", 15.0, theme::accent())))
        })))
        .child(div().h(px(1.0)).my(px(10.0)).bg(theme::line_faint()))
        .child(menu_label("EFFORT"))
        .child(h_flex().gap(px(5.0)).mb(px(12.0)).children([Effort::Fast, Effort::Balanced, Effort::Max].into_iter().enumerate().map(|(ei, e)| {
            let active = app.effort == e;
            let mut b = div().id(("eff", ei)).flex_1().h(px(30.0)).flex().items_center().justify_center().rounded(px(8.0)).text_size(px(12.0)).font_semibold().cursor_pointer().on_click(cx.listener(move |a, _, _, cx| a.set_effort(e, cx)));
            if active {
                b = b.bg(theme::accent()).text_color(theme::accent_contrast());
            } else {
                b = b.bg(theme::bg_raised()).text_color(theme::text_soft());
            }
            b.child(e.label())
        })))
        .child(menu_label("PERMISSIONS"))
        .child(v_flex().gap(px(6.0)).children([Permission::Review, Permission::Safe, Permission::Manual].into_iter().enumerate().map(|(pi, p)| {
            let active = app.permission == p;
            h_flex()
                .id(("perm", pi))
                .items_start()
                .gap(px(10.0))
                .w_full()
                .px(px(11.0))
                .py(px(10.0))
                .rounded(px(10.0))
                .border_1()
                .border_color(if active { theme::accent_ring() } else { theme::line() })
                .when(active, |d| d.bg(theme::accent_tint()))
                .cursor_pointer()
                .on_click(cx.listener(move |a, _, _, cx| a.set_permission(p, cx)))
                .child(v_flex().flex_1().child(div().text_size(px(12.5)).font_semibold().text_color(theme::text_strong()).child(p.label())).child(div().mt(px(2.0)).text_size(px(11.0)).text_color(theme::text_caption()).line_height(px(16.0)).child(p.desc())))
                .when(active, |d| d.child(icon("check", 15.0, theme::accent())))
        })))
}

fn ds_menu(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let systems: Vec<_> = app.projects.iter().filter(|p| p.kind == ProjectKind::System).cloned().collect();
    div()
        .id("cmenu")
        .absolute()
        .bottom(px(8.0))
        .left(px(8.0))
        .right(px(8.0))
        .max_h(relative(1.0))
        .overflow_y_scroll()
        .bg(theme::bg_raised())
        .border_1()
        .border_color(theme::line_strong())
        .rounded(px(theme::RADIUS_LG))
        .shadow(theme::shadow_pop())
        .p(px(8.0))
        .child(menu_label("DESIGN SYSTEM"))
        .children(systems.into_iter().enumerate().map(|(di, p)| {
            let id = p.id.clone();
            let active = app.applied_ds.as_ref() == Some(&p.id);
            h_flex()
                .id(("ds", di))
                .items_center()
                .gap(px(10.0))
                .w_full()
                .px(px(8.0))
                .py(px(9.0))
                .rounded(px(9.0))
                .cursor_pointer()
                .hover(|s| s.bg(theme::bg_hover()))
                .on_click(cx.listener(move |a, _, _, cx| a.choose_ds(id.clone(), cx)))
                .child(div().size(px(30.0)).rounded(px(8.0)).bg(theme::accent_grad()).flex().items_center().justify_center().font_family(theme::FONT_DISPLAY).font_bold().text_size(px(13.0)).text_color(theme::accent_contrast()).child(p.mono.clone()))
                .child(v_flex().flex_1().min_w_0().child(div().text_size(px(13.0)).font_semibold().text_color(theme::text_strong()).child(p.name.clone())).child(div().text_size(px(11.0)).text_color(theme::text_caption()).child(p.sub.clone())))
                .when(active, |d| d.child(icon("check", 15.0, theme::accent())))
        }))
        .when(app.applied_ds.is_some(), |d| {
            d.child(div().h(px(1.0)).mx(px(6.0)).my(px(5.0)).bg(theme::line_faint())).child(menu_row(("dsclear", 0usize), "close", "Remove design system", cx.listener(|a, _, _, cx| a.clear_ds(cx))))
        })
}

fn api_panel(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let shell = div()
        .id("api-panel")
        .absolute()
        .bottom(px(8.0))
        .left(px(8.0))
        .right(px(8.0))
        .max_h(relative(1.0))
        .overflow_y_scroll()
        .bg(theme::bg_raised())
        .border_1()
        .border_color(theme::line_strong())
        .rounded(px(theme::RADIUS_LG))
        .shadow(theme::shadow_pop())
        .p(px(12.0));

    match &app.api_spec {
        None => shell.child(
            v_flex()
                .items_center()
                .py(px(20.0))
                .gap(px(10.0))
                .child(div().size(px(46.0)).rounded(px(13.0)).bg(theme::bg_panel()).border_1().border_color(theme::line()).flex().items_center().justify_center().child(icon("plug", 22.0, theme::accent())))
                .child(div().font_family(theme::FONT_DISPLAY).text_size(px(15.0)).font_semibold().text_color(theme::text_strong()).child("Wire your site to a live API"))
                .child(div().max_w(px(340.0)).text_center().text_size(px(12.5)).text_color(theme::text_muted()).line_height(px(19.0)).child("Attach an OpenAPI (Swagger) spec and the studio integrates each endpoint \u{2014} turning your build into a working SPA."))
                .child(
                    h_flex()
                        .id("api-attach")
                        .mt(px(6.0))
                        .items_center()
                        .gap(px(8.0))
                        .h(px(36.0))
                        .px(px(16.0))
                        .rounded_full()
                        .bg(theme::accent())
                        .text_color(theme::accent_contrast())
                        .shadow(theme::glow_accent())
                        .text_size(px(12.5))
                        .font_bold()
                        .cursor_pointer()
                        .child(icon("plug", 15.0, theme::accent_contrast()))
                        .child("Attach OpenAPI spec")
                        .on_click(cx.listener(|a, _, _, cx| a.attach_openapi(cx))),
                ),
        ),
        Some(spec) => {
            let name = spec.name;
            let base = spec.base;
            let version = spec.version;
            shell
                .child(
                    h_flex()
                        .items_center()
                        .gap(px(10.0))
                        .pb(px(12.0))
                        .child(div().size(px(32.0)).rounded(px(9.0)).bg(theme::hexa(0x5CCB9A24)).flex().items_center().justify_center().child(icon("plug", 16.0, theme::success())))
                        .child(
                            v_flex()
                                .flex_1()
                                .min_w_0()
                                .child(h_flex().items_center().gap(px(8.0)).child(div().font_family(theme::FONT_MONO).text_size(px(12.5)).font_semibold().text_color(theme::text_strong()).child(name)).child(div().px(px(7.0)).py(px(2.0)).rounded(px(theme::RADIUS_XS)).bg(theme::accent_tint()).text_color(theme::accent()).text_size(px(10.0)).font_bold().child(version)))
                                .child(div().font_family(theme::FONT_MONO).text_size(px(11.0)).text_color(theme::text_caption()).child(base)),
                        )
                        .child(
                            div()
                                .id("api-remove")
                                .size(px(28.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .rounded(px(8.0))
                                .bg(theme::bg_hover())
                                .text_color(theme::text_caption())
                                .cursor_pointer()
                                .hover(|s| s.text_color(theme::danger()))
                                .child(icon("close", 14.0, theme::text_caption()))
                                .on_click(cx.listener(|a, _, _, cx| a.remove_api_spec(cx))),
                        ),
                )
                .child(menu_label("ENDPOINTS \u{b7} TOGGLE TO BIND TO YOUR SPA"))
                .child(v_flex().gap(px(5.0)).children(spec.endpoints.iter().enumerate().map(|(i, ep)| {
                    let (mc, mbg) = method_colors(ep.method);
                    let bound = ep.bound;
                    h_flex()
                        .id(("ep", i))
                        .items_center()
                        .gap(px(10.0))
                        .w_full()
                        .px(px(10.0))
                        .py(px(9.0))
                        .rounded(px(10.0))
                        .border_1()
                        .border_color(theme::line_faint())
                        .bg(theme::bg_panel())
                        .cursor_pointer()
                        .hover(|s| s.border_color(theme::accent_ring()))
                        .on_click(cx.listener(move |a, _, _, cx| a.toggle_endpoint(i, cx)))
                        .child(div().w(px(46.0)).py(px(3.0)).rounded(px(theme::RADIUS_XS)).bg(mbg).text_color(mc).font_family(theme::FONT_MONO).text_size(px(10.0)).font_bold().flex().justify_center().child(ep.method))
                        .child(v_flex().flex_1().min_w_0().child(div().font_family(theme::FONT_MONO).text_size(px(12.0)).text_color(theme::text_strong()).child(ep.path)).child(div().text_size(px(11.0)).text_color(theme::text_caption()).child(ep.desc)))
                        .child(if bound {
                            h_flex().items_center().gap(px(5.0)).text_size(px(10.5)).font_semibold().text_color(theme::success()).child(icon("check", 12.0, theme::success())).child("Bound").into_any_element()
                        } else {
                            div().text_size(px(10.5)).font_semibold().text_color(theme::text_faint()).child("Bind").into_any_element()
                        })
                })))
                .child(div().h(px(1.0)).my(px(10.0)).bg(theme::line_faint()))
                .child(
                    h_flex()
                        .items_center()
                        .gap(px(10.0))
                        .child(v_flex().flex_1().child(div().text_size(px(12.5)).font_semibold().text_color(theme::text_strong()).child("Single-page app (SPA)")).child(div().text_size(px(11.0)).text_color(theme::text_caption()).child("Client-side routing & live data fetching.")))
                        .child(toggle_switch(app.spa_mode, cx.listener(|a, _, _, cx| a.toggle_spa(cx)))),
                )
        }
    }
}

// ── small shared bits ────────────────────────────────────────────────────────
fn menu_row(
    id: impl Into<gpui::ElementId>,
    ic: &'static str,
    label: &'static str,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    h_flex()
        .id(id)
        .items_center()
        .gap(px(11.0))
        .w_full()
        .px(px(11.0))
        .py(px(10.0))
        .rounded(px(9.0))
        .text_size(px(13.0))
        .text_color(theme::text_soft())
        .cursor_pointer()
        .hover(|s| s.bg(theme::bg_hover()).text_color(theme::text_strong()))
        .child(icon(ic, 15.0, theme::accent()))
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

fn toggle_switch(on: bool, on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut gpui::App) + 'static) -> impl IntoElement {
    let mut track = h_flex()
        .id("spa-toggle")
        .w(px(42.0))
        .h(px(24.0))
        .p(px(3.0))
        .rounded_full()
        .items_center()
        .cursor_pointer()
        .on_click(on_click);
    track = if on { track.bg(theme::accent()).justify_end() } else { track.bg(theme::bg_hover()) };
    track.child(div().size(px(18.0)).rounded_full().bg(theme::white(1.0)))
}
