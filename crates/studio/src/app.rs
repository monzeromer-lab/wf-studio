use std::time::Instant;

use gpui::{Context, Entity, Render, Window, prelude::*};
use gpui_component::{input::InputState, v_flex};
use wf_core::{CompileStatus, Document};
use wf_preview::{ArtifactStore, PreviewCommand, PreviewEvent, PreviewHandle};

use crate::ui;

const SAMPLE_WF: &str = include_str!("../../../assets/sample.wf");
const PREVIEW_TITLE: &str = "WebFluent Studio — معاينة";

pub struct StudioApp {
    pub document: Document,
    pub status: CompileStatus,
    pub prompt_state: Entity<InputState>,
    pub store: ArtifactStore,
    /// `None` = preview window closed (user gets a Reopen button).
    pub preview: Option<PreviewHandle>,
}

impl StudioApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let prompt_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("صِف ما تريد إنشاءه… / Describe what you want to build…")
        });

        let mut this = Self {
            document: Document::new(SAMPLE_WF),
            status: CompileStatus::Idle,
            prompt_state,
            store: ArtifactStore::new(),
            preview: None,
        };

        // Publish before the preview's first navigation so index.html exists.
        this.recompile(cx);
        this.open_preview(cx);
        this
    }

    /// M0: compile synchronously (single page, ~instant — the wf-core test
    /// suite runs it in <1ms). M1 moves this to the background executor for
    /// generation-sized work.
    pub fn recompile(&mut self, cx: &mut Context<Self>) {
        self.status = CompileStatus::Compiling;
        cx.notify();

        let started = Instant::now();
        self.status = match self.document.compile() {
            Ok(artifacts) => {
                self.store.publish(artifacts);
                if let Some(preview) = &self.preview {
                    let _ = preview.send(PreviewCommand::Reload);
                }
                CompileStatus::Compiled {
                    duration_ms: started.elapsed().as_millis() as u64,
                }
            }
            Err(diagnostics) => CompileStatus::Failed { diagnostics },
        };
        cx.notify();
    }

    pub fn open_preview(&mut self, cx: &mut Context<Self>) {
        if self.preview.is_some() {
            return;
        }

        #[cfg(target_os = "linux")]
        match wf_preview::spawn_detached(self.store.clone(), PREVIEW_TITLE.into()) {
            Ok(handle) => {
                let events = handle.events.clone();
                self.preview = Some(handle);

                cx.spawn(async move |this, cx| {
                    while let Ok(event) = events.recv().await {
                        let alive = this.update(cx, |app, cx| app.on_preview_event(event, cx));
                        if alive.is_err() {
                            break;
                        }
                    }
                })
                .detach();
            }
            Err(error) => {
                eprintln!("wf-studio: failed to open preview: {error:#}");
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            // macOS/Windows get the embedded webview (M0 task 5 seam).
        }

        cx.notify();
    }

    fn on_preview_event(&mut self, event: PreviewEvent, cx: &mut Context<Self>) {
        match event {
            PreviewEvent::Ready => {}
            PreviewEvent::Ipc(message) => {
                // M0: bridge messages only get logged. FR-19 self-healing
                // consumes these in M4; click-to-select arrives with M2.
                eprintln!("wf-preview bridge: {message}");
            }
            PreviewEvent::WindowClosed => {
                self.preview = None;
            }
        }
        cx.notify();
    }
}

impl Render for StudioApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .child(ui::top_bar(self, window, cx))
            .child(ui::preview_area(self, window, cx))
            .child(ui::prompt_dock(self, window, cx))
    }
}
