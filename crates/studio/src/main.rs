mod app;
mod ui;

use gpui::{AppContext, Application, WindowOptions};
use gpui_component::{Root, TitleBar};

fn main() {
    let application = Application::new();
    application.run(move |cx| {
        gpui_component::init(cx);

        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    titlebar: Some(TitleBar::title_bar_options()),
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
