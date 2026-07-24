//! The proposal model (IMPLEMENTATION_PLAN §3.3/§4.5).
//!
//! An edit produces a [`Proposal`] — the base source, the proposed source, and the
//! [`Chip`]s between them — instead of mutating the live document. The user
//! accepts/rejects chips; [`Proposal::apply_accepted`] re-serializes the accepted
//! subset. "Reset" is just dropping the proposal (no method needed).

use webfluent::{apply_edits, EditOp};

use crate::{compile_source, diff, Chip};

/// A pending edit under review: base vs. proposal, the chips, and each chip's
/// accept state (all accepted by default).
pub struct Proposal {
    base: String,
    proposal: String,
    chips: Vec<Chip>,
    accepted: Vec<bool>,
}

impl Proposal {
    /// Build a proposal by diffing `base` against `proposal` (both must parse).
    pub fn new(base: impl Into<String>, proposal: impl Into<String>) -> anyhow::Result<Self> {
        let base = base.into();
        let proposal = proposal.into();
        let chips = diff(&base, &proposal)?;
        let accepted = vec![true; chips.len()];
        Ok(Self { base, proposal, chips, accepted })
    }

    pub fn chips(&self) -> &[Chip] {
        &self.chips
    }
    pub fn base(&self) -> &str {
        &self.base
    }
    pub fn proposal(&self) -> &str {
        &self.proposal
    }
    pub fn len(&self) -> usize {
        self.chips.len()
    }
    pub fn is_empty(&self) -> bool {
        self.chips.is_empty()
    }
    pub fn is_accepted(&self, i: usize) -> bool {
        self.accepted.get(i).copied().unwrap_or(false)
    }
    pub fn set_accepted(&mut self, i: usize, value: bool) {
        if let Some(a) = self.accepted.get_mut(i) {
            *a = value;
        }
    }
    pub fn toggle(&mut self, i: usize) {
        if let Some(a) = self.accepted.get_mut(i) {
            *a = !*a;
        }
    }
    pub fn accepted_count(&self) -> usize {
        self.accepted.iter().filter(|&&a| a).count()
    }

    /// The source after applying the accepted chips to the base:
    /// - **all** accepted → the proposal itself (already compiled — the exact,
    ///   overlap-free result the edit produced);
    /// - **none** → the base unchanged;
    /// - a **subset** → the accepted chips' ops applied to the base and re-validated.
    ///
    /// A subset of chips whose spans overlap (an ancestor `ReplaceNode` plus a
    /// descendant op — rare in scoped edits) can fail the reparse-guard; that
    /// surfaces as an error rather than corrupting the source.
    pub fn apply_accepted(&self) -> anyhow::Result<String> {
        let all = !self.chips.is_empty() && self.accepted.iter().all(|&a| a);
        if all {
            return Ok(self.proposal.clone());
        }
        let ops: Vec<EditOp> = self
            .chips
            .iter()
            .zip(&self.accepted)
            .filter_map(|(c, &a)| a.then(|| c.op.clone()))
            .collect();
        if ops.is_empty() {
            return Ok(self.base.clone());
        }
        let edited = apply_edits(&self.base, &ops).map_err(|e| anyhow::anyhow!("{e}"))?;
        compile_source(&edited)?; // partial applications must still compile
        Ok(edited)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "Page P (path: \"/\") { Container { Heading(\"Hi\", h1) Button(\"Go\") } }";
    // Two independent changes: the heading's text and the button's style.
    const PROPOSAL: &str = "Page P (path: \"/\") { Container { Heading(\"Hello\", h1) Button(\"Go\", primary) } }";

    #[test]
    fn new_diffs_into_chips_accepted_by_default() {
        let p = Proposal::new(BASE, PROPOSAL).unwrap();
        assert_eq!(p.len(), 2);
        assert_eq!(p.accepted_count(), 2);
        assert!(p.is_accepted(0) && p.is_accepted(1));
    }

    #[test]
    fn accept_all_returns_the_proposal() {
        let p = Proposal::new(BASE, PROPOSAL).unwrap();
        let out = p.apply_accepted().unwrap();
        assert!(out.contains("Hello"));
        assert!(out.contains("primary"));
    }

    #[test]
    fn reject_all_returns_the_base() {
        let mut p = Proposal::new(BASE, PROPOSAL).unwrap();
        p.set_accepted(0, false);
        p.set_accepted(1, false);
        let out = p.apply_accepted().unwrap();
        assert!(out.contains("Hi") && !out.contains("Hello"));
        assert!(!out.contains("primary"));
    }

    #[test]
    fn accept_a_subset_applies_only_those_chips() {
        let mut p = Proposal::new(BASE, PROPOSAL).unwrap();
        // chips are sorted by node id: [0]=Heading text (P:0.0), [1]=Button style (P:0.1).
        p.set_accepted(1, false); // keep the heading text, drop the button style
        let out = p.apply_accepted().unwrap();
        assert!(out.contains("Hello"), "accepted heading change landed");
        assert!(!out.contains("primary"), "rejected button change did not");
    }

    #[test]
    fn a_no_op_proposal_has_no_chips() {
        let p = Proposal::new(BASE, BASE).unwrap();
        assert!(p.is_empty());
        assert_eq!(p.apply_accepted().unwrap(), BASE);
    }
}
