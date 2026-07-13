//! Detached preview host: a native window on a dedicated thread running a
//! tao event loop (gtk-based on Linux — works on Wayland and X11 alike, the
//! same shape as a Tauri app) with a wry webview filling it.
//!
//! Studio → preview: `EventLoopProxy::send_event` (wakes the tao loop).
//! Preview → studio: `async_channel` drained from a gpui task.

use std::borrow::Cow;
use std::sync::{Mutex, mpsc};

use anyhow::{Context as _, anyhow};
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
    platform::{
        run_return::EventLoopExtRunReturn,
        unix::{EventLoopBuilderExtUnix, WindowExtUnix},
    },
    window::WindowBuilder,
};
use wry::{
    WebViewBuilder, WebViewBuilderExtUnix,
    http::{Request, Response, header::CONTENT_TYPE},
};

use crate::{ArtifactStore, BRIDGE_JS, PreviewCommand, PreviewEvent, PreviewHandle};

/// The custom-protocol origin. On Linux/macOS webkit resolves the scheme as
/// `wf://localhost/<path>`; only Linux matters for the detached host.
const PREVIEW_ORIGIN: &str = "wf://localhost";

/// Spawn the preview window on its own thread. Returns once the webview is
/// built and navigating (or with the build error).
pub fn spawn_detached(store: ArtifactStore, title: String) -> anyhow::Result<PreviewHandle> {
    let (ready_tx, ready_rx) =
        mpsc::channel::<anyhow::Result<EventLoopProxy<PreviewCommand>>>();
    let (event_tx, event_rx) = async_channel::unbounded::<PreviewEvent>();

    std::thread::Builder::new()
        .name("wf-preview".into())
        .spawn(move || {
            if let Err(e) = run_preview_loop(store, title, &ready_tx, event_tx) {
                // No-op if the proxy was already handed over successfully.
                let _ = ready_tx.send(Err(e));
            }
        })
        .context("failed to spawn preview thread")?;

    let proxy = ready_rx
        .recv()
        .context("preview thread died before handing over its proxy")??;

    let proxy = Mutex::new(proxy);
    Ok(PreviewHandle {
        sender: Box::new(move |cmd| {
            proxy
                .lock()
                .unwrap()
                .send_event(cmd)
                .map_err(|_| anyhow!("preview event loop is gone"))
        }),
        events: event_rx,
    })
}

fn run_preview_loop(
    store: ArtifactStore,
    title: String,
    ready_tx: &mpsc::Sender<anyhow::Result<EventLoopProxy<PreviewCommand>>>,
    event_tx: async_channel::Sender<PreviewEvent>,
) -> anyhow::Result<()> {
    let mut event_loop = EventLoopBuilder::<PreviewCommand>::with_user_event()
        .with_any_thread(true)
        .build();

    let window = WindowBuilder::new()
        .with_title(&title)
        .with_inner_size(LogicalSize::new(1000.0, 760.0))
        .build(&event_loop)
        .context("failed to create preview window")?;

    let protocol_store = store.clone();
    let ipc_tx = event_tx.clone();
    let builder = WebViewBuilder::new()
        .with_custom_protocol("wf".into(), move |_id, request| {
            serve(&protocol_store, request)
        })
        .with_initialization_script(BRIDGE_JS)
        .with_ipc_handler(move |message: Request<String>| {
            let _ = ipc_tx.send_blocking(PreviewEvent::Ipc(message.into_body()));
        })
        .with_url(format!("{PREVIEW_ORIGIN}/index.html"));

    // On Linux the webview goes into the tao window's gtk vbox — the only
    // route that works on both Wayland and X11 (raw-handle child webviews
    // are X11-only, see M0_RESULTS).
    let vbox = window
        .default_vbox()
        .ok_or_else(|| anyhow!("tao window has no gtk vbox"))?;
    let webview = builder
        .build_gtk(vbox)
        .context("failed to build preview webview")?;

    ready_tx
        .send(Ok(event_loop.create_proxy()))
        .map_err(|_| anyhow!("spawn_detached dropped before ready"))?;
    let _ = event_tx.send_blocking(PreviewEvent::Ready);

    let mut user_closed = false;
    event_loop.run_return(|event, _target, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::UserEvent(cmd) => match cmd {
                PreviewCommand::Reload => {
                    let version = store.version();
                    let _ = webview
                        .load_url(&format!("{PREVIEW_ORIGIN}/index.html?v={version}"));
                }
                PreviewCommand::EvalScript(js) => {
                    let _ = webview.evaluate_script(&js);
                }
                PreviewCommand::Shutdown => *control_flow = ControlFlow::Exit,
            },
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                user_closed = true;
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });

    if user_closed {
        let _ = event_tx.send_blocking(PreviewEvent::WindowClosed);
    }
    Ok(())
}

fn serve(store: &ArtifactStore, request: Request<Vec<u8>>) -> Response<Cow<'static, [u8]>> {
    match store.get(request.uri().path()) {
        Some((mime, bytes)) => Response::builder()
            .status(200)
            .header(CONTENT_TYPE, mime)
            .body(Cow::Owned(bytes))
            .unwrap(),
        None => Response::builder()
            .status(404)
            .header(CONTENT_TYPE, "text/plain")
            .body(Cow::Borrowed(b"not found".as_slice()))
            .unwrap(),
    }
}
