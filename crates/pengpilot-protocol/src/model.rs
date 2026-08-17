use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ProviderKind {
    Amp,
    Claude,
    #[default]
    Codex,
    Cursor,
    DeepSeek,
    OpenCode,
    Grok,
    Pi,
    Omp,
    Kiro,
    Hermes,
    Trae,
}

impl ProviderKind {
    /// Public catalog for new work. Other enum variants remain so older
    /// sessions still deserialize and continue running.
    pub const FEATURED: [Self; 11] = [
        Self::Claude,
        Self::Codex,
        Self::Cursor,
        Self::OpenCode,
        Self::Grok,
        Self::Kiro,
        Self::Trae,
        Self::DeepSeek,
        Self::Hermes,
        Self::Pi,
        Self::Omp,
    ];
    /// Runtime integrations retained for current and legacy sessions.
    pub const ALL: [Self; 11] = [
        Self::Claude,
        Self::Codex,
        Self::Cursor,
        Self::OpenCode,
        Self::Grok,
        Self::Kiro,
        Self::Trae,
        Self::DeepSeek,
        Self::Hermes,
        Self::Pi,
        Self::Omp,
    ];

    pub fn is_featured(self) -> bool {
        Self::FEATURED.contains(&self)
    }

    pub fn for_new_work(self) -> Self {
        if self.is_featured() {
            self
        } else {
            Self::Codex
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::Amp => "amp",
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Cursor => "cursor",
            Self::DeepSeek => "deepseek",
            Self::OpenCode => "opencode",
            Self::Grok => "grok",
            Self::Pi => "pi",
            Self::Omp => "omp",
            Self::Kiro => "kiro",
            Self::Hermes => "hermes",
            Self::Trae => "trae",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Amp => "Amp",
            Self::Claude => "Claude Code (CLI)",
            Self::Codex => "Codex (CLI)",
            Self::Cursor => "Cursor (CLI)",
            Self::DeepSeek => "DeepSeek Harness",
            Self::OpenCode => "OpenCode (CLI)",
            Self::Grok => "Grok Build (CLI)",
            Self::Pi => "Pi (CLI)",
            Self::Omp => "Oh My Pi (CLI)",
            Self::Kiro => "Kiro (CLI)",
            Self::Hermes => "Hermes Agent (CLI)",
            Self::Trae => "Trae (CLI)",
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            Self::Amp => "Amp",
            Self::Claude => "Claude",
            Self::Codex => "Codex",
            Self::Cursor => "Cursor",
            Self::DeepSeek => "DeepSeek",
            Self::OpenCode => "OpenCode",
            Self::Grok => "Grok",
            Self::Pi => "Pi",
            Self::Omp => "OMP",
            Self::Kiro => "Kiro",
            Self::Hermes => "Hermes",
            Self::Trae => "Trae",
        }
    }

    pub fn command(self) -> &'static str {
        match self {
            Self::Amp => "amp",
            Self::Claude => "claude",
            Self::Codex => "codex",
            // Cursor documents `agent` as its primary command, but that name is
            // shared by other CLIs. The backward-compatible alias is unambiguous.
            Self::Cursor => "cursor-agent",
            Self::DeepSeek => "dsh",
            Self::OpenCode => "opencode",
            Self::Grok => "grok",
            Self::Pi => "pi",
            Self::Omp => "omp",
            Self::Kiro => "kiro-cli",
            Self::Hermes => "hermes",
            Self::Trae => "traecli",
        }
    }

    pub fn supports_conversation_rollback(self) -> bool {
        matches!(
            self,
            Self::Amp
                | Self::Claude
                | Self::Codex
                | Self::Cursor
                | Self::DeepSeek
                | Self::OpenCode
                | Self::Grok
                | Self::Pi
        )
    }

    pub fn supports_conversation_fork(self) -> bool {
        matches!(
            self,
            Self::Amp
                | Self::Claude
                | Self::Codex
                | Self::Cursor
                | Self::DeepSeek
                | Self::OpenCode
                | Self::Grok
                | Self::Pi
        )
    }

    pub fn supports_model_discovery(self) -> bool {
        matches!(
            self,
            Self::Codex
                | Self::Cursor
                | Self::DeepSeek
                | Self::OpenCode
                | Self::Grok
                | Self::Pi
                | Self::Omp
                | Self::Kiro
                | Self::Hermes
        )
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "provider"
)]
pub enum ProviderResumeCursor {
    Amp {
        thread_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fork_context: Option<String>,
    },
    Claude {
        session_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        resume_at: Option<String>,
    },
    Codex {
        thread_id: String,
    },
    Cursor {
        session_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fork_context: Option<String>,
    },
    OpenCode {
        session_id: String,
    },
    DeepSeek {
        session_id: String,
    },
    Grok {
        session_id: String,
    },
    Pi {
        session_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        session_file: Option<PathBuf>,
    },
    Omp {
        session_id: String,
    },
    Kiro {
        session_id: String,
    },
    Hermes {
        session_id: String,
    },
    /// Session identity for providers that share one transport implementation.
    External {
        kind: ProviderKind,
        session_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        session_file: Option<PathBuf>,
    },
}

impl ProviderResumeCursor {
    pub fn from_session_id(provider: ProviderKind, id: String) -> Self {
        match provider {
            ProviderKind::Amp => Self::Amp {
                thread_id: id,
                fork_context: None,
            },
            ProviderKind::Claude => Self::Claude {
                session_id: id,
                resume_at: None,
            },
            ProviderKind::Codex => Self::Codex { thread_id: id },
            ProviderKind::Cursor => Self::Cursor {
                session_id: id,
                fork_context: None,
            },
            ProviderKind::DeepSeek => Self::DeepSeek { session_id: id },
            ProviderKind::OpenCode => Self::OpenCode { session_id: id },
            ProviderKind::Grok => Self::Grok { session_id: id },
            ProviderKind::Pi => Self::Pi {
                session_id: id,
                session_file: None,
            },
            ProviderKind::Omp => Self::Omp { session_id: id },
            ProviderKind::Kiro => Self::Kiro { session_id: id },
            ProviderKind::Hermes => Self::Hermes { session_id: id },
            kind => Self::External {
                kind,
                session_id: id,
                session_file: None,
            },
        }
    }

    pub fn provider(&self) -> ProviderKind {
        match self {
            Self::Amp { .. } => ProviderKind::Amp,
            Self::Claude { .. } => ProviderKind::Claude,
            Self::Codex { .. } => ProviderKind::Codex,
            Self::Cursor { .. } => ProviderKind::Cursor,
            Self::DeepSeek { .. } => ProviderKind::DeepSeek,
            Self::OpenCode { .. } => ProviderKind::OpenCode,
            Self::Grok { .. } => ProviderKind::Grok,
            Self::Pi { .. } => ProviderKind::Pi,
            Self::Omp { .. } => ProviderKind::Omp,
            Self::Kiro { .. } => ProviderKind::Kiro,
            Self::Hermes { .. } => ProviderKind::Hermes,
            Self::External { kind, .. } => *kind,
        }
    }

    pub fn native_id(&self) -> &str {
        match self {
            Self::Amp { thread_id, .. } => thread_id,
            Self::Claude { session_id, .. }
            | Self::Cursor { session_id, .. }
            | Self::DeepSeek { session_id }
            | Self::OpenCode { session_id }
            | Self::Grok { session_id }
            | Self::Pi { session_id, .. }
            | Self::Omp { session_id } => session_id,
            Self::Kiro { session_id } | Self::Hermes { session_id } => session_id,
            Self::External { session_id, .. } => session_id,
            Self::Codex { thread_id } => thread_id,
        }
    }
}
