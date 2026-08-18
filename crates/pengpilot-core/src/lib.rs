//! PengPilot's daemon-side core.
//!
//! Provider, database, filesystem, and Git implementations live here, behind
//! the transport-neutral contract in `pengpilot-protocol`. The desktop app
//! re-exports these modules so existing `crate::git_commit` paths keep working
//! while the engine moves out of the UI process.
//!
//! Phase 2 in-process backend: no WebSocket yet. `pengpilot-daemon` and RPC
//! land once the remaining UI-adjacent modules follow.

#![recursion_limit = "256"]

rust_i18n::i18n!("../../locales", fallback = "en");

macro_rules! tr {
    ($key:expr) => {
        crate::i18n::translate($key)
    };
    ($key:expr, $($args:tt)*) => {
        rust_i18n::t!($key, $($args)*).into_owned()
    };
}

pub mod amp_session;
pub mod blob_store;
pub mod checkpoint;
pub mod claude_session;
pub mod command_env;
pub mod computer_use;
pub mod cursor_session;
pub mod deepseek_pool;
pub mod deepseek_session;
pub mod driver;
pub mod git_branch;
pub mod git_commit;
pub mod grok_session;
pub mod i18n {
    pub use pengpilot_protocol::i18n::*;
}
pub mod library;
pub mod model;
pub mod model_catalog;
pub mod opencode_pool;
pub mod opencode_session;
pub mod persistence;
pub mod usage;
pub mod worktree;
