/// One compiled output file, addressed by the path the preview requests it at
/// (e.g. `index.html`).
#[derive(Debug, Clone)]
pub struct Artifact {
    pub path: String,
    pub mime: String,
    pub bytes: Vec<u8>,
}

/// The full compiled output of a document. M0 produces a single self-contained
/// `index.html` (webfluent embeds CSS/JS); later milestones split assets out.
#[derive(Debug, Clone, Default)]
pub struct Artifacts {
    pub entries: Vec<Artifact>,
}

impl Artifacts {
    pub fn single_page(html: String) -> Self {
        Self {
            entries: vec![Artifact {
                path: "index.html".into(),
                mime: "text/html".into(),
                bytes: html.into_bytes(),
            }],
        }
    }

    pub fn get(&self, path: &str) -> Option<&Artifact> {
        let path = path.trim_start_matches('/');
        let path = if path.is_empty() { "index.html" } else { path };
        self.entries.iter().find(|a| a.path == path)
    }
}
