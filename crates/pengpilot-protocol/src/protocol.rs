use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::agent::AgentSession;
use crate::computer_use::ComputerPermissions;
use crate::model::{ProviderKind, ProviderProbe};
use crate::session::Project;
use crate::usage::PlanUsage;

pub const PROTOCOL_VERSION: u32 = 3;
pub const DAEMON_TOKEN_ENV: &str = "PENGPILOT_DAEMON_TOKEN";
pub const DAEMON_ADDRESS_ENV: &str = "PENGPILOT_DAEMON_ADDRESS";
pub const APP_EXECUTABLE_ENV: &str = "PENGPILOT_APP_EXECUTABLE";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DaemonReady {
    pub address: String,
    pub protocol_version: u32,
    pub pid: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub request_id: Uuid,
    pub session_id: Uuid,
    pub runtime_id: Uuid,
    pub command: Command,
}

/// In-process command surface. Phase 3 adds the remaining JSON-RPC variants
/// from waku's `protocol.rs` when WebSocket serve lands.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Command {
    LoadTaskState,
    ProbeProvider {
        provider: ProviderKind,
        binary_override: Option<String>,
        discover_models: bool,
        probe_version: bool,
    },
    FetchPlanUsage {
        provider: ProviderKind,
        binary_override: Option<String>,
        cli_version: Option<String>,
    },
    ProbeComputerPermissions {
        prompt: bool,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ResponsePayload {
    Ack,
    ProviderProbe {
        probe: ProviderProbe,
        version: Option<String>,
    },
    PlanUsage {
        usage: Option<PlanUsage>,
    },
    ComputerPermissions {
        permissions: ComputerPermissions,
    },
    TaskState {
        projects: Vec<Project>,
        sessions: Vec<AgentSession>,
        default_cwd: PathBuf,
        projectless_root: Option<PathBuf>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_version_is_waku_aligned() {
        assert_eq!(PROTOCOL_VERSION, 3);
    }

    #[test]
    fn daemon_ready_serializes_camel_case() {
        let json = serde_json::to_value(DaemonReady {
            address: "inproc".into(),
            protocol_version: PROTOCOL_VERSION,
            pid: 1,
        })
        .unwrap();
        assert_eq!(json["protocolVersion"], PROTOCOL_VERSION);
        assert_eq!(json["address"], "inproc");
    }
}
