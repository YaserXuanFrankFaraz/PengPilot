//! Desktop persistence: app files stay local, task data crosses RPC.

use std::io;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use parking_lot::Mutex;
use uuid::Uuid;

pub use pengpilot_core::persistence::{
    ComposerDraft, ComposerDraftAttachment, ComposerDraftKey, ComposerDraftStore, ComposerDrafts,
    DEFAULT_RIGHT_PANEL_WIDTH, DEFAULT_SIDEBAR_WIDTH, PersistedState, PersistedWindowState,
    SessionMessageMatch,
};

use crate::model::{AgentSession, Project};

/// Desktop state store: settings stay local, the task catalog is daemon-owned.
pub struct StateStore {
    local: pengpilot_core::persistence::StateStore,
    daemon: pengpilot_client::DaemonSupervisor,
    remote_default_cwd: Mutex<Option<PathBuf>>,
    /// A task snapshot may only be written after this client has successfully
    /// loaded the daemon's authoritative state.
    task_state_loaded: AtomicBool,
}

impl Deref for StateStore {
    type Target = pengpilot_core::persistence::StateStore;

    fn deref(&self) -> &Self::Target {
        &self.local
    }
}

impl StateStore {
    pub fn default_path() -> PathBuf {
        pengpilot_core::persistence::StateStore::default_path()
    }

    pub fn remote(daemon: pengpilot_client::DaemonSupervisor) -> Self {
        Self {
            local: pengpilot_core::persistence::StateStore::new(Self::default_path()),
            daemon,
            remote_default_cwd: Mutex::new(None),
            task_state_loaded: AtomicBool::new(false),
        }
    }

    pub fn load_or_fresh(&self, cwd: PathBuf) -> PersistedState {
        let mut state = match self.load() {
            Ok(state) => {
                self.task_state_loaded.store(true, Ordering::Release);
                state
            }
            Err(_) if cwd.parent().is_none() => PersistedState::empty(),
            Err(_) => PersistedState::fresh(cwd),
        };
        if state.projects.is_empty()
            && let Some(cwd) = self.remote_default_cwd.lock().clone()
            && cwd.parent().is_some()
        {
            let project = Project::from_path(cwd);
            let session = state.new_session(project.id, state.last_provider);
            state.selected_project = Some(project.id);
            state.selected_session = Some(session.id);
            state.projects.push(project);
            state.push_session(session);
        }
        state.ensure_runtime_session();
        if let Some(selected) = state.selected_session
            && let Some(session) = state
                .sessions
                .iter_mut()
                .find(|session| session.id == selected)
        {
            let _ = self.hydrate(session);
        }
        state
    }

    pub fn load(&self) -> io::Result<PersistedState> {
        let (projects, mut sessions, default_cwd) = match self
            .daemon
            .client()
            .request(Uuid::nil(), Uuid::nil(), pengpilot_client::Command::LoadTaskState)
            .map_err(to_io_error)?
        {
            pengpilot_client::ResponsePayload::TaskState {
                projects,
                sessions,
                default_cwd,
                projectless_root: _,
            } => (projects, sessions, default_cwd),
            _ => {
                return Err(io::Error::other(
                    "PengPilot daemon returned an invalid task-state response",
                ));
            }
        };
        for session in &mut sessions {
            session.detail_loaded = false;
        }
        *self.remote_default_cwd.lock() = Some(default_cwd);
        let mut state = PersistedState::empty();
        state.projects = projects;
        state.sessions = sessions;
        self.local.overlay_desktop_files(&mut state)?;
        state.migrate_loaded();
        Ok(state)
    }

    pub fn hydrate(&self, session: &mut AgentSession) -> io::Result<()> {
        if session.detail_loaded {
            return Ok(());
        }
        match self
            .daemon
            .client()
            .request(
                Uuid::nil(),
                Uuid::nil(),
                pengpilot_client::Command::HydrateSession {
                    session_id: session.id,
                },
            )
            .map_err(to_io_error)?
        {
            pengpilot_client::ResponsePayload::Session {
                session: Some(stored),
            } => {
                *session = stored;
                Ok(())
            }
            pengpilot_client::ResponsePayload::Session { session: None } => {
                session.detail_loaded = true;
                Ok(())
            }
            _ => Err(io::Error::other(
                "PengPilot daemon returned an invalid session-hydration response",
            )),
        }
    }

    pub fn save(&self, state: &mut PersistedState) -> io::Result<()> {
        self.local.save_desktop_files(state)?;
        if !self.task_state_loaded.load(Ordering::Acquire) {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "task state was not loaded; refusing to overwrite daemon data",
            ));
        }
        let dirty_ids = state.dirty_session_ids();
        let sessions = state
            .sessions
            .iter()
            .filter(|session| dirty_ids.contains(&session.id))
            .cloned()
            .collect();
        let live_session_ids = state.sessions.iter().map(|session| session.id).collect();
        // ponytail: fire-and-forget like waku; quit still races a disk flush.
        self.daemon
            .client()
            .notify(
                Uuid::nil(),
                Uuid::nil(),
                pengpilot_client::Command::SaveTaskState {
                    projects: state.projects.clone(),
                    live_session_ids,
                    sessions,
                },
            )
            .map_err(to_io_error)?;
        state.clear_dirty_sessions();
        Ok(())
    }

    pub fn remove_session(&self, session_id: Uuid) -> io::Result<()> {
        self.daemon
            .client()
            .notify(session_id, Uuid::nil(), pengpilot_client::Command::RemoveSession)
            .map_err(to_io_error)
    }
}

fn to_io_error(error: anyhow::Error) -> io::Error {
    io::Error::other(error)
}
