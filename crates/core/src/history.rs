//! Edit history (FR-14 / §4.7): a linear revision log of project sources with
//! undo/redo and one-click restore. Each accepted change / generation records a
//! revision with a human summary (never git vocabulary — FR-14). In-memory here;
//! a future gix-backed store can persist the same log to a hidden repo.

use std::collections::BTreeMap;

/// A project's `.wf` sources by path — the unit a revision snapshots.
pub type Sources = BTreeMap<String, String>;

/// One point in history: a human summary + the full source snapshot at that point.
#[derive(Debug, Clone)]
pub struct Revision {
    pub summary: String,
    pub sources: Sources,
}

/// A linear undo/redo history with a cursor at the current revision. `checkpoint`
/// records a new tip (discarding any redo tail); `undo`/`redo` move the cursor;
/// `restore` jumps to any revision.
#[derive(Debug, Default)]
pub struct History {
    revisions: Vec<Revision>,
    cursor: usize,
}

impl History {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record `sources` as a new revision at the tip. If earlier revisions were
    /// undone, the redo tail is discarded (a new branch replaces it).
    pub fn checkpoint(&mut self, summary: impl Into<String>, sources: Sources) {
        if !self.revisions.is_empty() {
            self.revisions.truncate(self.cursor + 1);
        }
        self.revisions.push(Revision { summary: summary.into(), sources });
        self.cursor = self.revisions.len() - 1;
    }

    pub fn is_empty(&self) -> bool {
        self.revisions.is_empty()
    }
    pub fn len(&self) -> usize {
        self.revisions.len()
    }
    /// The index of the current revision.
    pub fn cursor(&self) -> usize {
        self.cursor
    }
    /// All revisions, oldest first.
    pub fn entries(&self) -> &[Revision] {
        &self.revisions
    }
    pub fn current(&self) -> Option<&Revision> {
        self.revisions.get(self.cursor)
    }

    pub fn can_undo(&self) -> bool {
        self.cursor > 0
    }
    pub fn can_redo(&self) -> bool {
        self.cursor + 1 < self.revisions.len()
    }

    /// Step back one revision, returning it (its sources to restore).
    pub fn undo(&mut self) -> Option<&Revision> {
        if self.can_undo() {
            self.cursor -= 1;
            self.revisions.get(self.cursor)
        } else {
            None
        }
    }

    /// Step forward one revision.
    pub fn redo(&mut self) -> Option<&Revision> {
        if self.can_redo() {
            self.cursor += 1;
            self.revisions.get(self.cursor)
        } else {
            None
        }
    }

    /// Jump to revision `index` (a history-panel restore).
    pub fn restore(&mut self, index: usize) -> Option<&Revision> {
        if index < self.revisions.len() {
            self.cursor = index;
            self.revisions.get(index)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sources(marker: &str) -> Sources {
        let mut m = BTreeMap::new();
        m.insert("src/pages/Home.wf".to_string(), format!("Page Home {{ Text(\"{marker}\") }}"));
        m
    }

    #[test]
    fn empty_history_cannot_undo_or_redo() {
        let h = History::new();
        assert!(h.is_empty());
        assert!(!h.can_undo() && !h.can_redo());
        assert!(h.current().is_none());
    }

    #[test]
    fn checkpoints_advance_the_cursor() {
        let mut h = History::new();
        h.checkpoint("start", sources("a"));
        h.checkpoint("edit 1", sources("b"));
        h.checkpoint("edit 2", sources("c"));
        assert_eq!(h.len(), 3);
        assert_eq!(h.cursor(), 2);
        assert_eq!(h.current().unwrap().summary, "edit 2");
        assert!(h.can_undo() && !h.can_redo());
    }

    #[test]
    fn undo_and_redo_move_the_cursor() {
        let mut h = History::new();
        h.checkpoint("start", sources("a"));
        h.checkpoint("edit", sources("b"));
        assert_eq!(h.undo().unwrap().summary, "start");
        assert!(h.can_redo());
        assert!(h.current().unwrap().sources["src/pages/Home.wf"].contains("\"a\""));
        assert_eq!(h.redo().unwrap().summary, "edit");
        assert!(!h.can_redo());
    }

    #[test]
    fn undo_at_the_start_is_a_no_op() {
        let mut h = History::new();
        h.checkpoint("start", sources("a"));
        assert!(h.undo().is_none());
        assert_eq!(h.cursor(), 0);
    }

    #[test]
    fn a_checkpoint_after_undo_discards_the_redo_tail() {
        let mut h = History::new();
        h.checkpoint("v0", sources("a"));
        h.checkpoint("v1", sources("b"));
        h.checkpoint("v2", sources("c"));
        h.undo(); // now at v1
        h.checkpoint("v1b", sources("d")); // branches from v1
        assert_eq!(h.len(), 3, "v2 was discarded");
        assert_eq!(h.current().unwrap().summary, "v1b");
        assert!(!h.can_redo());
    }

    #[test]
    fn restore_jumps_to_a_revision() {
        let mut h = History::new();
        h.checkpoint("v0", sources("a"));
        h.checkpoint("v1", sources("b"));
        h.checkpoint("v2", sources("c"));
        let r = h.restore(0).unwrap();
        assert_eq!(r.summary, "v0");
        assert_eq!(h.cursor(), 0);
        assert!(h.can_redo());
    }
}
