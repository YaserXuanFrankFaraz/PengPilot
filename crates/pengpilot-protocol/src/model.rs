use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

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

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeMode {
    /// Legacy combined mode. State migration moves this to `interaction_mode`.
    Plan,
    Ask,
    AutoAcceptEdits,
    Auto,
    #[default]
    FullAccess,
}

impl RuntimeMode {
    pub const ACCESS_OPTIONS: [Self; 4] = [
        Self::Ask,
        Self::AutoAcceptEdits,
        Self::Auto,
        Self::FullAccess,
    ];

    pub fn label(self) -> String {
        match self {
            Self::Plan => tr!("mode.plan"),
            Self::Ask => tr!("mode.supervised"),
            Self::AutoAcceptEdits => tr!("mode.auto_accept_edits"),
            Self::Auto => tr!("mode.auto"),
            Self::FullAccess => tr!("mode.full_access"),
        }
    }

    pub fn description(self) -> String {
        match self {
            Self::Plan => tr!("mode.plan_description"),
            Self::Ask => tr!("mode.supervised_description"),
            Self::AutoAcceptEdits => tr!("mode.auto_accept_edits_description"),
            Self::Auto => tr!("mode.auto_description"),
            Self::FullAccess => tr!("mode.full_access_description"),
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Self::Plan | Self::Ask => "icons/lock.svg",
            Self::AutoAcceptEdits => "icons/pencil.svg",
            Self::Auto => "icons/sparkle.svg",
            Self::FullAccess => "icons/lock-open.svg",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum InteractionMode {
    #[default]
    Build,
    Plan,
}

impl InteractionMode {
    pub fn label(self) -> String {
        match self {
            Self::Build => tr!("mode.build"),
            Self::Plan => tr!("mode.plan"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProviderModelOption {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl ProviderModelOption {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            description: None,
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        let description = description.into();
        if !description.trim().is_empty() {
            self.description = Some(description);
        }
        self
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProviderModel {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sub_provider: Option<String>,
    #[serde(default)]
    pub is_default: bool,
    #[serde(default)]
    pub reasoning_efforts: Vec<ProviderModelOption>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_reasoning_effort: Option<String>,
    #[serde(default)]
    pub service_tiers: Vec<ProviderModelOption>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_service_tier: Option<String>,
    /// Context window sizes the provider exposes as a per-session choice.
    /// Claude Code keeps its 1M window opt-in behind a model-id suffix, so the
    /// window is a trait of the session rather than of the model.
    #[serde(default)]
    pub context_windows: Vec<ProviderModelOption>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_context_window: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FavoriteModel {
    pub provider: ProviderKind,
    pub model: String,
}

/// One provider-owned agent composition available when a task starts.
///
/// DeepSeek Harness calls these agent presets. They are intentionally kept
/// separate from [`InteractionMode`]: a preset chooses the tools and prompt
/// composition, while Build/Plan controls what that composition should do.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProviderAgentPreset {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub is_default: bool,
    #[serde(default)]
    pub is_custom: bool,
}

impl ProviderAgentPreset {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            is_default: false,
            is_custom: false,
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        let description = description.into();
        if !description.trim().is_empty() {
            self.description = Some(description);
        }
        self
    }

    pub fn default(mut self) -> Self {
        self.is_default = true;
        self
    }

    /// Harness localizes its four shipped presets in the Web client rather
    /// than in the Host roster, whose metadata may use the install language.
    /// Mirror that boundary while leaving user-authored metadata untouched.
    pub fn display_name(&self) -> String {
        if !self.is_custom {
            match self.id.as_str() {
                "standard" => return tr!("agent_preset.standard"),
                "code" => return tr!("agent_preset.code"),
                "minimal" => return tr!("agent_preset.minimal"),
                "cordis" => return tr!("agent_preset.creator"),
                _ => {}
            }
        }
        self.name.clone()
    }

    pub fn display_description(&self) -> Option<String> {
        if !self.is_custom {
            match self.id.as_str() {
                "standard" => return Some(tr!("agent_preset.standard_description")),
                "code" => return Some(tr!("agent_preset.code_description")),
                "minimal" => return Some(tr!("agent_preset.minimal_description")),
                "cordis" => return Some(tr!("agent_preset.creator_description")),
                _ => {}
            }
        }
        self.description.clone()
    }
}

impl ProviderModel {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            sub_provider: None,
            is_default: false,
            reasoning_efforts: Vec::new(),
            default_reasoning_effort: None,
            service_tiers: Vec::new(),
            default_service_tier: None,
            context_windows: Vec::new(),
            default_context_window: None,
        }
    }

    pub fn default(mut self) -> Self {
        self.is_default = true;
        self
    }

    pub fn sub_provider(mut self, sub_provider: impl Into<String>) -> Self {
        self.sub_provider = Some(sub_provider.into());
        self
    }

    pub fn reasoning(
        mut self,
        efforts: impl IntoIterator<Item = ProviderModelOption>,
        default: impl Into<String>,
    ) -> Self {
        self.reasoning_efforts = efforts.into_iter().collect();
        self.default_reasoning_effort = Some(default.into());
        self
    }

    pub fn service_tiers(
        mut self,
        tiers: impl IntoIterator<Item = ProviderModelOption>,
        default: impl Into<String>,
    ) -> Self {
        self.service_tiers = tiers.into_iter().collect();
        self.default_service_tier = Some(default.into());
        self
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProviderProbe {
    pub provider: ProviderKind,
    pub installed: bool,
    pub path: Option<PathBuf>,
    #[serde(default)]
    pub models: Vec<ProviderModel>,
    #[serde(default)]
    pub agent_presets: Vec<ProviderAgentPreset>,
}

impl ProviderProbe {
    pub fn preferred_model(&self) -> Option<&ProviderModel> {
        self.models
            .iter()
            .find(|model| model.is_default)
            .or_else(|| self.models.first())
    }

    pub fn preferred_agent_preset(&self) -> Option<&ProviderAgentPreset> {
        self.agent_presets
            .iter()
            .find(|preset| preset.is_default)
            .or_else(|| self.agent_presets.first())
    }
}

/// The daemon runtime and its replay journal can outlive any particular
/// desktop or browser connection. Persisting this cursor with the transcript
/// lets a newly attached client replay only the events the stored projection
/// has not already applied.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RuntimeEventCursor {
    pub runtime_id: Uuid,
    pub epoch: Uuid,
    pub sequence: u64,
}

#[derive(Clone, Debug)]
pub enum DriverEvent {
    /// Client-only acknowledgement that every driver event through this
    /// sequence has been incorporated into the local session projection.
    /// Providers never emit this and transports never serialize it.
    RuntimeEventCursorAdvanced(RuntimeEventCursor),
    Connected {
        provider_cursor: Option<ProviderResumeCursor>,
    },
    /// The provider-owned agent composition this session actually runs. A
    /// fresh Harness session may resolve its deployment default when Waku did
    /// not name one explicitly, so the driver reports the resolved value.
    AgentPresetSelected(Option<String>),
    /// A provider-owned, automatically generated session title. `None`
    /// clears that fallback but never overwrites a user-owned title.
    AutoTitleUpdated(Option<String>),
    /// The slash commands the live process itself reports — Claude's
    /// stream-json init handshake and ACP's `available_commands_update`.
    /// Authoritative over filesystem discovery, which cannot see plugin or
    /// dynamically registered commands.
    AvailableCommands(Vec<crate::agent::ReportedCommand>),
    TurnStarted,
    TextDelta(String),
    ReasoningDelta(String),
    Activity {
        id: Option<String>,
        kind: crate::session::ActivityKind,
        title: String,
        detail: Option<String>,
        complete: bool,
    },
    RichActivity(crate::agent::ActivityItem),
    /// Session-level work that can outlive the turn which created it. This is
    /// deliberately separate from transcript activities: completing a turn
    /// must not make a detached process or subagent look complete.
    BackgroundWork(crate::agent::BackgroundWorkEvent),
    Permission {
        request_id: String,
        title: String,
        detail: String,
        options: Vec<crate::agent::PermissionOption>,
    },
    /// Structured questions the provider needs answered before it can
    /// continue the active turn. Unlike a permission, this is never
    /// auto-approved: the content itself has to come from the user.
    UserInputRequested {
        request_id: String,
        questions: Vec<crate::agent::UserInputQuestion>,
    },
    ComputerUseUpdated(crate::computer_use::ComputerUseState),
    /// The provider accepted a steering message into the running turn.
    SteerAccepted {
        message: String,
    },
    /// The provider could not steer the running turn (for example it ended
    /// before the request arrived). The app decides the fallback.
    SteerRejected {
        message: String,
        reason: String,
    },
    /// Context-window occupancy reported by the live stream. Fields arrive at
    /// different moments — token counts with each assistant message, the
    /// window size with the settled turn — so each is optional and the app
    /// merges them into [`crate::session::ContextUsage`].
    UsageUpdated {
        context_tokens: Option<u64>,
        context_window: Option<u64>,
    },
    /// Account-level rate-limit meters carried by the provider's own stream
    /// (Codex's `account/rateLimits/updated`). Same shape the OAuth fetcher
    /// produces for Claude, so the panel renders both identically.
    PlanUsageUpdated(crate::usage::PlanUsage),
    TurnFinished {
        success: bool,
        summary: Option<String>,
    },
    Error(String),
    ProcessExited,
}
