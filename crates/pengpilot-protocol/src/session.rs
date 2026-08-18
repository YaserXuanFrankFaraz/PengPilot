use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub fn unix_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

pub fn unix_time_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

/// Filesystem context a task runs in.
///
/// Drafts may carry [`Self::NewWorktree`] until their first prompt. Waku then
/// creates the Git worktree and replaces it with [`Self::Worktree`] before any
/// checkpoint or provider process can observe the task.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "kind"
)]
pub enum SessionWorkspace {
    /// Work directly in the project's ordinary checkout.
    #[default]
    Local,
    /// Create an isolated worktree when this draft is first submitted. A
    /// selected base branch is remembered without checking it out in the
    /// ordinary project directory.
    NewWorktree {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        base_branch: Option<String>,
    },
    /// A materialized worktree. `path` preserves a project that points at a
    /// subdirectory of its repository rather than the repository root itself.
    Worktree { path: PathBuf, branch: String },
}

impl SessionWorkspace {
    pub fn is_local(&self) -> bool {
        matches!(self, Self::Local)
    }

    pub fn is_worktree(&self) -> bool {
        matches!(self, Self::NewWorktree { .. } | Self::Worktree { .. })
    }

    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::Worktree { path, .. } => Some(path),
            Self::Local | Self::NewWorktree { .. } => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SessionStatus {
    #[default]
    Idle,
    Connecting,
    Working,
    Waiting,
    Failed,
}

impl SessionStatus {
    pub fn is_busy(self) -> bool {
        matches!(self, Self::Connecting | Self::Working | Self::Waiting)
    }
}

/// A follow-up message queued while the agent is busy. It becomes its own
/// turn once the current turn settles successfully.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QueuedMessage {
    pub id: Uuid,
    pub content: String,
    /// The text typed before Waku appended provider-facing attachment
    /// mentions. `None` is the legacy/plain-message representation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_content: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<MessageAttachment>,
    pub created_at: u64,
}

impl QueuedMessage {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            content: content.into(),
            display_content: None,
            attachments: Vec::new(),
            created_at: unix_time(),
        }
    }

    pub fn with_presentation(
        content: impl Into<String>,
        display_content: Option<String>,
        attachments: Vec<MessageAttachment>,
    ) -> Self {
        Self {
            display_content,
            attachments,
            ..Self::new(content)
        }
    }

    pub fn visible_content(&self) -> &str {
        self.display_content.as_deref().unwrap_or(&self.content)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TurnStatus {
    Running,
    Completed,
    Failed,
    Interrupted,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CheckpointStatus {
    Ready,
    Unavailable,
    Error,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CheckpointFile {
    pub path: String,
    pub additions: u64,
    pub deletions: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Checkpoint {
    pub turn_count: usize,
    pub git_ref: String,
    pub status: CheckpointStatus,
    #[serde(default)]
    pub files: Vec<CheckpointFile>,
    /// Cached once at capture time so a visible transcript row never walks a
    /// potentially huge file list on every frame.
    #[serde(default)]
    pub additions: u64,
    #[serde(default)]
    pub deletions: u64,
    pub created_at: u64,
}

impl Checkpoint {
    pub fn refresh_totals(&mut self) {
        self.additions = self.files.iter().map(|file| file.additions).sum();
        self.deletions = self.files.iter().map(|file| file.deletions).sum();
    }

    pub fn totals_are_current(&self) -> bool {
        self.additions == self.files.iter().map(|file| file.additions).sum::<u64>()
            && self.deletions == self.files.iter().map(|file| file.deletions).sum::<u64>()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AgentTurn {
    pub id: Uuid,
    pub turn_count: usize,
    pub status: TurnStatus,
    #[serde(default)]
    pub provider_turn_started: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_resume_at: Option<String>,
    pub started_at: u64,
    pub completed_at: Option<u64>,
    #[serde(default)]
    pub checkpoint: Option<Checkpoint>,
}

/// How full the provider's context window is, from the latest main-thread
/// model call. `tokens` is prompt + cache + output of that call; `window` is
/// the model's context size, which the provider only reports once a turn
/// settles — `None` means "not known yet", and the meter degrades to a bare
/// token count.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ContextUsage {
    pub tokens: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window: Option<u64>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// A file represented by a composer chip and retained with the sent message.
///
/// Render paths consume only this cached metadata; they never stat the file.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MessageAttachment {
    /// Absolute file path used by the thumbnail and handed to the provider.
    pub path: PathBuf,
    /// Provider-facing path text, relative to the workspace when possible.
    pub mention: String,
    pub name: String,
    pub is_dir: bool,
    pub is_image: bool,
    /// Clipboard images live in Waku's blob store. Keeping the reference in
    /// persisted metadata prevents the blob collector from reclaiming them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blob_reference: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message {
    pub id: Uuid,
    #[serde(default)]
    pub turn_id: Option<Uuid>,
    pub role: MessageRole,
    pub content: String,
    /// User-visible text before provider-facing attachment mentions were
    /// appended. Plain and legacy messages omit it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_content: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<MessageAttachment>,
    pub created_at: u64,
    pub streaming: bool,
}

impl Message {
    pub fn new(role: MessageRole, content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            turn_id: None,
            role,
            content: content.into(),
            display_content: None,
            attachments: Vec::new(),
            created_at: unix_time(),
            streaming: false,
        }
    }

    pub fn new_for_turn(role: MessageRole, content: impl Into<String>, turn_id: Uuid) -> Self {
        Self {
            turn_id: Some(turn_id),
            ..Self::new(role, content)
        }
    }

    pub fn with_presentation(
        mut self,
        display_content: Option<String>,
        attachments: Vec<MessageAttachment>,
    ) -> Self {
        self.display_content = display_content;
        self.attachments = attachments;
        self
    }

    pub fn visible_content(&self) -> &str {
        self.display_content.as_deref().unwrap_or(&self.content)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ActivityKind {
    Reasoning,
    Command,
    FileChange,
    FileRead,
    FileSearch,
    FileList,
    Search,
    Plan,
    Tool,
}

impl ActivityKind {
    /// Classifies provider tool names without mistaking unrelated MCP tools
    /// such as `create_thread` or `read_mcp_resource` for file operations.
    pub fn from_tool_name(name: &str) -> Self {
        let normalized = name.trim().to_ascii_lowercase().replace(['-', ' '], "_");
        let leaf = normalized
            .rsplit("__")
            .next()
            .unwrap_or(&normalized)
            .rsplit([':', '.', '/'])
            .next()
            .unwrap_or(&normalized);
        let compact = leaf.replace('_', "");

        if matches!(
            compact.as_str(),
            "todo" | "todowrite" | "updateplan" | "plan"
        ) {
            Self::Plan
        } else if matches!(
            compact.as_str(),
            "bash"
                | "command"
                | "execute"
                | "executecommand"
                | "commandexecution"
                | "runcommand"
                | "runterminalcommand"
                | "shell"
                | "shellcommand"
                | "terminal"
        ) {
            Self::Command
        } else if matches!(
            compact.as_str(),
            "applypatch"
                | "create"
                | "createfile"
                | "delete"
                | "deletefile"
                | "edit"
                | "filechange"
                | "fileedit"
                | "editfile"
                | "move"
                | "movefile"
                | "multiedit"
                | "notebookedit"
                | "patch"
                | "rename"
                | "renamefile"
                | "replace"
                | "savefile"
                | "strreplace"
                | "write"
                | "writefile"
        ) {
            Self::FileChange
        } else if matches!(
            compact.as_str(),
            "read" | "fileread" | "readfile" | "readtextfile" | "viewfile"
        ) {
            Self::FileRead
        } else if matches!(
            compact.as_str(),
            "filesearch"
                | "find"
                | "findfiles"
                | "glob"
                | "grep"
                | "ripgrep"
                | "searchfiles"
                | "searchinfiles"
        ) {
            Self::FileSearch
        } else if matches!(
            compact.as_str(),
            "directorylist"
                | "filelist"
                | "list"
                | "listdirectory"
                | "listfiles"
                | "ls"
                | "readdir"
        ) {
            Self::FileList
        } else if matches!(
            compact.as_str(),
            "search" | "searchtool" | "webfetch" | "websearch"
        ) {
            Self::Search
        } else {
            Self::Tool
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_names_are_classified_without_substring_false_positives() {
        for name in [
            "read",
            "ReadFile",
            "read_text_file",
            "mcp__filesystem__read_file",
        ] {
            assert_eq!(
                ActivityKind::from_tool_name(name),
                ActivityKind::FileRead,
                "{name}"
            );
        }
        for name in ["grep", "Glob", "fileSearch", "search_files"] {
            assert_eq!(
                ActivityKind::from_tool_name(name),
                ActivityKind::FileSearch,
                "{name}"
            );
        }
        for name in ["ls", "ListDirectory", "read_dir"] {
            assert_eq!(
                ActivityKind::from_tool_name(name),
                ActivityKind::FileList,
                "{name}"
            );
        }
        for name in ["WriteFile", "applyPatch", "move_file", "str_replace"] {
            assert_eq!(
                ActivityKind::from_tool_name(name),
                ActivityKind::FileChange,
                "{name}"
            );
        }
        for name in ["create_thread", "read_mcp_resource", "list_threads"] {
            assert_eq!(
                ActivityKind::from_tool_name(name),
                ActivityKind::Tool,
                "{name}"
            );
        }
    }
}
