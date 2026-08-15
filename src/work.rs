//! Progress, Eisenhower quadrant, and collection membership for local work.
//!
//! Workflow status is independent of [`crate::model::SessionStatus`]: the
//! latter is the live process, this is how far the work has moved.

use serde::{Deserialize, Serialize};

/// How far a piece of work has moved. Completing archives it.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WorkflowStatus {
    #[default]
    Todo,
    InProgress,
    InReview,
    Done,
}

impl WorkflowStatus {
    pub const LIVE: [Self; 3] = [Self::Todo, Self::InProgress, Self::InReview];

    pub fn is_live(self) -> bool {
        !matches!(self, Self::Done)
    }

    pub fn is_archived(self) -> bool {
        matches!(self, Self::Done)
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Todo => "workflow.todo",
            Self::InProgress => "workflow.in_progress",
            Self::InReview => "workflow.in_review",
            Self::Done => "workflow.done",
        }
    }
}

/// Important × urgent. Default new work is important and not urgent.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Quadrant {
    pub important: bool,
    pub urgent: bool,
}

impl Default for Quadrant {
    fn default() -> Self {
        Self {
            important: true,
            urgent: false,
        }
    }
}

impl Quadrant {
    pub const DO_NOW: Self = Self {
        important: true,
        urgent: true,
    };
    pub const SCHEDULE: Self = Self {
        important: true,
        urgent: false,
    };
    pub const DELEGATE: Self = Self {
        important: false,
        urgent: true,
    };
    pub const LATER: Self = Self {
        important: false,
        urgent: false,
    };

    pub fn label_key(self) -> &'static str {
        match (self.important, self.urgent) {
            (true, true) => "quadrant.do_now",
            (true, false) => "quadrant.schedule",
            (false, true) => "quadrant.delegate",
            (false, false) => "quadrant.later",
        }
    }

    pub fn hint_key(self) -> &'static str {
        match (self.important, self.urgent) {
            (true, true) => "quadrant.do_now_hint",
            (true, false) => "quadrant.schedule_hint",
            (false, true) => "quadrant.delegate_hint",
            (false, false) => "quadrant.later_hint",
        }
    }
}

/// Keyboard focus among the three shell zones.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FocusZone {
    Nav,
    List,
    #[default]
    Detail,
}

/// Which list the user is looking at. The board is a view of unfinished work.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum InboxCollection {
    #[default]
    Unfinished,
    Flagged,
    Archive,
}

impl InboxCollection {
    pub fn contains(self, workflow: WorkflowStatus, flagged: bool) -> bool {
        match self {
            Self::Unfinished => workflow.is_live(),
            Self::Flagged => flagged,
            Self::Archive => workflow.is_archived(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn done_is_the_only_archived_stage() {
        assert!(WorkflowStatus::Todo.is_live());
        assert!(WorkflowStatus::InProgress.is_live());
        assert!(WorkflowStatus::InReview.is_live());
        assert!(WorkflowStatus::Done.is_archived());
        assert!(!WorkflowStatus::Done.is_live());
    }

    #[test]
    fn unfinished_excludes_done() {
        assert!(InboxCollection::Unfinished.contains(WorkflowStatus::Todo, false));
        assert!(InboxCollection::Unfinished.contains(WorkflowStatus::InReview, true));
        assert!(!InboxCollection::Unfinished.contains(WorkflowStatus::Done, false));
        assert!(!InboxCollection::Unfinished.contains(WorkflowStatus::Done, true));
    }

    #[test]
    fn archive_is_done_only() {
        assert!(InboxCollection::Archive.contains(WorkflowStatus::Done, false));
        assert!(!InboxCollection::Archive.contains(WorkflowStatus::Todo, false));
    }

    #[test]
    fn flagged_is_independent_of_progress() {
        assert!(InboxCollection::Flagged.contains(WorkflowStatus::Todo, true));
        assert!(InboxCollection::Flagged.contains(WorkflowStatus::Done, true));
        assert!(!InboxCollection::Flagged.contains(WorkflowStatus::InProgress, false));
    }

    #[test]
    fn default_quadrant_is_important_not_urgent() {
        let q = Quadrant::default();
        assert!(q.important);
        assert!(!q.urgent);
        assert_eq!(q, Quadrant::SCHEDULE);
    }
}
