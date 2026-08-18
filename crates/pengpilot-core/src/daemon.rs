//! Provider backend for `pengpilot-daemon`.

use std::sync::OnceLock;

use anyhow::Context as _;
use parking_lot::Mutex;
use pengpilot_protocol::{Command, Request, ResponsePayload};

use crate::model::AgentSession;
use crate::persistence::{PersistedState, StateStore};
use crate::server::{Backend, EventSink};

pub struct PengPilotBackend {
    task_state: Mutex<PersistedState>,
    default_cwd: std::path::PathBuf,
}

impl PengPilotBackend {
    pub fn new(task_store: StateStore) -> anyhow::Result<Self> {
        let task_state = task_store
            .load()
            .context("could not load PengPilot task database")?;
        Ok(Self {
            task_state: Mutex::new(task_state),
            default_cwd: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
        })
    }
}

impl Backend for PengPilotBackend {
    fn handle(&self, request: Request, _events: EventSink) -> anyhow::Result<ResponsePayload> {
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
                        match probe.path.as_deref() {
                            Some(binary) => Some(crate::usage::fetch_grok_plan_usage(binary)?),
                            None => None,
                        }
                    }
                    _ => None,
                };
                Ok(ResponsePayload::PlanUsage { usage })
            }
            Command::ProbeComputerPermissions { prompt } => {
                let permissions = crate::computer_use::probe_permissions(prompt)?;
                Ok(ResponsePayload::ComputerPermissions { permissions })
            }
            _ => Ok(ResponsePayload::Ack),
        }
    }
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
}
