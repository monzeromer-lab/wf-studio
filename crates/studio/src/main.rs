//! WebFluent Studio — a native GPUI application. The Studio chrome is GPUI; the
//! generated website renders in a webview embedded in the canvas ([`app`]).

mod app;
mod assets;
mod ipc;
mod site;
mod state;
mod theme;
mod ui;

use gpui::{AppContext, Application, WindowDecorations, WindowOptions};
use gpui_component::{Root, Theme, ThemeMode, TitleBar};

use crate::assets::Assets;

fn main() {
    // The canvas preview is an embedded *child* webview. On Linux, webkit child
    // surfaces are X11-only, so force gpui + gtk/gdk onto X11 (XWayland) before
    // any window or webview is created.
    #[cfg(target_os = "linux")]
    unsafe {
        std::env::remove_var("WAYLAND_DISPLAY");
        std::env::set_var("GDK_BACKEND", "x11");
    }
    #[cfg(target_os = "linux")]
    gtk::init().expect("failed to initialise gtk");

    // `with_assets` lets gpui-component (and our chrome) resolve `icons/*.svg`
    // from the embedded set in `crate::assets`.
    Application::new().with_assets(Assets).run(move |cx| {
        gpui_component::init(cx);

        // gpui-component's built-in window controls (and every widget that reads
        // `cx.theme()`, e.g. Input's text color) follow this global theme rather
        // than our own `crate::theme` tokens, so force dark mode and align the
        // colors it actually uses with our palette instead of its generic ones.
        Theme::change(ThemeMode::Dark, None, cx);
        let ui_theme = Theme::global_mut(cx);
        ui_theme.foreground = theme::text();
        ui_theme.secondary_hover = theme::seg_active();
        ui_theme.secondary_active = theme::raised();
        ui_theme.secondary_foreground = theme::text();
        ui_theme.danger = theme::danger();
        ui_theme.danger_active = theme::accent_deep();
        ui_theme.danger_foreground = theme::white(1.0);

        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    titlebar: Some(TitleBar::title_bar_options()),
                    window_decorations: Some(WindowDecorations::Client),
                    ..Default::default()
                },
                |window, cx| {
                    let studio = cx.new(|cx| app::StudioApp::new(window, cx));
                    cx.new(|cx| Root::new(studio, window, cx))
                },
            )?;
            anyhow::Ok(())
        })
        .detach();
    });
}
