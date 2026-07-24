//! wf-core — project/document model for WebFluent Studio.
//!
//! Wraps the `webfluent` compiler behind Studio-facing types. This crate is
//! GPUI-free by design: everything here is testable headless.

mod artifacts;
mod generate;
mod project;

pub use artifacts::{Artifact, Artifacts};
pub use generate::{edit_node, generate_page, EditOutcome, GenConfig, GenError, GenOutcome};
pub use project::{compile_merged, compile_source, FileRange, OutlineNode, ResolvedNode, WfProject};

// Re-export the engine's compiled-output types so downstream crates (preview,
// studio) depend on the model through wf-core rather than webfluent directly.
pub use webfluent::{CompiledPage, CompiledSite};

/// Compile state surfaced in the Studio top bar (FR-13).
///
/// Diagnostics are structured data for the self-heal loop and the activity
/// log — the UI must never render them as code or raw compiler output (FR-6).
#[derive(Debug, Clone, PartialEq)]
pub enum CompileStatus {
    Idle,
    Compiling,
    Compiled { duration_ms: u64 },
    Failed { diagnostics: Vec<DiagnosticInfo> },
}

impl CompileStatus {
    pub fn is_failed(&self) -> bool {
        matches!(self, CompileStatus::Failed { .. })
    }
}

/// A structured compiler diagnostic, decoupled from webfluent's error types
/// so downstream crates (ai, studio) don't depend on the compiler directly.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct DiagnosticInfo {
    pub message: String,
    /// 1-based, absent for errors without a source location (codegen, IO).
    pub line: Option<usize>,
    /// 1-based, absent for errors without a source location.
    pub column: Option<usize>,
    pub hint: Option<String>,
}

impl DiagnosticInfo {
    pub fn from_error(err: &webfluent::WebFluentError) -> Self {
        use webfluent::WebFluentError as E;
        match err {
            E::LexerError(d) | E::ParseError(d) => Self {
                message: d.message.clone(),
                line: Some(d.line),
                column: Some(d.column),
                hint: d.hint.clone(),
            },
            E::CodegenError(msg) | E::ConfigError(msg) | E::IoError(msg) | E::EditError(msg) => Self {
                message: msg.clone(),
                line: None,
                column: None,
                hint: None,
            },
        }
    }
}
