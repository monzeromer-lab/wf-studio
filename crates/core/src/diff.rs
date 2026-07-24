//! AST diff → chips (IMPLEMENTATION_PLAN §4.5).
//!
//! Given a base `.wf` source and a proposal (e.g. from [`crate::edit_node`]), diff
//! the two by node id and emit one [`Chip`] per change — a *human* description of
//! what changed plus the [`EditOp`] that applies it. Chips are the reviewable unit:
//! the UI shows their labels (never syntax, FR-6) and the user accepts/rejects each.
//!
//! Matching is by the engine's structural node id. Each matched node is compared on
//! its OWN header (`paren_span` — args + modifiers) and style block only; children
//! are separate nodes with their own ids, so a node whose only change is a
//! child add/remove produces no chip — its child does. Nodes only in the base are
//! *removed*; nodes only in the proposal (whose parent is a base node) are *added*.
//!
//! Known limitation (first cut): because ids are structural, an insert/remove in
//! the MIDDLE of a child list shifts the later siblings' ids, so those show as
//! `Changed` rather than one clean move. End-appends and property edits — the
//! common scoped-edit shapes — diff cleanly.

use std::collections::BTreeMap;

use tracing::debug;
use webfluent::codegen::node_id::visit_nodes;
use webfluent::lexer::Lexer;
use webfluent::parser::ast::{Arg, ComponentRef, Expr, Program, UIElement};
use webfluent::parser::Parser;
use webfluent::EditOp;

/// The category of a change, matching the studio's review chips.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipKind {
    Text,
    Style,
    Structure,
    Behavior,
}

/// One reviewable change: a human label + the edit that applies it if accepted.
#[derive(Debug, Clone)]
pub struct Chip {
    pub node_id: String,
    pub kind: ChipKind,
    /// Human-readable, never syntax (FR-6).
    pub label: String,
    /// The span edit to apply to the BASE source if this chip is accepted.
    pub op: EditOp,
}

/// Diff `base` against `proposal`, producing one chip per change. Both must parse.
pub fn diff(base: &str, proposal: &str) -> anyhow::Result<Vec<Chip>> {
    let base_prog = parse(base)?;
    let prop_prog = parse(proposal)?;
    let base_nodes = collect(&base_prog);
    let prop_nodes = collect(&prop_prog);

    let mut chips = Vec::new();

    // Changed + removed nodes (walk the base).
    for (id, bui) in &base_nodes {
        match prop_nodes.get(id) {
            Some(pui) => {
                if let Some((kind, label)) = classify(bui, base, pui, proposal) {
                    chips.push(Chip {
                        node_id: id.clone(),
                        kind,
                        label,
                        op: EditOp::ReplaceNode { node: id.clone(), wf: node_source(pui, proposal) },
                    });
                }
            }
            None => {
                chips.push(Chip {
                    node_id: id.clone(),
                    kind: ChipKind::Structure,
                    label: format!("Removed {}", elem_name(bui)),
                    op: EditOp::RemoveNode { node: id.clone() },
                });
            }
        }
    }

    // Added nodes: the root of each added subtree (its parent is a base node).
    for (id, pui) in &prop_nodes {
        if base_nodes.contains_key(id) {
            continue;
        }
        if let Some((parent, index)) = numeric_parent(id) {
            if base_nodes.contains_key(&parent) {
                chips.push(Chip {
                    node_id: id.clone(),
                    kind: ChipKind::Structure,
                    label: format!("Added {}", elem_name(pui)),
                    op: EditOp::InsertChild { node: parent, index, wf: node_source(pui, proposal) },
                });
            }
        }
    }

    // Deterministic order for a stable review list.
    chips.sort_by(|a, b| a.node_id.cmp(&b.node_id));
    debug!(
        count = chips.len(),
        kinds = ?chips.iter().map(|c| c.kind).collect::<Vec<_>>(),
        "diff produced chips"
    );
    Ok(chips)
}

/// True if `fix` preserves the design of `base`: same set of elements with the
/// same modifiers and style blocks (§4.6 design-freeze). It ignores element order
/// and non-UI statements, so a logic-only fix (adding `state`/an action) — which
/// shifts structural ids — still counts as design-preserving; only a changed
/// element type, modifier, or style block trips it. Text/logic may change.
pub fn design_preserved(base: &str, fix: &str) -> anyhow::Result<bool> {
    Ok(style_fingerprint(base)? == style_fingerprint(fix)?)
}

/// A sorted, position-independent fingerprint of a page's appearance: one entry
/// per element = `name | sorted-modifiers | style-block-source`.
fn style_fingerprint(source: &str) -> anyhow::Result<Vec<String>> {
    let prog = parse(source)?;
    let mut fp = Vec::new();
    visit_nodes(&prog, &mut |ui, _id, _component| {
        let mut mods = ui.modifiers.clone();
        mods.sort();
        fp.push(format!("{}|{}|{}", elem_name(ui), mods.join(","), span_src(ui.style_span, source)));
    });
    fp.sort();
    Ok(fp)
}

fn parse(src: &str) -> anyhow::Result<Program> {
    let tokens = Lexer::new(src, "<diff>").tokenize().map_err(|e| anyhow::anyhow!("{e}"))?;
    Parser::new(tokens, "<diff>").parse().map_err(|e| anyhow::anyhow!("{e}"))
}

/// id → element, via the engine's one shared node-id traversal.
fn collect(prog: &Program) -> BTreeMap<String, &UIElement> {
    let mut map = BTreeMap::new();
    visit_nodes(prog, &mut |ui, id, _component| {
        map.insert(id.to_string(), ui);
    });
    map
}

/// Classify how a matched node's OWN properties changed, or `None` if unchanged.
fn classify(bui: &UIElement, bsrc: &str, pui: &UIElement, psrc: &str) -> Option<(ChipKind, String)> {
    let bname = elem_name(bui);
    let pname = elem_name(pui);
    if bname != pname {
        return Some((ChipKind::Structure, format!("Changed {bname} to {pname}")));
    }

    let bparen = span_src(bui.paren_span, bsrc);
    let pparen = span_src(pui.paren_span, psrc);
    let bstyle = span_src(bui.style_span, bsrc);
    let pstyle = span_src(pui.style_span, psrc);

    if bparen == pparen && bstyle == pstyle {
        return None; // own properties unchanged; any child change is its own chip
    }
    if bstyle != pstyle {
        return Some((ChipKind::Style, format!("{pname} style updated")));
    }
    if first_text(bui) != first_text(pui) {
        return Some((ChipKind::Text, format!("{pname} text changed")));
    }
    if bui.modifiers != pui.modifiers {
        return Some((ChipKind::Style, format!("{pname} style updated")));
    }
    Some((ChipKind::Behavior, format!("{pname} settings changed")))
}

fn elem_name(ui: &UIElement) -> String {
    match &ui.component {
        ComponentRef::BuiltIn(n) | ComponentRef::UserDefined(n) => n.clone(),
        ComponentRef::SubComponent(p, s) => format!("{p}.{s}"),
    }
}

/// The first positional string-literal argument (an element's text), if any.
fn first_text(ui: &UIElement) -> Option<&str> {
    ui.args.iter().find_map(|a| match a {
        Arg::Positional(Expr::StringLiteral(s)) => Some(s.as_str()),
        _ => None,
    })
}

fn node_source(ui: &UIElement, src: &str) -> String {
    ui.span.slice(src).to_string()
}

fn span_src(span: Option<webfluent::parser::ast::Span>, src: &str) -> String {
    span.map(|s| s.slice(src).to_string()).unwrap_or_default()
}

/// The direct structural parent id + child index, for an id ending in `.N`.
/// `None` for top-level ids (`Page:0`) or control-flow branches (`…t.0`).
fn numeric_parent(id: &str) -> Option<(String, usize)> {
    let pos = id.rfind('.')?;
    let index: usize = id[pos + 1..].parse().ok()?;
    Some((id[..pos].to_string(), index))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(body: &str) -> String {
        format!("Page P (path: \"/\") {{ {body} }}")
    }

    #[test]
    fn identical_sources_produce_no_chips() {
        let s = page("Container { Heading(\"Hi\", h1) }");
        assert!(diff(&s, &s).unwrap().is_empty());
    }

    #[test]
    fn a_text_change_is_one_text_chip() {
        let base = page("Container { Heading(\"Hi\", h1) }");
        let prop = page("Container { Heading(\"Hello\", h1) }");
        let chips = diff(&base, &prop).unwrap();
        assert_eq!(chips.len(), 1);
        assert_eq!(chips[0].kind, ChipKind::Text);
        assert!(matches!(chips[0].op, EditOp::ReplaceNode { .. }));
        // The unchanged Container produced no chip (its child changed, not itself).
        assert!(chips[0].label.contains("text changed"));
    }

    #[test]
    fn a_modifier_change_is_a_style_chip() {
        let base = page("Container { Button(\"Go\") }");
        let prop = page("Container { Button(\"Go\", primary) }");
        let chips = diff(&base, &prop).unwrap();
        assert_eq!(chips.len(), 1);
        assert_eq!(chips[0].kind, ChipKind::Style);
    }

    #[test]
    fn an_appended_child_is_an_added_structure_chip() {
        let base = page("Container { Text(\"a\") }");
        let prop = page("Container { Text(\"a\") Button(\"b\") }");
        let chips = diff(&base, &prop).unwrap();
        assert_eq!(chips.len(), 1, "only the added node, not its unchanged parent");
        assert_eq!(chips[0].kind, ChipKind::Structure);
        assert!(chips[0].label.starts_with("Added Button"));
        assert!(matches!(chips[0].op, EditOp::InsertChild { .. }));
    }

    #[test]
    fn a_removed_trailing_child_is_a_removed_chip() {
        let base = page("Container { Text(\"a\") Button(\"b\") }");
        let prop = page("Container { Text(\"a\") }");
        let chips = diff(&base, &prop).unwrap();
        assert_eq!(chips.len(), 1);
        assert!(chips[0].label.starts_with("Removed Button"));
        assert!(matches!(chips[0].op, EditOp::RemoveNode { .. }));
    }

    #[test]
    fn retyping_an_element_is_a_structure_chip() {
        let base = page("Container { Text(\"x\") }");
        let prop = page("Container { Heading(\"x\", h1) }");
        let chips = diff(&base, &prop).unwrap();
        assert_eq!(chips.len(), 1);
        assert_eq!(chips[0].kind, ChipKind::Structure);
        assert!(chips[0].label.contains("Changed Text to Heading"));
    }
}
