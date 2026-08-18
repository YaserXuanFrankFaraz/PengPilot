//! PengPilot's daemon-side core.
//!
//! Provider, database, filesystem, and Git implementations live here, behind
//! the transport-neutral contract in `pengpilot-protocol`. The desktop app
//! re-exports these modules so existing `crate::git_commit` paths keep working
//! while the engine moves out of the UI process.
//!
//! Phase 2 in-process backend: no WebSocket yet. `pengpilot-daemon` and RPC
//! land once drivers follow these modules.

pub mod blob_store;
pub mod checkpoint;
pub mod command_env;
pub mod git_branch;
pub mod git_commit;
pub mod library;
pub mod persistence;
pub mod worktree;
