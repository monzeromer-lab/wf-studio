//! The project model and the seam to the WebFluent engine.
//!
//! [`WfProject`] is the canonical `Document` (IMPLEMENTATION_PLAN §3.3): it holds
//! the `.wf` sources in memory plus the latest [`webfluent::CompiledSite`] (SSG
//! pages, CSS, JS, node map) the preview needs. GPUI-free, so it is testable
//! headless.
//!
//! Sources are compiled from a single **merged** string (files concatenated in
//! order) rather than by merging independently-parsed ASTs — because spans are
//! byte offsets, and per-file parsing would make them collide across files
//! (every file starts at offset 0), corrupting the span-keyed node ids. The
//! merge keeps every span unique; per-file offset ranges map any node back to
//! its source file.

use std::collections::BTreeMap;
use webfluent::{compile_studio, apply_edits, CompiledSite, EditOp, NodeInfo};
use webfluent::lexer::Lexer;
use webfluent::parser::Parser;
use webfluent::config::ProjectConfig;

/// The default preview project config (theme defaults, no i18n).
fn preview_config() -> ProjectConfig {
    serde_json::from_str(r#"{"name":"preview"}"#).expect("static preview config parses")
}

/// One source file's byte range within the merged source.
#[derive(Debug, Clone)]
pub struct FileRange {
    pub path: String,
    pub start: usize,
    pub end: usize,
}

/// Concatenate ordered sources into one string, tracking each file's byte range.
/// A newline is inserted between files so adjacent declarations never fuse.
fn merge_sources<'a>(sources: impl IntoIterator<Item = (&'a str, &'a str)>) -> (String, Vec<FileRange>) {
    let mut merged = String::new();
    let mut ranges = Vec::new();
    for (path, content) in sources {
        let start = merged.len();
        merged.push_str(content);
        if !content.ends_with('\n') {
            merged.push('\n');
        }
        ranges.push(FileRange { path: path.to_string(), start, end: merged.len() });
    }
    (merged, ranges)
}

/// Compile ordered `.wf` sources into a [`CompiledSite`], the merged source they
/// were compiled from, and the per-file offset ranges. Ordering matters — put
/// `App.wf` first, as `wf build` does.
pub fn compile_merged<'a>(
    sources: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> anyhow::Result<(CompiledSite, String, Vec<FileRange>)> {
    let (merged, ranges) = merge_sources(sources);
    let tokens = Lexer::new(&merged, "<studio>")
        .tokenize()
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let program = Parser::new(tokens, "<studio>")
        .parse()
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    // Semantic gate (M4.E): the codegen is permissive and never fails, so a program
    // that parses can still reference an undeclared component, aim a Route at a
    // missing page, or declare a page twice — all of which only break at runtime.
    // Reject such a compile here so a bad AI edit rolls back instead of shipping a
    // broken preview. (Line/column are merged-source coordinates; mapping back to
    // the source file is a later milestone.)
    if let Some(first) = webfluent::validate_semantics(&program, "<studio>").into_iter().next() {
        return Err(anyhow::anyhow!("{first}"));
    }
    let site = compile_studio(&program, &preview_config(), &Default::default());
    Ok((site, merged, ranges))
}

/// Compile a single `.wf` source string (convenience over [`compile_merged`]).
pub fn compile_source(source: &str) -> anyhow::Result<CompiledSite> {
    Ok(compile_merged([("<studio>", source)])?.0)
}

// ─── Project model ───────────────────────────────────────

/// The starter page a fresh preview boots with.
const SEED_HOME: &str = "\
Page Home (path: \"/\", title: \"WebFluent Studio\") {
  Container {
    Heading(\"Welcome to WebFluent Studio\", h1)
    Text(\"This preview is compiled live from .wf source.\")
    Button(\"Get started\", primary)
  }
}
";

/// A node resolved from a preview click: its sidecar info, the source file it
/// lives in, and its exact source text.
#[derive(Debug)]
pub struct ResolvedNode<'a> {
    pub info: &'a NodeInfo,
    pub file: String,
    pub source_slice: String,
}

/// A node in the page outline: its id, a short label (the element name), and its
/// children, forming the tree the outline panel renders.
#[derive(Debug, Clone, PartialEq)]
pub struct OutlineNode {
    pub id: String,
    pub label: String,
    pub children: Vec<OutlineNode>,
}

/// Build outline subtrees for `ids` from the child + label lookups.
fn build_outline(
    ids: &[String],
    children: &std::collections::HashMap<String, Vec<String>>,
    labels: &std::collections::HashMap<String, String>,
) -> Vec<OutlineNode> {
    ids.iter()
        .map(|id| OutlineNode {
            id: id.clone(),
            label: labels.get(id).cloned().unwrap_or_default(),
            children: children.get(id).map(|c| build_outline(c, children, labels)).unwrap_or_default(),
        })
        .collect()
}

/// An in-memory WebFluent project: `.wf` sources plus the latest compile output.
///
/// On a failed recompile the previous good [`CompiledSite`] is kept, so the
/// preview never goes blank while the user (or AI) is mid-edit.
pub struct WfProject {
    sources: BTreeMap<String, String>,
    /// The merged source the current `compiled`/`ranges` were built from.
    merged: String,
    /// Per-file byte ranges within `merged`.
    ranges: Vec<FileRange>,
    compiled: CompiledSite,
    error: Option<String>,
}

impl WfProject {
    /// A minimal seeded project (a single starter page), compiled and ready.
    pub fn seed() -> Self {
        let mut sources = BTreeMap::new();
        sources.insert("src/pages/Home.wf".to_string(), SEED_HOME.to_string());
        let mut project = Self {
            sources,
            merged: String::new(),
            ranges: Vec::new(),
            compiled: CompiledSite::default(),
            error: None,
        };
        project.recompile();
        project
    }

    /// The most recent successful compile.
    pub fn compiled(&self) -> &CompiledSite {
        &self.compiled
    }

    /// The error from the last recompile attempt, if it failed.
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Replace one source file's content (creating it if new).
    pub fn set_source(&mut self, path: impl Into<String>, content: impl Into<String>) {
        self.sources.insert(path.into(), content.into());
    }

    /// The current source of a file, if it exists.
    pub fn file_source(&self, path: &str) -> Option<&str> {
        self.sources.get(path).map(|s| s.as_str())
    }

    /// Resolve a node id (from a `data-wf-node` click) to its info, source file,
    /// and exact source text. `None` if the id is unknown.
    pub fn resolve_node(&self, node_id: &str) -> Option<ResolvedNode<'_>> {
        let info = self.compiled.node_map.info(node_id)?;
        let start = info.span.start as usize;
        let range = self.ranges.iter().find(|r| start >= r.start && start < r.end)?;
        let source_slice = info.span.slice(&self.merged).to_string();
        Some(ResolvedNode { info, file: range.path.clone(), source_slice })
    }

    /// Apply structured edits to the source file containing `node_id`. On success
    /// the file's source is updated in place (call [`WfProject::recompile`] to
    /// rebuild). On any error the sources are left untouched.
    pub fn edit_node(&mut self, node_id: &str, ops: &[EditOp]) -> anyhow::Result<()> {
        let file = self
            .resolve_node(node_id)
            .map(|r| r.file)
            .ok_or_else(|| anyhow::anyhow!("unknown node {node_id}"))?;
        let src = self
            .sources
            .get(&file)
            .ok_or_else(|| anyhow::anyhow!("no source for {file}"))?
            .clone();
        let edited = apply_edits(&src, ops).map_err(|e| anyhow::anyhow!("{e}"))?;
        self.sources.insert(file, edited);
        Ok(())
    }

    /// The page's element tree, derived from the node map — the source for the
    /// outline panel. Each node's label is its leading source token (the element
    /// name); control-flow scaffolding (`if`/`for` branches) is transparent, so a
    /// node attaches to its nearest rendered ancestor.
    pub fn outline(&self) -> Vec<OutlineNode> {
        use std::collections::{HashMap, HashSet};

        let mut labels: HashMap<String, String> = HashMap::new();
        let mut ids: Vec<String> = Vec::new();
        for (id, info) in self.compiled.node_map.iter() {
            let label = info
                .span
                .slice(&self.merged)
                .split(|c: char| c == '(' || c == ' ' || c == '{' || c == '\n')
                .next()
                .unwrap_or("")
                .to_string();
            labels.insert(id.clone(), label);
            ids.push(id.clone());
        }
        ids.sort();
        let idset: HashSet<&str> = ids.iter().map(|s| s.as_str()).collect();

        // A node's parent is the nearest ancestor id that is itself a node —
        // stripping trailing `.seg`/`:seg` (skipping non-node branch scaffolding).
        let parent_of = |id: &str| -> Option<String> {
            let mut cur = id;
            while let Some(pos) = cur.rfind(|c| c == '.' || c == ':') {
                let cand = &cur[..pos];
                if cand.is_empty() {
                    return None;
                }
                if idset.contains(cand) {
                    return Some(cand.to_string());
                }
                cur = cand;
            }
            None
        };

        let mut children: HashMap<String, Vec<String>> = HashMap::new();
        let mut roots: Vec<String> = Vec::new();
        for id in &ids {
            match parent_of(id) {
                Some(p) => children.entry(p).or_default().push(id.clone()),
                None => roots.push(id.clone()),
            }
        }
        build_outline(&roots, &children, &labels)
    }

    /// Recompile from the current sources. Keeps the last good compile on error.
    pub fn recompile(&mut self) {
        let sources = self.sources.iter().map(|(k, v)| (k.as_str(), v.as_str()));
        match compile_merged(sources) {
            Ok((site, merged, ranges)) => {
                self.compiled = site;
                self.merged = merged;
                self.ranges = ranges;
                self.error = None;
            }
            Err(e) => self.error = Some(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_project_compiles_and_resolves() {
        let p = WfProject::seed();
        assert!(p.error().is_none(), "seed should compile: {:?}", p.error());
        let site = p.compiled();
        assert_eq!(site.pages.len(), 1);
        assert!(site.pages[0].html.contains("data-wf-node="));

        // Resolve a node back to its file + source text.
        let heading = site.node_map.iter()
            .find(|(_, i)| i.span.slice(&p.merged).starts_with("Heading"))
            .map(|(id, _)| id.clone())
            .expect("a Heading node");
        let r = p.resolve_node(&heading).expect("resolves");
        assert_eq!(r.file, "src/pages/Home.wf");
        assert!(r.source_slice.starts_with("Heading("));
    }

    #[test]
    fn identical_files_do_not_collide_on_node_ids() {
        // Two structurally-identical pages: per-file parsing would give their
        // nodes identical spans and collapse the node map. The merge prevents it.
        let (site, _merged, _ranges) = compile_merged([
            ("src/pages/A.wf", "Page A (path: \"/a\") { Text(\"hi\") }"),
            ("src/pages/B.wf", "Page B (path: \"/b\") { Text(\"hi\") }"),
        ])
        .unwrap();
        // Both pages present, and each Text node has a distinct id.
        assert_eq!(site.pages.len(), 2);
        assert!(site.node_map.info("A:0").is_some());
        assert!(site.node_map.info("B:0").is_some());
        // Distinct spans → 2 nodes, no collapse.
        assert_eq!(site.node_map.len(), 2);
    }

    #[test]
    fn resolve_node_targets_the_right_file() {
        let (site, merged, ranges) = compile_merged([
            ("src/pages/A.wf", "Page A (path: \"/a\") { Text(\"aaa\") }"),
            ("src/pages/B.wf", "Page B (path: \"/b\") { Text(\"bbb\") }"),
        ])
        .unwrap();
        let a = site.node_map.info("A:0").unwrap();
        let b = site.node_map.info("B:0").unwrap();
        let file_of = |start: usize| ranges.iter().find(|r| start >= r.start && start < r.end).map(|r| r.path.as_str());
        assert_eq!(file_of(a.span.start as usize), Some("src/pages/A.wf"));
        assert_eq!(file_of(b.span.start as usize), Some("src/pages/B.wf"));
        assert_eq!(a.span.slice(&merged), "Text(\"aaa\")");
        assert_eq!(b.span.slice(&merged), "Text(\"bbb\")");
    }

    #[test]
    fn recompile_keeps_last_good_on_error() {
        let mut p = WfProject::seed();
        let good = p.compiled().pages[0].html.clone();
        p.set_source("src/pages/Home.wf", "Page Home (path: \"/\") { Button(\"oops\"");
        p.recompile();
        assert!(p.error().is_some());
        assert_eq!(p.compiled().pages[0].html, good, "last good compile must survive an error");
    }

    #[test]
    fn recompile_reflects_source_edits() {
        let mut p = WfProject::seed();
        p.set_source("src/pages/Home.wf", "Page Home (path: \"/\") { Text(\"changed\") }");
        p.recompile();
        assert!(p.error().is_none());
        assert!(p.compiled().pages[0].html.contains("changed"));
    }

    #[test]
    fn malformed_source_errors_cleanly() {
        assert!(compile_source("Page Home (path: \"/\") { Button(\"x\"").is_err());
    }

    #[test]
    fn semantic_gate_rejects_a_parseable_but_broken_program() {
        // Parses cleanly, but `ProfileCard` is never declared — the M4.E gate
        // must reject it rather than compile a preview that breaks at runtime.
        let err = compile_source("Page Home (path: \"/\") { ProfileCard() }\n").unwrap_err();
        assert!(err.to_string().contains("ProfileCard"), "err: {err}");
    }

    #[test]
    fn recompile_keeps_last_good_on_a_semantic_break() {
        let mut p = WfProject::seed();
        let good = p.compiled().pages[0].html.clone();
        // Reparses fine, but references an undeclared component: the gate rejects
        // it and the previous good compile survives (the transactional invariant).
        p.set_source("src/pages/Home.wf", "Page Home (path: \"/\") { ProfileCard() }");
        p.recompile();
        assert!(p.error().is_some(), "semantic break must set an error");
        assert_eq!(p.compiled().pages[0].html, good, "last good compile must survive a semantic break");
    }

    #[test]
    fn edit_node_edits_the_source_and_recompiles() {
        let mut p = WfProject::seed();
        let heading = p.compiled().node_map.iter()
            .find(|(_, i)| i.span.slice(&p.merged).starts_with("Heading"))
            .map(|(id, _)| id.clone())
            .expect("Heading node");

        // SetText → source + recompiled page reflect it.
        p.edit_node(&heading, &[EditOp::SetText { node: heading.clone(), value: "Changed title".into() }]).unwrap();
        p.recompile();
        assert!(p.error().is_none());
        assert!(p.compiled().pages[0].html.contains("Changed title"));

        // SetStyle creates a style block on the node.
        p.edit_node(&heading, &[EditOp::SetStyle { node: heading.clone(), prop: "color".into(), value: "\"#ff0000\"".into() }]).unwrap();
        p.recompile();
        assert!(p.error().is_none());
        assert!(p.sources.get("src/pages/Home.wf").unwrap().contains("color: \"#ff0000\""));
    }

    #[test]
    fn outline_reflects_the_node_tree() {
        let p = WfProject::seed();
        let tree = p.outline();
        assert_eq!(tree.len(), 1, "one root (the Container)");
        assert_eq!(tree[0].label, "Container");
        let labels: Vec<&str> = tree[0].children.iter().map(|c| c.label.as_str()).collect();
        assert_eq!(labels, vec!["Heading", "Text", "Button"]);
    }

    #[test]
    fn append_child_adds_a_block_and_grows_the_outline() {
        let mut p = WfProject::seed();
        let container = p.compiled().node_map.iter()
            .find(|(_, i)| i.span.slice(&p.merged).starts_with("Container"))
            .map(|(id, _)| id.clone())
            .expect("Container node");
        assert_eq!(p.outline()[0].children.len(), 3);

        p.edit_node(&container, &[EditOp::AppendChild { node: container.clone(), wf: "Button(\"New\")".into() }]).unwrap();
        p.recompile();
        assert!(p.error().is_none());
        assert!(p.compiled().pages[0].html.contains(">New<"));
        assert_eq!(p.outline()[0].children.len(), 4, "outline gains the appended child");
    }

    #[test]
    fn edit_node_rejects_unknown_node_without_touching_sources() {
        let mut p = WfProject::seed();
        let before = p.sources.clone();
        assert!(p.edit_node("Nope:9", &[EditOp::SetText { node: "Nope:9".into(), value: "x".into() }]).is_err());
        assert_eq!(p.sources, before);
    }
}
