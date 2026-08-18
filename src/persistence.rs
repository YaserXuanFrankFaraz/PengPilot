//! Desktop persistence: app files stay local, task data crosses RPC.

use std::io;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

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
    /// Sequence of the latest catalog save. Stale acks and superseded
    /// in-flight requests must not clear dirty sessions or write the daemon.
    save_seq: Arc<AtomicU64>,
    save_acks: Arc<Mutex<Vec<SaveAck>>>,
    save_pending: Arc<Mutex<Option<PendingSave>>>,
    save_inflight: Arc<AtomicBool>,
    /// Serializes `SaveTaskState` so a slower request cannot overwrite a
    /// newer snapshot on the daemon.
    save_rpc: Arc<Mutex<()>>,
}

struct SaveAck {
    seq: u64,
    dirty_generation: u64,
    result: Result<(), String>,
}

struct PendingSave {
    seq: u64,
    dirty_generation: u64,
    command: pengpilot_client::Command,
}

/// Result of applying finished catalog-save acks on the UI thread.
pub enum SaveAckDrain {
    None,
    Saved,
    Retry,
    Failed(String),
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
            save_seq: Arc::new(AtomicU64::new(0)),
            save_acks: Arc::new(Mutex::new(Vec::new())),
            save_pending: Arc::new(Mutex::new(None)),
            save_inflight: Arc::new(AtomicBool::new(false)),
            save_rpc: Arc::new(Mutex::new(())),
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
        self.save_task_state(state, true, None)
    }

    /// Desktop files on this thread; `SaveTaskState` on a worker so the UI
    /// pump is not blocked for the RPC round trip. Dirty sessions stay set
    /// until [`Self::take_save_acks`] sees a matching `TaskStateSaved`.
    pub fn save_async(
        &self,
        state: &mut PersistedState,
        wake: smol::channel::Sender<()>,
    ) -> io::Result<()> {
        self.save_task_state(state, false, Some(wake))
    }

    /// Apply finished catalog-save acks.
    pub fn take_save_acks(&self, state: &mut PersistedState) -> SaveAckDrain {
        let acks = std::mem::take(&mut *self.save_acks.lock());
        let latest = self.save_seq.load(Ordering::Acquire);
        let mut drain = SaveAckDrain::None;
        for ack in acks {
            if ack.seq != latest {
                continue;
            }
            match ack.result {
                Ok(()) if state.dirty_generation() == ack.dirty_generation => {
                    state.clear_dirty_sessions();
                    drain = SaveAckDrain::Saved;
                }
                Ok(()) => drain = SaveAckDrain::Retry,
                Err(message) => drain = SaveAckDrain::Failed(message),
            }
        }
        drain
    }

    fn save_task_state(
        &self,
        state: &mut PersistedState,
        blocking: bool,
        wake: Option<smol::channel::Sender<()>>,
    ) -> io::Result<()> {
        self.local.save_desktop_files(state)?;
        if !self.task_state_loaded.load(Ordering::Acquire) {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "task state was not loaded; refusing to overwrite daemon data",
            ));
        }
        let command = save_command(state);
        if blocking {
            self.save_seq.fetch_add(1, Ordering::AcqRel);
            self.save_pending.lock().take();
            let _rpc = self.save_rpc.lock();
            return match self
                .daemon
                .client()
                .request(Uuid::nil(), Uuid::nil(), command)
                .map_err(to_io_error)?
            {
                pengpilot_client::ResponsePayload::TaskStateSaved { .. } => {
                    state.clear_dirty_sessions();
                    Ok(())
                }
                _ => Err(io::Error::other(INVALID_SAVE_RESPONSE)),
            };
        }
        let seq = self.save_seq.fetch_add(1, Ordering::AcqRel) + 1;
        *self.save_pending.lock() = Some(PendingSave {
            seq,
            dirty_generation: state.dirty_generation(),
            command,
        });
        if self.save_inflight.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        let daemon = self.daemon.client();
        let acks = Arc::clone(&self.save_acks);
        let pending = Arc::clone(&self.save_pending);
        let inflight = Arc::clone(&self.save_inflight);
        let save_rpc = Arc::clone(&self.save_rpc);
        let save_seq = Arc::clone(&self.save_seq);
        std::thread::Builder::new()
            .name("pengpilot-save-task-state".into())
            .spawn(move || {
                loop {
                    let Some(job) = pending.lock().take() else {
                        inflight.store(false, Ordering::Release);
                        if pending.lock().is_some()
                            && !inflight.swap(true, Ordering::AcqRel)
                        {
                            continue;
                        }
                        if let Some(wake) = &wake {
                            let _ = wake.try_send(());
                        }
                        return;
                    };
                    if job.seq != save_seq.load(Ordering::Acquire) {
                        continue;
                    }
                    let _rpc = save_rpc.lock();
                    if job.seq != save_seq.load(Ordering::Acquire) {
                        continue;
                    }
                    let result = match daemon.request(Uuid::nil(), Uuid::nil(), job.command) {
                        Ok(pengpilot_client::ResponsePayload::TaskStateSaved { .. }) => Ok(()),
                        Ok(_) => Err(INVALID_SAVE_RESPONSE.into()),
                        Err(error) => Err(error.to_string()),
                    };
                    drop(_rpc);
                    acks.lock().push(SaveAck {
                        seq: job.seq,
                        dirty_generation: job.dirty_generation,
                        result,
                    });
                }
            })?;
        Ok(())
    }

    pub fn remove_session(&self, session_id: Uuid) -> io::Result<()> {
        self.daemon
            .client()
            .notify(session_id, Uuid::nil(), pengpilot_client::Command::RemoveSession)
            .map_err(to_io_error)
    }
}

const INVALID_SAVE_RESPONSE: &str = "PengPilot daemon returned an invalid task-state save response";

fn save_command(state: &PersistedState) -> pengpilot_client::Command {
    let dirty_ids = state.dirty_session_ids();
    let sessions = state
        .sessions
        .iter()
        .filter(|session| dirty_ids.contains(&session.id))
        .cloned()
        .collect();
    let live_session_ids = state.sessions.iter().map(|session| session.id).collect();
    pengpilot_client::Command::SaveTaskState {
        projects: state.projects.clone(),
        live_session_ids,
        sessions,
    }
}

fn to_io_error(error: anyhow::Error) -> io::Error {
    io::Error::other(error)
}
