use std::path::Path;

use webfluent::Template;

use crate::{Artifacts, DiagnosticInfo};

/// A WebFluent document: the canonical `.wf` source of one project page.
///
/// The AST is always derived from `source` at compile time — source is the
/// single source of truth (see IMPLEMENTATION_PLAN §3.3).
#[derive(Debug, Clone)]
pub struct Document {
    source: String,
}

impl Document {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
        }
    }

    pub fn from_file(path: &Path) -> std::io::Result<Self> {
        Ok(Self::new(std::fs::read_to_string(path)?))
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn set_source(&mut self, source: impl Into<String>) {
        self.source = source.into();
    }

    /// Compile to preview artifacts. Errors come back as structured
    /// diagnostics — currently at most one, but the signature is a `Vec`
    /// because the self-heal loop (FR-19/20) accumulates them.
    pub fn compile(&self) -> Result<Artifacts, Vec<DiagnosticInfo>> {
        let template =
            Template::from_str(&self.source).map_err(|e| vec![DiagnosticInfo::from_error(&e)])?;
        let html = template
            .render_html(&serde_json::json!({}))
            .map_err(|e| vec![DiagnosticInfo::from_error(&e)])?;
        Ok(Artifacts::single_page(html))
    }
}
