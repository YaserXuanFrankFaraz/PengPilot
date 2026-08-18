//! Provider backend for `pengpilot-daemon`.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use anyhow::{Context as _, anyhow, bail};
use parking_lot::Mutex;
use pengpilot_protocol::{
    Command, Request, ResponsePayload, WireDriverEvent, decode_enum, event_to_wire,
};
use serde_json::Value;
use uuid::Uuid;

use crate::driver::{self, DriverHandle, DriverStartOptions, SessionOptions};
use crate::model::{AgentSession, Project};
use crate::persistence::{PersistedState, StateStore};
use crate::server::{Backend, EventSink};

pub struct PengPilotBackend {
    task_store: StateStore,
    task_state: Mutex<PersistedState>,
    removed_session_ids: Mutex<HashSet<Uuid>>,
    sessions: Mutex<HashMap<Uuid, (Uuid, DriverHandle)>>,
    default_cwd: std::path::PathBuf,
}

impl PengPilotBackend {
    pub fn new(task_store: StateStore) -> anyhow::Result<Self> {
        let task_state = task_store
            .load()
            .context("could not load PengPilot task database")?;
        Ok(Self {
            task_store,
            task_state: Mutex::new(task_state),
            removed_session_ids: Mutex::new(HashSet::new()),
            sessions: Mutex::new(HashMap::new()),
            default_cwd: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
        })
    }
}

impl Backend for PengPilotBackend {
    fn handle(&self, request: Request, events: EventSink) -> anyhow::Result<ResponsePayload> {
        let session_id = request.session_id;
        let runtime_id = request.runtime_id;
        match request.command {
            Command::LoadTaskState => {
                let state = self.task_state.lock();
                Ok(ResponsePayload::TaskState {
                    projects: state.projects.clone(),
                    sessions: state
                        .sessions
                        .iter()
                        .map(AgentSession::list_projection)
                        .collect(),
                    default_cwd: self.default_cwd.clone(),
                    projectless_root: pengpilot_protocol::projectless::workspace_root()
                        .map(std::path::Path::to_path_buf),
                })
            }
            Command::SaveTaskState {
                projects,
                live_session_ids: _,
                sessions,
            } => {
                // ponytail: last-write upsert. Stale-projection merge when a
                // second client shares this daemon.
                let mut state = self.task_state.lock();
                let removed_session_ids = self.removed_session_ids.lock();
                for project in projects {
                    if let Some(existing) = state
                        .projects
                        .iter_mut()
                        .find(|existing| existing.id == project.id)
                    {
                        *existing = project;
                    } else {
                        state.projects.push(project);
                    }
                }
                let sessions = sessions
                    .into_iter()
                    .filter(|session| !removed_session_ids.contains(&session.id))
                    .collect::<Vec<_>>();
                drop(removed_session_ids);
                let saved_ids = sessions
                    .iter()
                    .map(|session| session.id)
                    .collect::<Vec<_>>();
                for session in sessions {
                    if let Some(existing) = state
                        .sessions
                        .iter_mut()
                        .find(|existing| existing.id == session.id)
                    {
                        *existing = session;
                    } else {
                        state.sessions.push(session);
                    }
                }
                for session_id in &saved_ids {
                    state.mark_session_dirty(*session_id);
                }
                self.task_store.save(&mut state)?;
                let sessions = saved_ids
                    .into_iter()
                    .filter_map(|session_id| {
                        state
                            .sessions
                            .iter()
                            .find(|session| session.id == session_id)
                            .cloned()
                    })
                    .collect();
                Ok(ResponsePayload::TaskStateSaved { sessions })
            }
            Command::RemoveSession => {
                {
                    let mut state = self.task_state.lock();
                    self.removed_session_ids.lock().insert(session_id);
                    let project_id = state
                        .sessions
                        .iter()
                        .find(|session| session.id == session_id)
                        .map(|session| session.project_id);
                    state.sessions.retain(|session| session.id != session_id);
                    if let Some(project_id) = project_id {
                        let remove_project = state
                            .projects
                            .iter()
                            .find(|project| project.id == project_id)
                            .is_some_and(Project::is_projectless)
                            && !state
                                .sessions
                                .iter()
                                .any(|session| session.project_id == project_id);
                        if remove_project {
                            state.projects.retain(|project| project.id != project_id);
                        }
                    }
                    self.task_store.save(&mut state)?;
                }
                Ok(ResponsePayload::Ack)
            }
            Command::HydrateSession { session_id } => {
                let mut state = self.task_state.lock();
                let session = if let Some(session) = state
                    .sessions
                    .iter_mut()
                    .find(|session| session.id == session_id)
                {
                    self.task_store.hydrate(session)?;
                    Some(session.clone())
                } else {
                    None
                };
                Ok(ResponsePayload::Session { session })
            }
            Command::ProbeProvider {
                provider,
                binary_override,
                discover_models,
                probe_version,
            } => {
                ensure_shell_environment();
                let mut probe = if discover_models || probe_version {
                    crate::model::provider_probe(provider, binary_override.as_deref())
                } else {
                    crate::model::cached_provider_probe(provider, binary_override.as_deref())
                };
                let version = probe_version
                    .then(|| {
                        probe
                            .path
                            .as_deref()
                            .and_then(crate::model::probe_provider_version)
                    })
                    .flatten();
                if discover_models {
                    probe = crate::model::discover_provider_models(probe);
                }
                Ok(ResponsePayload::ProviderProbe { probe, version })
            }
            Command::FetchPlanUsage {
                provider,
                binary_override,
                cli_version,
            } => {
                let usage = match provider {
                    crate::model::ProviderKind::Claude => Some(
                        crate::usage::fetch_claude_plan_usage(cli_version.as_deref())?,
                    ),
                    crate::model::ProviderKind::Codex => {
                        Some(crate::usage::fetch_codex_plan_usage()?)
                    }
                    crate::model::ProviderKind::OpenCode => {
                        crate::usage::fetch_opencode_go_plan_usage()?
                    }
                    crate::model::ProviderKind::Grok => {
                        ensure_shell_environment();
                        let probe =
                            crate::model::provider_probe(provider, binary_override.as_deref());
                        let binary = probe
                            .path
                            .ok_or_else(|| anyhow!("grok is not installed"))?;
                        Some(crate::usage::fetch_grok_plan_usage(&binary)?)
                    }
                    _ => bail!("provider has no plan usage fetcher"),
                };
                Ok(ResponsePayload::PlanUsage { usage })
            }
            Command::ProbeComputerPermissions { prompt } => {
                let permissions = crate::computer_use::probe_permissions(prompt)?;
                Ok(ResponsePayload::ComputerPermissions { permissions })
            }
            Command::LoadSkills { projects } => {
                let locations = crate::skills::skill_locations(&projects);
                Ok(ResponsePayload::SkillsCatalog {
                    catalog: crate::skills::scan_skills(&locations),
                })
            }
            Command::SetSkillsEnabled { dirs, enabled } => {
                for dir in dirs {
                    crate::skills::set_skill_enabled(&dir, enabled)
                        .map_err(|error| anyhow!(error))?;
                }
                Ok(ResponsePayload::Ack)
            }
            Command::TrashSkills { dirs } => {
                crate::skills::trash_skills(&dirs).map_err(|error| anyhow!(error))?;
                Ok(ResponsePayload::Ack)
            }
            Command::Start { options } => {
                let previous = self.sessions.lock().remove(&session_id);
                drop(previous);
                let provider = decode_enum(&options.provider)?;
                let options = DriverStartOptions {
                    binary: options.binary,
                    cwd: options.cwd,
                    mode: decode_enum(&options.mode)?,
                    interaction_mode: decode_enum(&options.interaction_mode)?,
                    model: options.model,
                    reasoning_effort: options.reasoning_effort,
                    service_tier: options.service_tier,
                    // ponytail: wire has context_window; local DriverStartOptions does not.
                    agent_preset: options.agent_preset,
                    computer_use_enabled: options.computer_use_enabled,
                    provider_cursor: options
                        .provider_cursor
                        .map(serde_json::from_value)
                        .transpose()
                        .context("daemon received an invalid provider cursor")?,
                };
                let (wake, _wake_events) = smol::channel::bounded(1);
                let (event_sender, event_receiver) = driver::event_channel(wake);
                let handle = driver::start(provider, options, event_sender)?;
                let supports_steer = handle.supports_steer();
                std::thread::Builder::new()
                    .name(format!("pengpilot-daemon-events-{session_id}"))
                    .spawn(move || {
                        while let Ok(event) = event_receiver.recv() {
                            let wire = event_to_wire(event).unwrap_or_else(|error| {
                                WireDriverEvent::new(
                                    "error",
                                    Value::String(format!(
                                        "could not encode daemon event: {error}"
                                    )),
                                )
                            });
                            if events.send(wire).is_err() {
                                break;
                            }
                        }
                    })
                    .context("could not start daemon event forwarding thread")?;
                self.sessions
                    .lock()
                    .insert(session_id, (runtime_id, handle));
                Ok(ResponsePayload::Started { supports_steer })
            }
            Command::CloseSession => {
                let removed = {
                    let mut sessions = self.sessions.lock();
                    sessions
                        .get(&session_id)
                        .is_some_and(|(active_runtime_id, _)| *active_runtime_id == runtime_id)
                        .then(|| sessions.remove(&session_id))
                        .flatten()
                };
                drop(removed);
                Ok(ResponsePayload::Ack)
            }
            command @ (Command::Prompt { .. }
            | Command::Steer { .. }
            | Command::Cancel
            | Command::CancelComputerUse
            | Command::RefreshBackgroundWork
            | Command::StopBackgroundWork { .. }
            | Command::Respond { .. }
            | Command::RespondUserInput { .. }
            | Command::RunComputerTool { .. }
            | Command::RejectComputerTool { .. }
            | Command::ApplyOptions { .. }
            | Command::Rollback { .. }
            | Command::Fork { .. }) => {
                let driver = {
                    let sessions = self.sessions.lock();
                    let (active_runtime_id, driver) = sessions
                        .get(&session_id)
                        .ok_or_else(|| anyhow!("daemon session {session_id} is not running"))?;
                    if *active_runtime_id != runtime_id {
                        bail!(
                            "daemon session {session_id} belongs to runtime {active_runtime_id}, not {runtime_id}"
                        );
                    }
                    driver.clone()
                };
                handle_driver_command(&driver, command)
            }
            Command::AttachSession
            | Command::ForkSessionFromResponse { .. }
            | Command::RewindSessionToMessage { .. }
            | Command::OpenTerminal { .. }
            | Command::WriteTerminal { .. }
            | Command::ResizeTerminal { .. }
            | Command::CloseTerminal => {
                bail!("daemon command is not implemented yet")
            }
        }
    }

    fn shutdown(&self) {
        let sessions = std::mem::take(&mut *self.sessions.lock());
        drop(sessions);
    }
}

fn handle_driver_command(
    driver: &DriverHandle,
    command: Command,
) -> anyhow::Result<ResponsePayload> {
    match command {
        Command::Prompt { prompt } => driver.prompt(prompt),
        Command::Steer { prompt } => driver.steer(prompt),
        Command::Cancel => driver.cancel(),
        Command::CancelComputerUse => driver.cancel_computer_use(),
        Command::RefreshBackgroundWork => driver.refresh_background_work(),
        Command::StopBackgroundWork { key, control_id } => {
            driver.stop_background_work(
                serde_json::from_value(key).context("invalid background-work key")?,
                control_id,
            );
        }
        Command::Respond {
            request_id,
            option_id,
        } => driver.respond(request_id, option_id),
        Command::RespondUserInput {
            request_id,
            answers,
        } => driver.respond_user_input(request_id, answers),
        Command::RunComputerTool { request } => {
            driver.run_computer_tool(crate::computer_use::ComputerToolRequest {
                call_id: request.call_id,
                tool: request.tool,
                arguments: request.arguments,
            });
        }
        Command::RejectComputerTool { request, reason } => {
            driver.reject_computer_tool(
                crate::computer_use::ComputerToolRequest {
                    call_id: request.call_id,
                    tool: request.tool,
                    arguments: request.arguments,
                },
                reason,
            );
        }
        Command::ApplyOptions { options } => {
            return Ok(ResponsePayload::OptionsApplied {
                applied: driver.apply_options(SessionOptions {
                    mode: decode_enum(&options.mode)?,
                    interaction_mode: decode_enum(&options.interaction_mode)?,
                    model: options.model,
                    reasoning_effort: options.reasoning_effort,
                    service_tier: options.service_tier,
                }),
            });
        }
        Command::Rollback { turns } => {
            let cursor = driver
                .rollback(turns)?
                .map(serde_json::to_value)
                .transpose()?;
            return Ok(ResponsePayload::Cursor { cursor });
        }
        Command::Fork { turns_to_remove } => {
            let cursor = Some(serde_json::to_value(driver.fork(turns_to_remove)?)?);
            return Ok(ResponsePayload::Cursor { cursor });
        }
        _ => bail!("daemon received a command in the wrong dispatch path"),
    }
    Ok(ResponsePayload::Ack)
}

fn ensure_shell_environment() {
    static REFRESHED: OnceLock<()> = OnceLock::new();
    REFRESHED.get_or_init(|| {
        crate::command_env::refresh_from_default_shell();
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use pengpilot_protocol::model::ProviderKind;
    use pengpilot_protocol::protocol::Command;
    use uuid::Uuid;

    fn backend_in(directory: &std::path::Path) -> PengPilotBackend {
        PengPilotBackend::new(StateStore::new(directory.join("app.db"))).unwrap()
    }

    fn request(command: Command) -> Request {
        Request {
            request_id: Uuid::new_v4(),
            session_id: Uuid::nil(),
            runtime_id: Uuid::nil(),
            command,
        }
    }

    #[test]
    fn load_task_state_returns_empty_projections_for_a_fresh_store() {
        let directory = std::env::temp_dir().join(format!("pengpilot-backend-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let backend = backend_in(&directory);
        let ResponsePayload::TaskState {
            projects, sessions, ..
        } = backend
            .handle(request(Command::LoadTaskState), EventSink::discarded())
            .unwrap()
        else {
            panic!("expected task state");
        };
        assert!(projects.is_empty());
        assert!(sessions.is_empty());
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn probe_provider_reports_install_state_without_live_discovery() {
        let directory =
            std::env::temp_dir().join(format!("pengpilot-backend-probe-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let backend = backend_in(&directory);
        let ResponsePayload::ProviderProbe { probe, version } = backend
            .handle(
                request(Command::ProbeProvider {
                    provider: ProviderKind::Codex,
                    binary_override: None,
                    discover_models: false,
                    probe_version: false,
                }),
                EventSink::discarded(),
            )
            .unwrap()
        else {
            panic!("expected provider probe");
        };
        assert_eq!(probe.provider, ProviderKind::Codex);
        assert!(version.is_none());
        let _ = std::fs::remove_dir_all(directory);
    }

    fn started_session(project_id: Uuid) -> AgentSession {
        use pengpilot_protocol::session::{MessageRole, TurnStatus};
        let mut session = AgentSession::new(project_id, ProviderKind::Codex);
        session.begin_turn("Ask");
        session.push_message(MessageRole::Assistant, "an answer");
        session.finish_active_turn(TurnStatus::Completed);
        session
    }

    #[test]
    fn save_then_reload_hydrates_the_transcript() {
        let directory =
            std::env::temp_dir().join(format!("pengpilot-backend-save-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let project = Project::from_path(directory.join("project"));
        let session = started_session(project.id);
        let session_id = session.id;
        let backend = backend_in(&directory);
        let ResponsePayload::TaskStateSaved { sessions } = backend
            .handle(
                request(Command::SaveTaskState {
                    projects: vec![project.clone()],
                    live_session_ids: vec![session_id],
                    sessions: vec![session],
                }),
                EventSink::discarded(),
            )
            .unwrap()
        else {
            panic!("expected save");
        };
        assert_eq!(sessions.len(), 1);

        let backend = backend_in(&directory);
        let ResponsePayload::TaskState { sessions, .. } = backend
            .handle(request(Command::LoadTaskState), EventSink::discarded())
            .unwrap()
        else {
            panic!("expected task state");
        };
        assert_eq!(sessions.len(), 1);
        assert!(!sessions[0].detail_loaded);

        let ResponsePayload::Session {
            session: Some(hydrated),
        } = backend
            .handle(
                request(Command::HydrateSession { session_id }),
                EventSink::discarded(),
            )
            .unwrap()
        else {
            panic!("expected hydrated session");
        };
        assert!(hydrated.detail_loaded);
        assert_eq!(hydrated.messages.len(), 2);
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn remove_session_drops_the_row() {
        let directory =
            std::env::temp_dir().join(format!("pengpilot-backend-remove-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let project = Project::from_path(directory.join("project"));
        let session = started_session(project.id);
        let session_id = session.id;
        let backend = backend_in(&directory);
        backend
            .handle(
                request(Command::SaveTaskState {
                    projects: vec![project],
                    live_session_ids: vec![session_id],
                    sessions: vec![session],
                }),
                EventSink::discarded(),
            )
            .unwrap();
        backend
            .handle(
                Request {
                    request_id: Uuid::new_v4(),
                    session_id,
                    runtime_id: Uuid::nil(),
                    command: Command::RemoveSession,
                },
                EventSink::discarded(),
            )
            .unwrap();
        let ResponsePayload::TaskState { sessions, .. } = backend
            .handle(request(Command::LoadTaskState), EventSink::discarded())
            .unwrap()
        else {
            panic!("expected task state");
        };
        assert!(sessions.is_empty());
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn prompt_without_a_session_errors() {
        let directory =
            std::env::temp_dir().join(format!("pengpilot-backend-prompt-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let backend = backend_in(&directory);
        let error = backend
            .handle(
                request(Command::Prompt {
                    prompt: "hello".into(),
                }),
                EventSink::discarded(),
            )
            .unwrap_err();
        assert!(error.to_string().contains("is not running"));
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn fetch_plan_usage_rejects_providers_without_a_fetcher() {
        let directory =
            std::env::temp_dir().join(format!("pengpilot-backend-usage-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let backend = backend_in(&directory);
        let error = backend
            .handle(
                request(Command::FetchPlanUsage {
                    provider: ProviderKind::Amp,
                    binary_override: None,
                    cli_version: None,
                }),
                EventSink::discarded(),
            )
            .unwrap_err();
        assert!(error.to_string().contains("no plan usage fetcher"));
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn load_skills_returns_a_catalog() {
        let directory =
            std::env::temp_dir().join(format!("pengpilot-backend-skills-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let backend = backend_in(&directory);
        let ResponsePayload::SkillsCatalog { catalog } = backend
            .handle(
                request(Command::LoadSkills {
                    projects: Vec::new(),
                }),
                EventSink::discarded(),
            )
            .unwrap()
        else {
            panic!("expected skills catalog");
        };
        let _ = catalog.skills.len();
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn unimplemented_commands_error_instead_of_ack() {
        let directory =
            std::env::temp_dir().join(format!("pengpilot-backend-unimpl-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let backend = backend_in(&directory);
        let error = backend
            .handle(request(Command::AttachSession), EventSink::discarded())
            .unwrap_err();
        assert!(error.to_string().contains("not implemented"));
        let _ = std::fs::remove_dir_all(directory);
    }
}
