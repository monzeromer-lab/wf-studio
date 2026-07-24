//! Home dashboard: a grid of the user's projects, with filter tabs and a
//! "New project" entry point.

use gpui::{App, ClickEvent, Context, SharedString, Window, div, prelude::*, px};
use gpui_component::{StyledExt, h_flex, v_flex};

use crate::app::StudioApp;
use crate::state::{HomeFilter, Project, ProjectKind};
use crate::theme;
use crate::ui::widgets::{Btn, icon};

const CARD_W: f32 = 250.0;

pub fn render(app: &StudioApp, _window: &mut Window, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let projects: Vec<Project> = app
        .projects
        .iter()
        .filter(|p| match app.home_filter {
            HomeFilter::All => true,
            HomeFilter::Website => p.kind == ProjectKind::Website,
            HomeFilter::System => p.kind == ProjectKind::System,
        })
        .cloned()
        .collect();

    v_flex()
        .id("home")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .px(px(56.0))
        .py(px(40.0))
        .child(
            v_flex()
                .w_full()
                .max_w(px(1080.0))
                .mx_auto()
                .child(header(cx))
                .child(tabs_row(app, cx))
                .child(
                    h_flex()
                        .mt(px(20.0))
                        .flex_wrap()
                        .gap(px(16.0))
                        .child(new_card(cx))
                        .children(projects.into_iter().enumerate().map(|(i, p)| project_card(i, p, cx))),
                ),
        )
}

fn header(cx: &mut Context<StudioApp>) -> impl IntoElement {
    h_flex()
        .items_end()
        .gap(px(16.0))
        .flex_wrap()
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .child(
                    div()
                        .font_family(theme::FONT_DISPLAY)
                        .text_size(px(28.0))
                        .font_semibold()
                        .text_color(theme::text_strong())
                        .child("Welcome back, Rana"),
                )
                .child(div().mt(px(7.0)).text_size(px(14.5)).text_color(theme::text_muted()).child("Pick up a project, or start something new.")),
        )
        .child(Btn::primary("New project").icon("plus").render("new-proj-top", cx.listener(|a, _, window, cx| a.new_project(window, cx))))
}

fn tabs_row(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    h_flex()
        .mt(px(26.0))
        .items_center()
        .gap(px(12.0))
        .child(
            h_flex()
                .gap(px(6.0))
                .child(tab("tab-all", "All", app.home_filter == HomeFilter::All, cx.listener(|a, _, _, cx| a.set_home_filter(HomeFilter::All, cx))))
                .child(tab("tab-web", "Websites", app.home_filter == HomeFilter::Website, cx.listener(|a, _, _, cx| a.set_home_filter(HomeFilter::Website, cx))))
                .child(tab("tab-sys", "Design systems", app.home_filter == HomeFilter::System, cx.listener(|a, _, _, cx| a.set_home_filter(HomeFilter::System, cx)))),
        )
        .child(div().flex_1())
        .child(
            h_flex()
                .items_center()
                .gap(px(8.0))
                .h(px(32.0))
                .px(px(12.0))
                .min_w(px(180.0))
                .rounded(px(theme::RADIUS_SM))
                .border_1()
                .border_color(theme::line())
                .bg(theme::bg_sunken())
                .text_size(px(12.5))
                .text_color(theme::text_caption())
                .child(icon("search", 15.0, theme::text_caption()))
                .child("Search projects\u{2026}"),
        )
}

fn tab(
    id: &'static str,
    label: &'static str,
    active: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let mut b = div()
        .id(id)
        .h(px(32.0))
        .px(px(14.0))
        .flex()
        .items_center()
        .rounded(px(theme::RADIUS_SM))
        .text_size(px(12.5))
        .font_semibold()
        .cursor_pointer()
        .on_click(on_click);
    if active {
        b = b.bg(theme::bg_raised()).border_1().border_color(theme::line()).text_color(theme::text_strong());
    } else {
        b = b.text_color(theme::text_muted()).hover(|s| s.text_color(theme::text_soft()));
    }
    b.child(label)
}

fn new_card(cx: &mut Context<StudioApp>) -> impl IntoElement {
    v_flex()
        .id("new-card")
        .w(px(CARD_W))
        .min_h(px(168.0))
        .items_center()
        .justify_center()
        .gap(px(12.0))
        .rounded(px(theme::RADIUS_LG))
        .border_1()
        .border_dashed()
        .border_color(theme::line_bright())
        .text_color(theme::text_muted())
        .cursor_pointer()
        .hover(|s| s.border_color(theme::accent_ring()).bg(theme::accent_tint()).text_color(theme::text_strong()))
        .on_click(cx.listener(|a, _, window, cx| a.new_project(window, cx)))
        .child(
            div()
                .size(px(46.0))
                .rounded(px(13.0))
                .bg(theme::bg_raised())
                .flex()
                .items_center()
                .justify_center()
                .child(icon("plus", 20.0, theme::accent())),
        )
        .child(div().text_size(px(13.5)).font_semibold().child("New project"))
}

fn project_card(idx: usize, p: Project, cx: &mut Context<StudioApp>) -> impl IntoElement + use<> {
    let id = p.id.clone();
    let mono = p.mono.clone();
    // Wrapper owns the width; the clickable body + the action overlay are SIBLINGS,
    // so tapping rename/delete never propagates into the body's open_project click.
    let actions_id = p.id.clone();
    div()
        .id(("proj-wrap", idx))
        .relative()
        .w(px(CARD_W))
        // Body first, action overlay second — the later sibling paints on top and
        // is hit-tested first, so the buttons win the click over the body.
        .child(project_card_body(idx, p, mono, id, cx))
        .child(card_actions(idx, actions_id, cx))
}

/// Hover-revealed rename/delete buttons pinned to the card's top-right corner.
fn card_actions(idx: usize, id: SharedString, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let id_ren = id.clone();
    h_flex()
        // `occlude` makes these buttons opaque to mouse events, so a click on them
        // never falls through to the card body's open_project handler underneath.
        .occlude()
        .absolute()
        .top(px(9.0))
        .right(px(9.0))
        .gap(px(6.0))
        .child(action_btn(("proj-rename", idx), "pencil", theme::text_soft(), cx.listener(move |a, _, window, cx| a.request_rename(id_ren.clone(), window, cx))))
        .child(action_btn(("proj-delete", idx), "trash", theme::danger(), cx.listener(move |a, _, _, cx| a.request_delete(id.clone(), cx))))
}

fn action_btn(id: (&'static str, usize), ic: &'static str, color: gpui::Hsla, on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> impl IntoElement {
    div()
        .id(id)
        .size(px(26.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .bg(theme::black(0.55))
        .border_1()
        .border_color(theme::line_faint())
        .cursor_pointer()
        .hover(|s| s.bg(theme::black(0.78)))
        .child(icon(ic, 13.0, color))
        .on_click(on_click)
}

fn project_card_body(idx: usize, p: Project, mono: SharedString, id: SharedString, cx: &mut Context<StudioApp>) -> impl IntoElement {
    v_flex()
        .id(("proj", idx))
        .w_full()
        .min_h(px(168.0))
        .rounded(px(theme::RADIUS_LG))
        .border_1()
        .border_color(theme::line())
        .bg(theme::bg_panel())
        .overflow_hidden()
        .cursor_pointer()
        .text_color(theme::text_body())
        .hover(|s| s.border_color(theme::line_bright()).bg(theme::bg_raised()))
        .on_click(cx.listener(move |a, _, _, cx| a.open_project(id.clone(), cx)))
        .child(
            // tinted tile with the monogram + a type badge
            div()
                .relative()
                .h(px(84.0))
                .flex()
                .items_center()
                .justify_center()
                .bg(p.tone.tint())
                .rounded_tl(px(theme::RADIUS_LG))
                .rounded_tr(px(theme::RADIUS_LG))
                .border_b_1()
                .border_color(theme::line_faint())
                .child(
                    div()
                        .size(px(46.0))
                        .rounded(px(13.0))
                        .bg(theme::accent_grad())
                        .flex()
                        .items_center()
                        .justify_center()
                        .font_family(theme::FONT_DISPLAY)
                        .font_bold()
                        .text_size(px(20.0))
                        .text_color(theme::accent_contrast())
                        .child(mono),
                )
                .child(
                    div()
                        .absolute()
                        .top(px(10.0))
                        .right(px(10.0))
                        .px(px(8.0))
                        .py(px(3.0))
                        .rounded_full()
                        .bg(theme::black(0.5))
                        .text_size(px(10.0))
                        .font_bold()
                        .text_color(p.tone.color())
                        .child(p.kind.type_label()),
                ),
        )
        .child(
            v_flex()
                .flex_1()
                .px(px(15.0))
                .pt(px(13.0))
                .pb(px(15.0))
                .child(div().text_size(px(14.5)).font_semibold().text_color(theme::text_strong()).child(p.name.clone()))
                .child(div().mt(px(3.0)).text_size(px(12.0)).text_color(theme::text_caption()).child(p.sub.clone()))
                .child(div().flex_1())
                .child(
                    h_flex()
                        .mt(px(12.0))
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            h_flex()
                                .items_center()
                                .gap(px(5.0))
                                .text_size(px(11.0))
                                .font_semibold()
                                .text_color(p.status.color())
                                .child(div().size(px(6.0)).rounded_full().bg(p.status.color()))
                                .child(p.status.label()),
                        )
                        .child(div().flex_1())
                        .child(div().text_size(px(11.0)).text_color(theme::text_faint()).child(p.updated.clone())),
                ),
        )
}
