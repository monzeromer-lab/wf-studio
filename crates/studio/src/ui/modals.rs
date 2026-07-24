//! The centered modal layer: New-project, Exit, Swap-design-system, Compile
//! log, Publish, Share, Settings, Version history, and Profile.

use gpui::{AnyElement, App, ClickEvent, Context, ElementId, Hsla, SharedString, Window, deferred, div, prelude::*, px};
use gpui_component::{StyledExt, h_flex, input::Input, v_flex};

use crate::app::StudioApp;
use crate::state::{ConnMode, ExportKind, LinkAccess, Modal, PROVIDERS, ProjectKind, PublishTab, SettingsTab, ShareMenu, ShareRole, compile_log};
use crate::theme;
use crate::ui::widgets::{Btn, Tone, avatar, icon};

pub fn render(app: &StudioApp, _window: &mut Window, cx: &mut Context<StudioApp>) -> AnyElement {
    let Some(modal) = app.modal else {
        return div().into_any_element();
    };
    let body = match modal {
        Modal::NewProject => new_project(app, cx).into_any_element(),
        Modal::Exit => exit(cx).into_any_element(),
        Modal::SwapDs => swap_ds(app, cx).into_any_element(),
        Modal::Compile => compile(cx).into_any_element(),
        Modal::Publish => publish(app, cx).into_any_element(),
        Modal::Share => share(app, cx).into_any_element(),
        Modal::Settings => settings(app, cx).into_any_element(),
        Modal::History => history(app, cx).into_any_element(),
        Modal::Profile => profile(cx).into_any_element(),
    };
    scrim(body, cx)
}

// ── shared shell ──────────────────────────────────────────────────────────────
fn scrim(dialog: impl IntoElement, cx: &mut Context<StudioApp>) -> AnyElement {
    div()
        .absolute()
        .inset_0()
        .child(div().id("modal-scrim").absolute().inset_0().bg(theme::hexa(0x040508a8)).on_click(cx.listener(|a, _, _, cx| a.close_modal(cx))))
        .child(div().absolute().inset_0().flex().items_center().justify_center().p(px(24.0)).child(dialog))
        .into_any_element()
}

fn card(width: f32) -> gpui::Stateful<gpui::Div> {
    v_flex()
        .id("modal-card")
        .occlude()
        .w(px(width))
        .max_w_full()
        .bg(theme::bg_panel())
        .border_1()
        .border_color(theme::line_strong())
        .rounded(px(theme::RADIUS_3XL))
        .shadow(theme::shadow_modal())
        .overflow_hidden()
}

fn close_btn(cx: &mut Context<StudioApp>) -> impl IntoElement {
    div()
        .id("modal-close")
        .size(px(30.0))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(8.0))
        .text_color(theme::icon_color())
        .cursor_pointer()
        .hover(|s| s.bg(theme::bg_hover()).text_color(theme::text_strong()))
        .child(icon("close", 16.0, theme::icon_color()))
        .on_click(cx.listener(|a, _, _, cx| a.close_modal(cx)))
}

fn heading(title: &'static str, sub: &'static str) -> impl IntoElement {
    v_flex()
        .flex_1()
        .min_w_0()
        .child(div().font_family(theme::FONT_DISPLAY).text_size(px(19.0)).font_semibold().text_color(theme::text_strong()).child(title))
        .child(div().mt(px(3.0)).text_size(px(13.0)).text_color(theme::text_muted()).child(sub))
}

fn eyebrow(text: &'static str) -> impl IntoElement {
    div().text_size(px(11.5)).font_bold().text_color(theme::text_caption()).mb(px(10.0)).child(text)
}

fn tile(icon_name: &'static str, bg: Hsla, fg: Hsla) -> impl IntoElement {
    div().size(px(38.0)).flex_none().rounded(px(11.0)).bg(bg).flex().items_center().justify_center().child(icon(icon_name, 20.0, fg))
}

fn toggle_switch(id: &'static str, on: bool, on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> impl IntoElement {
    let mut t = h_flex().id(id).w(px(42.0)).h(px(24.0)).p(px(3.0)).rounded_full().items_center().flex_none().cursor_pointer().on_click(on_click);
    t = if on { t.bg(theme::accent()).justify_end() } else { t.bg(theme::bg_hover()) };
    t.child(div().size(px(18.0)).rounded_full().bg(theme::white(1.0)))
}

// ── New project ──────────────────────────────────────────────────────────────
fn new_project(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let web = app.new_type == ProjectKind::Website;
    card(560.0)
        .child(h_flex().w_full().px(px(24.0)).pt(px(22.0)).items_start().gap(px(13.0)).child(heading("Start a new project", "What are you building?")).child(close_btn(cx)))
        .child(
            h_flex()
                .w_full()
                .px(px(24.0))
                .py(px(20.0))
                .gap(px(12.0))
                .child(type_card("nt-web", "globe-big", theme::accent_tint(), theme::accent(), "Design project", "A website, landing page, or app \u{2014} built by conversation.", web, cx.listener(|a, _, _, cx| a.set_new_type(ProjectKind::Website, cx))))
                .child(type_card("nt-sys", "boxes", theme::violet_tint(), theme::violet_soft(), "Design system", "A reusable kit of tokens, type and components your projects build from.", !web, cx.listener(|a, _, _, cx| a.set_new_type(ProjectKind::System, cx)))),
        )
        .child(
            h_flex()
                .w_full()
                .px(px(24.0))
                .pb(px(22.0))
                .gap(px(10.0))
                .justify_end()
                .child(Btn::secondary("Cancel").render("np-cancel", cx.listener(|a, _, _, cx| a.close_modal(cx))))
                .child(Btn::primary("Create").icon_right("arrow-right").render("np-create", cx.listener(|a, _, window, cx| a.create_project(window, cx)))),
        )
}

fn type_card(id: &'static str, icon_name: &'static str, icon_bg: Hsla, icon_fg: Hsla, title: &'static str, desc: &'static str, selected: bool, on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> impl IntoElement {
    v_flex()
        .id(id)
        .relative()
        .flex_1()
        .min_w_0()
        .p(px(18.0))
        .rounded(px(theme::RADIUS_LG))
        .border_1()
        .border_color(if selected { theme::accent_ring() } else { theme::line() })
        .bg(if selected { theme::accent_tint() } else { theme::bg_raised() })
        .cursor_pointer()
        .on_click(on_click)
        .child(div().size(px(42.0)).rounded(px(12.0)).bg(icon_bg).flex().items_center().justify_center().child(icon(icon_name, 24.0, icon_fg)))
        .child(div().mt(px(13.0)).text_size(px(15.0)).font_semibold().text_color(theme::text_strong()).child(title))
        .child(div().mt(px(5.0)).text_size(px(12.5)).text_color(theme::text_caption()).line_height(px(18.0)).child(desc))
        .when(selected, |d| d.child(div().absolute().top(px(16.0)).right(px(16.0)).size(px(20.0)).rounded_full().bg(theme::accent()).flex().items_center().justify_center().child(icon("check", 12.0, theme::accent_contrast()))))
}

// ── Exit ──────────────────────────────────────────────────────────────────────
fn exit(cx: &mut Context<StudioApp>) -> impl IntoElement {
    card(400.0).child(
        v_flex()
            .p(px(24.0))
            .child(h_flex().items_center().gap(px(12.0)).child(tile("check-circle", theme::success_tint(), theme::success())).child(div().font_family(theme::FONT_DISPLAY).text_size(px(17.0)).font_semibold().text_color(theme::text_strong()).child("Leave this project?")))
            .child(div().mt(px(12.0)).text_size(px(13.0)).text_color(theme::text_muted()).line_height(px(21.0)).child("Everything is saved automatically to version history. You can pick up right where you left off."))
            .child(
                h_flex()
                    .w_full()
                    .mt(px(20.0))
                    .gap(px(10.0))
                    .child(Btn::secondary("Stay").grow().render("exit-stay", cx.listener(|a, _, _, cx| a.close_modal(cx))))
                    .child(Btn::primary("Back to projects").grow().icon("arrow-left").render("exit-go", cx.listener(|a, _, _, cx| a.confirm_exit(cx)))),
            ),
    )
}

// ── Swap design system ────────────────────────────────────────────────────────
fn swap_ds(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let from = app.applied_ds.as_ref().and_then(|id| app.projects.iter().find(|p| &p.id == id)).map(|p| p.name.clone()).unwrap_or_else(|| "none".into());
    let to = match &app.pending_ds {
        Some(id) => app.projects.iter().find(|p| &p.id == id).map(|p| p.name.clone()).unwrap_or_else(|| "another system".into()),
        None => "no design system".into(),
    };
    card(420.0).child(
        v_flex()
            .p(px(24.0))
            .child(h_flex().items_center().gap(px(12.0)).child(tile("shield", theme::warning_tint(), theme::warning())).child(div().font_family(theme::FONT_DISPLAY).text_size(px(17.0)).font_semibold().text_color(theme::text_strong()).child("Change design system?")))
            .child(
                h_flex()
                    .mt(px(12.0))
                    .flex_wrap()
                    .gap(px(4.0))
                    .text_size(px(13.0))
                    .text_color(theme::text_muted())
                    .child("Switching from")
                    .child(div().font_semibold().text_color(theme::text_strong()).child(from))
                    .child("to")
                    .child(div().font_semibold().text_color(theme::text_strong()).child(to))
                    .child("will restyle every component on this page. Your content stays; the look changes."),
            )
            .child(
                h_flex()
                    .w_full()
                    .mt(px(20.0))
                    .gap(px(10.0))
                    .child(Btn::secondary("Cancel").grow().render("sw-cancel", cx.listener(|a, _, _, cx| a.cancel_swap_ds(cx))))
                    .child(Btn::primary("Switch & restyle").grow().icon("boxes").render("sw-go", cx.listener(|a, _, _, cx| a.confirm_swap_ds(cx)))),
            ),
    )
}

// ── Compile log ───────────────────────────────────────────────────────────────
fn compile(cx: &mut Context<StudioApp>) -> impl IntoElement {
    card(500.0)
        .child(h_flex().w_full().px(px(24.0)).pt(px(22.0)).items_start().gap(px(13.0)).child(tile("check-circle", theme::bg_raised(), theme::success())).child(heading("Compile log", "Every build, its timing, and any errors WebFluent hit or healed.")).child(close_btn(cx)))
        .child(v_flex().px(px(24.0)).pt(px(16.0)).pb(px(22.0)).gap(px(10.0)).children(compile_log().into_iter().map(|c| {
            let note_color = match c.note_tone {
                crate::state::Tone::Err => theme::danger(),
                crate::state::Tone::Warn => theme::warning(),
                _ => theme::text_muted(),
            };
            v_flex()
                .p(px(14.0))
                .rounded(px(12.0))
                .bg(theme::bg_raised())
                .border_1()
                .border_color(theme::line())
                .child(
                    h_flex()
                        .items_center()
                        .gap(px(10.0))
                        .child(icon(c.icon, 16.0, (c.dot)()))
                        .child(div().flex_1().text_size(px(13.5)).font_semibold().text_color(theme::text_strong()).child(c.title))
                        .child(div().font_family(theme::FONT_MONO).text_size(px(11.5)).text_color(theme::text_caption()).child(c.ms))
                        .child(div().text_size(px(11.5)).text_color(theme::text_faint()).child(c.time)),
                )
                .child(div().mt(px(8.0)).text_size(px(12.5)).text_color(note_color).line_height(px(19.0)).child(c.note))
                .when(c.detail.is_some(), |d| d.child(div().mt(px(8.0)).px(px(11.0)).py(px(9.0)).rounded(px(8.0)).bg(theme::bg_sunken()).border_1().border_color(theme::line_faint()).font_family(theme::FONT_MONO).text_size(px(11.5)).text_color(theme::text_caption()).line_height(px(18.0)).child(c.detail.unwrap_or_default())))
        })))
}

// ── Publish ───────────────────────────────────────────────────────────────────
fn publish(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let deploy = app.publish_tab == PublishTab::Deploy;
    let mut c = card(480.0)
        .child(h_flex().w_full().px(px(24.0)).pt(px(22.0)).items_start().gap(px(13.0)).child(div().size(px(38.0)).flex_none().rounded(px(11.0)).bg(theme::accent_grad()).shadow(theme::glow_violet()).flex().items_center().justify_center().child(icon("cloud", 20.0, theme::accent_contrast()))).child(heading("Publish Layali", "Put your site online, or export the code to host anywhere.")).child(close_btn(cx)))
        .child(
            div().w_full().px(px(24.0)).pt(px(18.0)).child(
                h_flex()
                    .w_full()
                    .gap(px(4.0))
                    .p(px(4.0))
                    .bg(theme::bg_sunken())
                    .rounded(px(11.0))
                    .child(pill_tab("pub-deploy", "Deploy", deploy, cx.listener(|a, _, _, cx| a.set_publish_tab(PublishTab::Deploy, cx))))
                    .child(pill_tab("pub-export", "Export", !deploy, cx.listener(|a, _, _, cx| a.set_publish_tab(PublishTab::Export, cx)))),
            ),
        );
    if deploy {
        if app.published {
            c = c.child(
                v_flex()
                    .px(px(24.0))
                    .py(px(22.0))
                    .child(
                        v_flex()
                            .items_center()
                            .p(px(22.0))
                            .rounded(px(16.0))
                            .bg(theme::success_tint())
                            .border_1()
                            .border_color(theme::hexa(0x5CCB9A4d))
                            .child(div().size(px(44.0)).rounded_full().bg(theme::success()).flex().items_center().justify_center().child(icon("check", 20.0, theme::bg_void())))
                            .child(div().mt(px(12.0)).font_family(theme::FONT_DISPLAY).text_size(px(17.0)).font_semibold().text_color(theme::text_strong()).child("You\u{2019}re live"))
                            .child(h_flex().mt(px(12.0)).items_center().gap(px(10.0)).px(px(14.0)).py(px(9.0)).rounded(px(10.0)).bg(theme::bg_sunken()).border_1().border_color(theme::line()).child(icon("globe", 15.0, theme::success())).child(div().font_family(theme::FONT_MONO).text_size(px(13.0)).text_color(theme::text_soft()).child("layali.webfluent.app")))
                            .child(div().mt(px(12.0)).text_size(px(11.5)).text_color(theme::text_caption()).child("Published just now \u{b7} free plan")),
                    )
                    .child(h_flex().w_full().mt(px(20.0)).gap(px(10.0)).justify_end().child(Btn::secondary("Done").render("pub-done", cx.listener(|a, _, _, cx| a.close_modal(cx)))).child(Btn::primary("Copy link").icon("globe").render("pub-copy", cx.listener(|a, _, _, cx| a.copy_link(cx))))),
            );
        } else if app.deploying {
            c = c.child(v_flex().items_center().px(px(24.0)).py(px(34.0)).child(icon("loader", 30.0, theme::accent())).child(div().mt(px(14.0)).text_size(px(14.0)).font_semibold().text_color(theme::text_strong()).child("Deploying to the edge\u{2026}")).child(div().mt(px(5.0)).font_family(theme::FONT_MONO).text_size(px(12.0)).text_color(theme::text_muted()).child("building \u{b7} uploading \u{b7} warming CDN")));
        } else {
            c = c
                .child(
                    v_flex()
                        .px(px(24.0))
                        .py(px(22.0))
                        .child(eyebrow("SITE ADDRESS"))
                        .child(h_flex().items_center().rounded(px(10.0)).border_1().border_color(theme::line_strong()).bg(theme::bg_sunken()).overflow_hidden().child(div().flex_1().px(px(14.0)).py(px(12.0)).font_family(theme::FONT_MONO).text_size(px(14.0)).text_color(theme::text_strong()).child("layali")).child(div().px(px(14.0)).py(px(12.0)).bg(theme::bg_panel()).border_l_1().border_color(theme::line()).font_family(theme::FONT_MONO).text_size(px(14.0)).text_color(theme::text_caption()).child(".webfluent.app")))
                        .child(h_flex().mt(px(12.0)).items_center().gap(px(8.0)).text_size(px(12.5)).text_color(theme::text_muted()).child(icon("check-circle", 15.0, theme::success())).child("Free plan \u{b7} 1 live site \u{b7} WebFluent subdomain")),
                )
                .child(h_flex().w_full().px(px(24.0)).pb(px(22.0)).gap(px(10.0)).justify_end().child(Btn::secondary("Cancel").render("pub-cancel", cx.listener(|a, _, _, cx| a.close_modal(cx)))).child(Btn::primary("Publish").icon("cloud").render("pub-go", cx.listener(|a, _, _, cx| a.do_publish(cx)))));
        }
    } else {
        let stat = app.export_kind == ExportKind::Static;
        c = c
            .child(
                v_flex()
                    .px(px(24.0))
                    .py(px(22.0))
                    .gap(px(10.0))
                    .child(export_row("ex-static", "download", "Static site", "Ready-to-host HTML, CSS & JS", stat, cx.listener(|a, _, _, cx| a.set_export_kind(ExportKind::Static, cx))))
                    .child(export_row("ex-full", "layers", "Full project + assets", "Everything, including images", !stat, cx.listener(|a, _, _, cx| a.set_export_kind(ExportKind::Full, cx))))
                    .child(h_flex().mt(px(4.0)).items_center().gap(px(8.0)).text_size(px(12.0)).text_color(theme::text_caption()).child(icon("shield", 14.0, theme::text_caption())).child("Standard web code only \u{2014} no framework lock-in.")),
            )
            .child(h_flex().w_full().px(px(24.0)).pb(px(22.0)).gap(px(10.0)).justify_end().child(Btn::secondary("Cancel").render("ex-cancel", cx.listener(|a, _, _, cx| a.close_modal(cx)))).child(Btn::primary("Download").icon("download").render("ex-go", cx.listener(|a, _, _, cx| a.close_modal(cx)))));
    }
    c
}

fn export_row(id: &'static str, icon_name: &'static str, title: &'static str, sub: &'static str, on: bool, on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> impl IntoElement {
    h_flex()
        .id(id)
        .items_center()
        .gap(px(12.0))
        .w_full()
        .p(px(15.0))
        .rounded(px(12.0))
        .border_1()
        .border_color(if on { theme::accent_ring() } else { theme::line() })
        .bg(if on { theme::accent_tint() } else { theme::bg_raised() })
        .cursor_pointer()
        .on_click(on_click)
        .child(icon(icon_name, 18.0, if on { theme::accent() } else { theme::text_soft() }))
        .child(v_flex().flex_1().child(div().text_size(px(13.5)).font_semibold().text_color(theme::text_strong()).child(title)).child(div().text_size(px(12.0)).text_color(theme::text_muted()).child(sub)))
        .child(div().size(px(18.0)).rounded_full().border_1().border_color(if on { theme::accent() } else { theme::line_bright() }).bg(if on { theme::accent() } else { gpui::transparent_black() }).flex().items_center().justify_center().when(on, |d| d.child(icon("check", 12.0, theme::accent_contrast()))))
}

fn pill_tab(id: &'static str, label: &'static str, active: bool, on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> impl IntoElement {
    let mut b = div().id(id).flex_1().h(px(32.0)).flex().items_center().justify_center().rounded(px(theme::RADIUS_SM)).text_size(px(13.0)).font_semibold().cursor_pointer().on_click(on_click);
    if active {
        b = b.bg(theme::accent()).text_color(theme::accent_contrast());
    } else {
        b = b.text_color(theme::text_soft());
    }
    b.child(label)
}

// ── Share ─────────────────────────────────────────────────────────────────────
fn share(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let menu = app.share_menu;
    card(470.0)
        .relative()
        .when(menu.is_some(), |c| {
            c.child(deferred(div().id("share-backdrop").occlude().absolute().inset_0().on_click(cx.listener(|a, _, _, cx| a.close_share_menu(cx)))))
        })
        .child(h_flex().w_full().px(px(24.0)).pt(px(22.0)).items_start().gap(px(13.0)).child(tile("share", theme::bg_raised(), theme::accent())).child(heading("Share Layali", "Invite people to edit or view in real time.")).child(close_btn(cx)))
        .child(
            v_flex()
                .px(px(24.0))
                .py(px(20.0))
                // invite row
                .child(
                    h_flex()
                        .w_full()
                        .gap(px(8.0))
                        .child(h_flex().flex_1().min_w_0().items_center().gap(px(8.0)).px(px(12.0)).rounded(px(10.0)).border_1().border_color(theme::line_strong()).bg(theme::bg_sunken()).child(icon("user", 15.0, theme::icon_color())).child(div().flex_1().text_size(px(13.5)).text_color(theme::text_caption()).py(px(11.0)).child("name@email.com")))
                        .child(
                            div()
                                .relative()
                                .child(
                                    h_flex()
                                        .id("share-role")
                                        .items_center()
                                        .gap(px(6.0))
                                        .px(px(12.0))
                                        .py(px(11.0))
                                        .rounded(px(10.0))
                                        .border_1()
                                        .border_color(theme::line_strong())
                                        .bg(theme::bg_raised())
                                        .text_size(px(12.5))
                                        .font_semibold()
                                        .text_color(theme::text_soft())
                                        .cursor_pointer()
                                        .hover(|s| s.border_color(theme::line_bright()))
                                        .child(app.share_role.label())
                                        .child(icon("chevron-down", 14.0, theme::text_muted()))
                                        .on_click(cx.listener(|a, _, _, cx| a.toggle_share_menu(ShareMenu::InviteRole, cx))),
                                )
                                .when(menu == Some(ShareMenu::InviteRole), |d| d.child(role_dropdown("sr", app.share_role, cx, |a, r, cx| a.set_share_role(r, cx)))),
                        )
                        .child(Btn::primary("Invite").render("share-invite", cx.listener(|a, _, _, cx| a.invite_sent(cx)))),
                )
                .child(div().mt(px(20.0)).child(eyebrow("PEOPLE WITH ACCESS")))
                .child(v_flex().children(crate::state::COLLABORATORS.iter().enumerate().map(|(i, c)| collaborator_row(app, i, c, cx))))
                // link access
                .child(
                    v_flex()
                        .mt(px(18.0))
                        .pt(px(18.0))
                        .border_t_1()
                        .border_color(theme::line_faint())
                        .child(
                            div()
                                .relative()
                                .child(
                                    h_flex()
                                        .id("link-access")
                                        .items_center()
                                        .gap(px(11.0))
                                        .cursor_pointer()
                                        .on_click(cx.listener(|a, _, _, cx| a.toggle_share_menu(ShareMenu::LinkAccess, cx)))
                                        .child(div().size(px(34.0)).flex_none().rounded_full().bg(theme::bg_raised()).flex().items_center().justify_center().child(icon("globe", 15.0, theme::text_soft())))
                                        .child(v_flex().flex_1().child(div().text_size(px(13.0)).font_semibold().text_color(theme::text_strong()).child(app.link_access.label())).child(div().text_size(px(11.5)).text_color(theme::text_caption()).child(app.link_access.desc())))
                                        .child(icon("chevron-down", 15.0, theme::text_caption())),
                                )
                                .when(menu == Some(ShareMenu::LinkAccess), |d| d.child(link_dropdown(app, cx))),
                        )
                        .child(h_flex().mt(px(10.0)).items_center().gap(px(8.0)).child(h_flex().flex_1().min_w_0().items_center().gap(px(8.0)).px(px(12.0)).py(px(10.0)).rounded(px(10.0)).border_1().border_color(theme::line()).bg(theme::bg_sunken()).child(icon("link", 15.0, theme::icon_color())).child(div().flex_1().min_w_0().font_family(theme::FONT_MONO).text_size(px(12.5)).text_color(theme::text_soft()).overflow_hidden().child("webfluent.app/s/layali-x9f2"))).child(Btn::secondary("Copy").icon("copy").render("share-copy", cx.listener(|a, _, _, cx| a.copy_link(cx))))),
                ),
        )
}

/// A role picker menu (`Can edit` / `Can view`) anchored under its chip, drawn
/// on top via `deferred` so it isn't clipped by the rows below it.
fn role_dropdown(prefix: &'static str, current: ShareRole, cx: &mut Context<StudioApp>, apply: impl Fn(&mut StudioApp, ShareRole, &mut Context<StudioApp>) + Copy + 'static) -> impl IntoElement {
    deferred(
        v_flex()
            .id(prefix)
            .occlude()
            .absolute()
            .top_full()
            .right_0()
            .mt(px(6.0))
            .min_w(px(150.0))
            .bg(theme::bg_raised())
            .border_1()
            .border_color(theme::line_strong())
            .rounded(px(10.0))
            .shadow(theme::shadow_pop())
            .p(px(6.0))
            .child(role_option((prefix, 0usize), "Can edit", current == ShareRole::Edit, cx.listener(move |a, _, _, cx| apply(a, ShareRole::Edit, cx))))
            .child(role_option((prefix, 1usize), "Can view", current == ShareRole::View, cx.listener(move |a, _, _, cx| apply(a, ShareRole::View, cx)))),
    )
    .with_priority(1)
}

fn role_option(id: impl Into<ElementId>, label: &'static str, selected: bool, on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> impl IntoElement {
    let mut row = h_flex()
        .id(id)
        .w_full()
        .items_center()
        .gap(px(8.0))
        .px(px(10.0))
        .py(px(8.0))
        .rounded(px(7.0))
        .cursor_pointer()
        .text_size(px(12.5))
        .font_semibold()
        .on_click(on_click)
        .child(div().flex_1().child(label));
    if selected {
        row = row.bg(theme::accent_tint()).text_color(theme::text_strong()).child(icon("check", 14.0, theme::accent()));
    } else {
        row = row.text_color(theme::text_soft()).hover(|s| s.bg(theme::bg_hover()));
    }
    row
}

/// The link-access picker, opening upward (`bottom_full`) since it sits low.
fn link_dropdown(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    deferred(
        v_flex()
            .id("la-menu")
            .occlude()
            .absolute()
            .bottom_full()
            .left_0()
            .mb(px(6.0))
            .min_w(px(264.0))
            .bg(theme::bg_raised())
            .border_1()
            .border_color(theme::line_strong())
            .rounded(px(10.0))
            .shadow(theme::shadow_pop())
            .p(px(6.0))
            .child(link_option("la-r", "Restricted", "Only invited people can open this", app.link_access == LinkAccess::Restricted, cx.listener(|a, _, _, cx| a.set_link_access(LinkAccess::Restricted, cx))))
            .child(link_option("la-a", "Anyone with the link", "Can view the published site", app.link_access == LinkAccess::Anyone, cx.listener(|a, _, _, cx| a.set_link_access(LinkAccess::Anyone, cx)))),
    )
    .with_priority(1)
}

fn link_option(id: &'static str, title: &'static str, desc: &'static str, selected: bool, on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> impl IntoElement {
    let mut row = h_flex()
        .id(id)
        .w_full()
        .items_start()
        .gap(px(8.0))
        .px(px(10.0))
        .py(px(8.0))
        .rounded(px(7.0))
        .cursor_pointer()
        .on_click(on_click)
        .child(v_flex().flex_1().child(div().text_size(px(12.5)).font_semibold().text_color(theme::text_strong()).child(title)).child(div().mt(px(1.0)).text_size(px(11.0)).text_color(theme::text_caption()).child(desc)));
    if selected {
        row = row.child(icon("check", 14.0, theme::accent()));
    } else {
        row = row.hover(|s| s.bg(theme::bg_hover()));
    }
    row
}

fn collaborator_row(app: &StudioApp, i: usize, c: &crate::state::Collaborator, cx: &mut Context<StudioApp>) -> impl IntoElement + use<> {
    let tone = match i {
        0 => Tone::Violet,
        1 => Tone::Blue,
        _ => Tone::Teal,
    };
    let role: SharedString = if c.owner {
        "Owner".into()
    } else if i == 1 {
        app.collab_mk.label().into()
    } else {
        app.collab_ah.label().into()
    };
    h_flex()
        .items_center()
        .gap(px(11.0))
        .py(px(8.0))
        .child(avatar(c.initials, tone, false, c.online, 34.0))
        .child(v_flex().flex_1().min_w_0().child(div().text_size(px(13.5)).font_semibold().text_color(theme::text_strong()).child(c.name)).child(div().text_size(px(11.5)).text_color(theme::text_caption()).child(role.clone())))
        .when(c.owner, |r| r.child(div().text_size(px(12.0)).text_color(theme::text_caption()).pr(px(6.0)).child("Owner")))
        .when(!c.owner, |r| {
            r.child(
                div()
                    .relative()
                    .child(
                        h_flex()
                            .id(("collab", i))
                            .items_center()
                            .gap(px(4.0))
                            .px(px(9.0))
                            .py(px(5.0))
                            .rounded(px(8.0))
                            .border_1()
                            .border_color(theme::line())
                            .text_size(px(12.0))
                            .font_semibold()
                            .text_color(theme::text_soft())
                            .cursor_pointer()
                            .hover(|s| s.bg(theme::bg_hover()))
                            .child(role)
                            .child(icon("chevron-down", 14.0, theme::text_muted()))
                            .on_click(cx.listener(move |a, _, _, cx| a.toggle_share_menu(ShareMenu::Collab(i), cx))),
                    )
                    .when(app.share_menu == Some(ShareMenu::Collab(i)), |d| {
                        d.child(role_dropdown("col", if i == 1 { app.collab_mk } else { app.collab_ah }, cx, move |a, rr, cx| a.set_collab_role(i, rr, cx)))
                    }),
            )
        })
}

// ── History ───────────────────────────────────────────────────────────────────
fn history(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    // Newest first, keeping each revision's real index for restore.
    let entries: Vec<_> = app.history_entries().into_iter().enumerate().rev().collect();
    card(440.0)
        .child(h_flex().w_full().px(px(24.0)).pt(px(22.0)).items_start().gap(px(13.0)).child(tile("clock", theme::bg_raised(), theme::accent())).child(heading("Version history", "Every change you keep is saved automatically.")).child(close_btn(cx)))
        .child(v_flex().px(px(24.0)).pt(px(14.0)).pb(px(22.0)).children(entries.into_iter().map(|(i, (label, current))| {
            let dot_color = if current { theme::accent() } else { theme::text_faint() };
            h_flex()
                .gap(px(13.0))
                .py(px(12.0))
                .items_center()
                .child(div().size(px(11.0)).flex_none().rounded_full().bg(dot_color))
                .child(
                    h_flex()
                        .flex_1()
                        .items_center()
                        .gap(px(8.0))
                        .child(div().text_size(px(13.0)).font_semibold().text_color(theme::text_strong()).child(label))
                        .when(current, |d| d.child(div().px(px(7.0)).py(px(2.0)).rounded(px(5.0)).bg(theme::accent_tint()).text_color(theme::accent()).text_size(px(9.5)).font_bold().child("CURRENT"))),
                )
                .when(!current, |r| r.child(div().id(("restore", i)).px(px(12.0)).py(px(6.0)).rounded(px(8.0)).border_1().border_color(theme::line_strong()).bg(theme::bg_raised()).text_size(px(12.0)).font_semibold().text_color(theme::text_soft()).cursor_pointer().hover(|s| s.text_color(theme::text_strong())).child("Restore").on_click(cx.listener(move |a, _, _, cx| a.restore_revision(i, cx)))))
        })))
}

// ── Profile ───────────────────────────────────────────────────────────────────
fn profile(cx: &mut Context<StudioApp>) -> impl IntoElement {
    card(384.0)
        .child(
            h_flex()
                .px(px(20.0))
                .py(px(18.0))
                .items_center()
                .gap(px(13.0))
                .border_b_1()
                .border_color(theme::line_faint())
                .child(avatar("RS", Tone::Violet, true, false, 46.0))
                .child(v_flex().flex_1().min_w_0().child(h_flex().items_center().gap(px(8.0)).child(div().font_family(theme::FONT_DISPLAY).text_size(px(16.0)).font_semibold().text_color(theme::text_strong()).child("Rana Saeed")).child(div().px(px(7.0)).py(px(2.0)).rounded_full().bg(theme::accent_tint()).text_color(theme::accent()).text_size(px(9.5)).font_bold().child("ALPHA"))).child(div().mt(px(2.0)).text_size(px(12.0)).text_color(theme::text_muted()).child("rana@studio.sa")))
                .child(close_btn(cx)),
        )
        .child(
            v_flex()
                .px(px(20.0))
                .py(px(16.0))
                .child(
                    v_flex()
                        .p(px(16.0))
                        .rounded(px(theme::RADIUS_LG))
                        .bg(theme::bg_sunken())
                        .border_1()
                        .border_color(theme::line())
                        .child(h_flex().items_center().gap(px(8.0)).child(div().text_size(px(13.0)).font_semibold().text_color(theme::text_strong()).child("Free plan")).child(div().flex_1()).child(div().text_size(px(12.0)).text_color(theme::text_caption()).child("240 / 500 builds")))
                        .child(div().mt(px(10.0)).h(px(6.0)).w_full().rounded_full().bg(theme::bg_hover()).overflow_hidden().child(div().h_full().w(gpui::relative(0.48)).bg(theme::accent_grad())))
                        .child(div().mt(px(11.0)).child(Btn::primary("Upgrade to Pro").sm().full().icon("zap").render("upgrade", cx.listener(|a, _, _, cx| a.close_modal(cx))))),
                )
                .child(
                    v_flex()
                        .mt(px(14.0))
                        .child(profile_row("pf-account", "user", "Account & providers", Some("chevron-right"), None, cx.listener(|a, _, _, cx| a.open_modal(Modal::Settings, cx))))
                        .child(profile_row("pf-billing", "credit-card", "Billing & usage", Some("chevron-right"), None, cx.listener(|a, _, _, cx| a.close_modal(cx))))
                        .child(profile_row("pf-appearance", "palette", "Appearance", None, Some("Dark"), cx.listener(|a, _, _, cx| a.close_modal(cx)))),
                )
                .child(div().h(px(1.0)).my(px(12.0)).bg(theme::line_faint()))
                .child(
                    h_flex()
                        .id("pf-signout")
                        .items_center()
                        .gap(px(12.0))
                        .w_full()
                        .px(px(12.0))
                        .py(px(11.0))
                        .rounded(px(10.0))
                        .text_size(px(13.5))
                        .font_semibold()
                        .text_color(theme::danger())
                        .cursor_pointer()
                        .hover(|s| s.bg(theme::danger_tint()))
                        .child(icon("logout", 15.0, theme::danger()))
                        .child("Sign out")
                        .on_click(cx.listener(|a, _, window, cx| a.sign_out(window, cx))),
                ),
        )
}

fn profile_row(id: &'static str, icon_name: &'static str, label: &'static str, chevron: Option<&'static str>, trailing: Option<&'static str>, on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> impl IntoElement {
    h_flex()
        .id(id)
        .items_center()
        .gap(px(12.0))
        .w_full()
        .px(px(12.0))
        .py(px(11.0))
        .rounded(px(10.0))
        .text_size(px(13.5))
        .text_color(theme::text_soft())
        .cursor_pointer()
        .hover(|s| s.bg(theme::bg_hover()).text_color(theme::text_strong()))
        .child(icon(icon_name, 15.0, theme::icon_color()))
        .child(div().flex_1().child(label))
        .when_some(trailing, |r, t| r.child(div().text_size(px(12.0)).text_color(theme::text_caption()).child(t)))
        .when_some(chevron, |r, c| r.child(icon(c, 15.0, theme::text_caption())))
        .on_click(on_click)
}

// ── Settings ──────────────────────────────────────────────────────────────────
fn settings(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    card(680.0)
        .h(px(560.0))
        .max_h(gpui::relative(0.9))
        .child(h_flex().px(px(24.0)).py(px(20.0)).items_center().gap(px(10.0)).border_b_1().border_color(theme::line_faint()).child(icon("settings", 18.0, theme::accent())).child(div().flex_1().font_family(theme::FONT_DISPLAY).text_size(px(18.0)).font_semibold().text_color(theme::text_strong()).child("Settings")).child(close_btn(cx)))
        .child(
            h_flex()
                .flex_1()
                .min_h_0()
                .child(
                    v_flex()
                        .w(px(210.0))
                        .flex_none()
                        .h_full()
                        .p(px(16.0))
                        .gap(px(4.0))
                        .border_r_1()
                        .border_color(theme::line_faint())
                        .child(settings_tab("st-prov", "key", "Providers & keys", app.settings_tab == SettingsTab::Providers, cx.listener(|a, _, _, cx| a.set_settings_tab(SettingsTab::Providers, cx))))
                        .child(settings_tab("st-mcp", "server", "MCP servers", app.settings_tab == SettingsTab::Mcp, cx.listener(|a, _, _, cx| a.set_settings_tab(SettingsTab::Mcp, cx))))
                        .child(settings_tab("st-adv", "sliders", "Advanced", app.settings_tab == SettingsTab::Advanced, cx.listener(|a, _, _, cx| a.set_settings_tab(SettingsTab::Advanced, cx)))),
                )
                .child(v_flex().id("settings-body").flex_1().min_w_0().h_full().overflow_y_scroll().px(px(24.0)).py(px(20.0)).child(match app.settings_tab {
                    SettingsTab::Providers => settings_providers(app, cx).into_any_element(),
                    SettingsTab::Mcp => settings_mcp(app, cx).into_any_element(),
                    SettingsTab::Advanced => settings_advanced(app, cx).into_any_element(),
                })),
        )
        .child(h_flex().px(px(24.0)).py(px(14.0)).border_t_1().border_color(theme::line_faint()).justify_end().child(Btn::primary("Done").render("st-done", cx.listener(|a, _, _, cx| a.close_modal(cx)))))
}

fn settings_tab(id: &'static str, icon_name: &'static str, label: &'static str, active: bool, on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> impl IntoElement {
    let mut b = h_flex().id(id).items_center().gap(px(10.0)).w_full().px(px(12.0)).py(px(10.0)).rounded(px(10.0)).text_size(px(13.0)).font_semibold().cursor_pointer().on_click(on_click);
    if active {
        b = b.bg(theme::accent_tint()).text_color(theme::text_strong());
    } else {
        b = b.text_color(theme::text_soft()).hover(|s| s.bg(theme::bg_hover()));
    }
    b.child(icon(icon_name, 15.0, if active { theme::accent() } else { theme::icon_color() })).child(label)
}

fn settings_providers(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    let key = app.conn_mode == ConnMode::Key;
    let has_key = app.key_text(cx).trim().len() >= 12;
    let mut body = v_flex()
        .child(eyebrow("CONNECTION MODE"))
        .child(
            h_flex()
                .w_full()
                .gap(px(4.0))
                .p(px(4.0))
                .mb(px(20.0))
                .bg(theme::bg_sunken())
                .rounded(px(11.0))
                .child(pill_tab("cm-key", "Your own key", key, cx.listener(|a, _, _, cx| a.set_conn_mode(ConnMode::Key, cx))))
                .child(pill_tab("cm-acp", "Connect agent \u{b7} ACP", !key, cx.listener(|a, _, _, cx| a.set_conn_mode(ConnMode::Acp, cx)))),
        );
    if key {
        body = body
            .child(eyebrow("PROVIDER"))
            .child(h_flex().flex_wrap().gap(px(8.0)).mb(px(20.0)).children(PROVIDERS.iter().enumerate().map(|(i, p)| {
                let id = p.id;
                let active = app.provider == p.id;
                div().w(px(200.0)).child(
                    h_flex()
                        .id(("sp", i))
                        .w_full()
                        .items_center()
                        .gap(px(10.0))
                        .p(px(10.0))
                        .rounded(px(11.0))
                        .border_1()
                        .border_color(if active { theme::accent_ring() } else { theme::line() })
                        .bg(if active { theme::accent_tint() } else { theme::bg_raised() })
                        .cursor_pointer()
                        .on_click(cx.listener(move |a, _, window, cx| a.pick_provider(id, window, cx)))
                        .child(div().size(px(28.0)).rounded(px(8.0)).bg(theme::hex(p.mono_bg)).flex().items_center().justify_center().font_family(theme::FONT_DISPLAY).font_bold().text_size(px(13.0)).text_color(theme::white(1.0)).child(p.mono))
                        .child(v_flex().flex_1().min_w_0().child(div().text_size(px(13.0)).font_semibold().text_color(theme::text_strong()).child(p.name)).child(h_flex().items_center().gap(px(5.0)).text_size(px(10.5)).text_color(theme::text_caption()).child(div().size(px(5.0)).rounded_full().bg(if active && has_key { theme::success() } else { theme::text_faint() })).child(if active && has_key { "Key saved" } else { "No key" })))
                        .when(active, |d| d.child(icon("check", 15.0, theme::accent()))),
                )
            })))
            .child(eyebrow("API KEY"))
            .child(h_flex().items_center().gap(px(8.0)).px(px(12.0)).rounded(px(10.0)).border_1().border_color(theme::line_strong()).bg(theme::bg_sunken()).child(icon("lock", 15.0, if has_key { theme::success() } else { theme::icon_color() })).child(div().flex_1().min_w_0().py(px(11.0)).child(Input::new(&app.api_key).appearance(false).text_size(px(13.0)))).when(has_key, |d| d.child(h_flex().items_center().gap(px(5.0)).text_size(px(11.0)).font_semibold().text_color(theme::success()).child(icon("check-circle", 13.0, theme::success())).child("Verified"))))
            .child(h_flex().mt(px(10.0)).items_center().gap(px(7.0)).text_size(px(12.0)).text_color(theme::text_caption()).child(icon("shield", 14.0, theme::success())).child("Each provider stores its own key in your OS keychain."));
    } else if app.acp_connected {
        body = body.child(eyebrow("AGENT ENDPOINT")).child(
            h_flex()
                .items_center()
                .gap(px(11.0))
                .p(px(14.0))
                .rounded(px(11.0))
                .border_1()
                .border_color(theme::hexa(0x5CCB9A4d))
                .bg(theme::success_tint())
                .child(div().size(px(32.0)).flex_none().rounded(px(9.0)).bg(theme::bg_panel()).flex().items_center().justify_center().child(icon("check-circle", 16.0, theme::success())))
                .child(v_flex().flex_1().min_w_0().child(div().text_size(px(13.0)).font_semibold().text_color(theme::text_strong()).child("Agent connected")).child(div().font_family(theme::FONT_MONO).text_size(px(11.5)).text_color(theme::text_caption()).child(app.acp_url.read(cx).value())))
                .child(div().id("acp-disc").px(px(12.0)).py(px(7.0)).rounded(px(8.0)).border_1().border_color(theme::line_strong()).text_size(px(12.0)).font_semibold().text_color(theme::text_soft()).cursor_pointer().child("Disconnect").on_click(cx.listener(|a, _, window, cx| a.disconnect_acp(window, cx)))),
        );
    } else {
        body = body
            .child(eyebrow("AGENT ENDPOINT"))
            .child(h_flex().items_center().gap(px(8.0)).px(px(12.0)).rounded(px(10.0)).border_1().border_color(theme::line_strong()).bg(theme::bg_sunken()).child(icon("link", 15.0, theme::icon_color())).child(div().flex_1().min_w_0().py(px(11.0)).child(Input::new(&app.acp_url).appearance(false).text_size(px(12.5)))))
            .child(div().mt(px(12.0)).child(Btn::primary("Connect agent").icon("link").render("acp-connect", cx.listener(|a, _, _, cx| a.connect_acp(cx)))))
            .child(div().mt(px(12.0)).text_size(px(12.0)).text_color(theme::text_caption()).line_height(px(18.0)).child("WebFluent sends only design instructions over ACP \u{2014} your model and credentials stay with the agent."));
    }
    body
}

fn settings_mcp(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    v_flex()
        .child(div().text_size(px(13.0)).text_color(theme::text_muted()).line_height(px(20.0)).mb(px(16.0)).child("Model Context Protocol servers extend the assistant with outside tools \u{2014} payments, a CMS, your own API."))
        .child(v_flex().gap(px(8.0)).children(app.mcp_list.iter().map(|m| {
            let id = m.id;
            h_flex()
                .items_center()
                .gap(px(11.0))
                .p(px(12.0))
                .rounded(px(11.0))
                .border_1()
                .border_color(theme::line())
                .bg(theme::bg_raised())
                .child(icon("server", 15.0, theme::text_soft()))
                .child(v_flex().flex_1().min_w_0().child(div().text_size(px(13.0)).font_semibold().text_color(theme::text_strong()).child(m.name.clone())).child(div().font_family(theme::FONT_MONO).text_size(px(11.5)).text_color(theme::text_caption()).child(m.meta.clone())))
                .child(h_flex().items_center().gap(px(5.0)).text_size(px(10.5)).font_semibold().text_color(if m.on { theme::success() } else { theme::text_caption() }).child(div().size(px(6.0)).rounded_full().bg(if m.on { theme::success() } else { theme::text_faint() })).child(if m.on { "Connected" } else { "Off" }))
                .child(toggle_switch("mcp-toggle", m.on, cx.listener(move |a, _, _, cx| a.toggle_mcp(id, cx))))
                .child(div().id(("mcp-rm", id as usize)).size(px(26.0)).flex().items_center().justify_center().rounded(px(7.0)).text_color(theme::icon_color()).cursor_pointer().hover(|s| s.bg(theme::danger_tint()).text_color(theme::danger())).child(icon("trash", 15.0, theme::icon_color())).on_click(cx.listener(move |a, _, _, cx| a.remove_mcp(id, cx))))
        })))
        .child(
            v_flex()
                .mt(px(16.0))
                .p(px(14.0))
                .rounded(px(11.0))
                .border_1()
                .border_dashed()
                .border_color(theme::line_strong())
                .child(div().text_size(px(12.0)).font_semibold().text_color(theme::text_soft()).mb(px(10.0)).child("Add a server"))
                .child(
                    h_flex()
                        .gap(px(8.0))
                        .child(div().flex_1().min_w_0().px(px(11.0)).py(px(9.0)).rounded(px(9.0)).border_1().border_color(theme::line_strong()).bg(theme::bg_sunken()).child(Input::new(&app.mcp_name).appearance(false).text_size(px(12.5))))
                        .child(div().flex_1().min_w_0().px(px(11.0)).py(px(9.0)).rounded(px(9.0)).border_1().border_color(theme::line_strong()).bg(theme::bg_sunken()).child(Input::new(&app.mcp_cmd).appearance(false).text_size(px(12.5))))
                        .child(div().id("mcp-add").size(px(36.0)).flex_none().flex().items_center().justify_center().rounded(px(9.0)).bg(theme::accent()).text_color(theme::accent_contrast()).cursor_pointer().child(icon("plus", 16.0, theme::accent_contrast())).on_click(cx.listener(|a, _, window, cx| a.add_mcp(window, cx)))),
                ),
        )
}

fn settings_advanced(app: &StudioApp, cx: &mut Context<StudioApp>) -> impl IntoElement {
    v_flex()
        .child(setting_toggle_row("SPA", "Single-page app (SPA)", "Client-side routing & live data fetching \u{2014} required for API integration.", app.spa_mode, cx.listener(|a, _, _, cx| a.toggle_spa(cx))))
        .child(hairline())
        .child(setting_toggle_row("ctx", "Context pruning", "Trims unrelated code from each request \u{2014} saves ~60% of tokens.", app.pruning, cx.listener(|a, _, _, cx| a.toggle_ctx(cx))))
        .child(hairline())
        .child(setting_toggle_row("cache", "Prompt caching", "Reuses the language spec across calls to cut cost and latency.", app.caching, cx.listener(|a, _, _, cx| a.toggle_prompt_cache(cx))))
        .child(hairline())
        .child(
            h_flex()
                .items_center()
                .gap(px(12.0))
                .py(px(14.0))
                .child(v_flex().flex_1().child(div().text_size(px(13.5)).font_semibold().text_color(theme::text_strong()).child("Self-healing attempts")).child(div().mt(px(2.0)).text_size(px(12.0)).text_color(theme::text_caption()).child("How many times WebFluent auto-fixes a build error before asking you.")))
                .child(
                    h_flex()
                        .items_center()
                        .gap(px(4.0))
                        .p(px(4.0))
                        .rounded(px(9.0))
                        .bg(theme::bg_sunken())
                        .border_1()
                        .border_color(theme::line())
                        .child(div().id("heal-dec").size(px(26.0)).flex().items_center().justify_center().rounded(px(6.0)).bg(theme::bg_raised()).text_color(theme::text_soft()).cursor_pointer().child(icon("minus", 14.0, theme::text_soft())).on_click(cx.listener(|a, _, _, cx| a.dec_heal(cx))))
                        .child(div().w(px(24.0)).text_center().font_family(theme::FONT_DISPLAY).text_size(px(14.0)).font_bold().text_color(theme::text_strong()).child(format!("{}", app.heal_attempts)))
                        .child(div().id("heal-inc").size(px(26.0)).flex().items_center().justify_center().rounded(px(6.0)).bg(theme::bg_raised()).text_color(theme::text_soft()).cursor_pointer().child(icon("plus", 14.0, theme::text_soft())).on_click(cx.listener(|a, _, _, cx| a.inc_heal(cx)))),
                ),
        )
}

fn setting_toggle_row(id: &'static str, title: &'static str, desc: &'static str, on: bool, on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> impl IntoElement {
    h_flex()
        .items_center()
        .gap(px(12.0))
        .py(px(14.0))
        .child(v_flex().flex_1().child(div().text_size(px(13.5)).font_semibold().text_color(theme::text_strong()).child(title)).child(div().mt(px(2.0)).text_size(px(12.0)).text_color(theme::text_caption()).line_height(px(18.0)).child(desc)))
        .child(toggle_switch(id, on, on_click))
}

fn hairline() -> impl IntoElement {
    div().h(px(1.0)).w_full().bg(theme::line_faint())
}
