//! wf-preview — webview hosting for the live preview.
//!
//! Two backends behind one seam (IMPLEMENTATION_PLAN §4.1 + M0 plan):
//! - [`detached`]: a separate native preview window on its own thread
//!   (tao + wry). The Linux/Wayland path, and the M0 deliverable.
//! - embedded (M0 task 5 spike): gpui-component `WebView` inside the studio
//!   window; X11/macOS/Windows only.
//!
//! This crate is GPUI-free: the studio adapts it into entities/tasks.

mod bridge;
mod serve;
mod store;

#[cfg(target_os = "linux")]
mod detached;
#[cfg(target_os = "linux")]
pub use detached::spawn_detached;

pub use bridge::BRIDGE_JS;
pub use serve::{resolve, respond, self_contained, serve, DIFF_SHELL, MIME_CSS, MIME_HTML, MIME_JS};
pub use store::ArtifactStore;

/// Commands into the preview host (studio → preview).
#[derive(Debug, Clone)]
pub enum PreviewCommand {
    /// Re-navigate to the current artifacts (picks up a new store version).
    Reload,
    /// Run JS in the page — the M2 bridge channel (selection outlines, scrub).
    EvalScript(String),
    /// Close the window and end the host thread.
    Shutdown,
}

/// Events out of the preview host (preview → studio).
#[derive(Debug, Clone)]
pub enum PreviewEvent {
    /// Webview constructed and first navigation issued.
    Ready,
    /// A message from the in-page bridge (console errors now; selection later).
    Ipc(String),
    /// The user closed the preview window; the host thread has ended.
    WindowClosed,
}

/// A running preview host. Dropping it does not kill the window; send
/// [`PreviewCommand::Shutdown`] for a clean close.
pub struct PreviewHandle {
    pub(crate) sender: Box<dyn Fn(PreviewCommand) -> anyhow::Result<()> + Send + Sync>,
    pub events: async_channel::Receiver<PreviewEvent>,
}

impl PreviewHandle {
    pub fn send(&self, cmd: PreviewCommand) -> anyhow::Result<()> {
        (self.sender)(cmd)
    }
}
