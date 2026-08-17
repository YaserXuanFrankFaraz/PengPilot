//! The diff shown inside an expanded file-change activity.
//!
//! There is nothing to parse here: provider payloads are normalized into
//! unified-diff bodies when the tool event arrives, and
//! [`review_diff::from_file_changes`] turns those into the same positioned,
//! syntax-tokenized rows the Review panel reads. This module only decides how
//! much of that snapshot a transcript row should carry.

use crate::model::ActivityItem;
use crate::review_diff::{self, LineKind, Snapshot};

/// A diff inside a transcript row is a summary, not a review surface: past
/// this many rows, Review is where the change should be read.
const MAX_ROWS: usize = 400;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Diff {
    pub(super) snapshot: Snapshot,
    /// Rows dropped at [`MAX_ROWS`], so the card can say so instead of
    /// quietly presenting part of a change as the whole one.
    pub(super) hidden_rows: usize,
}

impl Diff {
    pub(super) fn is_empty(&self) -> bool {
        self.snapshot.lines.is_empty()
    }

    /// Whether any row knows where it sits in its file. Providers that only
    /// report before/after text leave every row unpositioned, and the gutter
    /// falls back to the `+`/`-` marker.
    #[cfg(test)]
    fn has_line_numbers(&self) -> bool {
        self.snapshot
            .lines
            .iter()
            .any(|line| line.old_line.is_some() || line.new_line.is_some())
    }
}

/// Build the rows for one activity's file changes.
///
/// Runs once when the activity is expanded — never from a row builder — and
/// the caller keeps the result until the activity's changes are replaced.
pub(super) fn build(activity: &ActivityItem) -> Diff {
    let mut snapshot = review_diff::from_file_changes(&activity.file_changes);
    // One file needs no header: the activity's own row already names it.
    if snapshot.files.len() < 2 {
        snapshot
            .lines
            .retain(|line| line.kind != LineKind::FileHeader);
    }
    let hidden_rows = snapshot.lines.len().saturating_sub(MAX_ROWS);
    snapshot.lines.truncate(MAX_ROWS);
    Diff {
        snapshot,
        hidden_rows,
    }
}
