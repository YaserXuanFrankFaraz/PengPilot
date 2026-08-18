use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use pengpilot_protocol::agent::{
    ActivityFileChange, ActivityFileChangeStatus, ActivityItem, AgentSession, BackgroundWorkEvent,
    BackgroundWorkItem, BackgroundWorkKey, BackgroundWorkKind, BackgroundWorkStatus,
    PendingPermission, PermissionOption, ReasoningBlock, ReportedCommand, TranscriptBlock,
    UserInputAnswer, UserInputOption, UserInputQuestion, compact_path, is_generic_activity_title,
};
pub use pengpilot_protocol::model::{
    FavoriteModel, InteractionMode, ProviderAgentPreset, ProviderKind, ProviderModel,
    ProviderModelOption, ProviderProbe, ProviderResumeCursor, RuntimeMode,
};
pub use pengpilot_protocol::session::{
    ActivityKind, AgentTurn, Checkpoint, CheckpointFile, CheckpointStatus, ContextUsage, Message,
    MessageAttachment, MessageRole, Project, QueuedMessage, SessionStatus, SessionWorkspace,
    TurnStatus, unix_time, unix_time_millis,
};

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
    AvailableCommands(Vec<ReportedCommand>),
    TurnStarted,
    TextDelta(String),
    ReasoningDelta(String),
    Activity {
        id: Option<String>,
        kind: ActivityKind,
        title: String,
        detail: Option<String>,
        complete: bool,
    },
    RichActivity(ActivityItem),
    /// Session-level work that can outlive the turn which created it. This is
    /// deliberately separate from transcript activities: completing a turn
    /// must not make a detached process or subagent look complete.
    BackgroundWork(BackgroundWorkEvent),
    Permission {
        request_id: String,
        title: String,
        detail: String,
        options: Vec<PermissionOption>,
    },
    /// Structured questions the provider needs answered before it can
    /// continue the active turn. Unlike a permission, this is never
    /// auto-approved: the content itself has to come from the user.
    UserInputRequested {
        request_id: String,
        questions: Vec<UserInputQuestion>,
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
    /// merges them into [`ContextUsage`].
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

