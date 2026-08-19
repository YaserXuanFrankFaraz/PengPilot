//! Transport for clients of `pengpilot-daemon`.
//!
//! This crate depends only on [`pengpilot_protocol`], so GUI clients cannot
//! reach daemon-owned filesystem, Git, database, or provider implementations.

mod client;
mod process;
mod workspace_client;

pub use client::DaemonClient;
pub use pengpilot_protocol::*;
pub use process::{
    DEFAULT_EXPOSED_DAEMON_PORT, DaemonExposureSettings, DaemonProcess, DaemonSupervisor,
    parse_allowed_origins,
};
pub use workspace_client::WorkspaceClient;
