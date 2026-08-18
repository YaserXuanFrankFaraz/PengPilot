//! Transport-neutral wire contract shared by the PengPilot desktop app and its
//! daemon. Serializable value types only — no I/O. Mirrors waku's
//! `waku-protocol`: a future JSON-RPC-over-WebSocket protocol serializes
//! these types, and both sides depend on this crate.

rust_i18n::i18n!("../../locales", fallback = "en");

/// Same surface as the app's `tr!`; the protocol crate's value types use it for
/// display labels. The runtime locale is process-global (set by the app via
/// `rust_i18n::set_locale`), so both sides read the same language.
macro_rules! tr {
    ($key:expr) => {
        rust_i18n::t!($key).into_owned()
    };
    ($key:expr, $($args:tt)*) => {
        rust_i18n::t!($key, $($args)*).into_owned()
    };
}

pub mod agent;
pub mod computer_use;
mod driver_wire;
pub mod i18n;
pub mod identity;
pub mod model;
pub mod projectless;
pub mod protocol;
pub mod session;
pub mod theme;
pub mod usage;
pub mod work;

pub use driver_wire::{decode_enum, encode_enum, event_from_wire, event_to_wire};
pub use protocol::{
    APP_EXECUTABLE_ENV, ClientMessage, Command, DAEMON_ADDRESS_ENV, DAEMON_TOKEN_ENV, DaemonReady,
    MAX_WIRE_MESSAGE_BYTES, PROTOCOL_VERSION, ReplayCursor, Request, ResponseOutcome,
    ResponsePayload, RpcError, SequencedEvent, ServerMessage, WireComputerToolRequest,
    WireDriverEvent, WireDriverStartOptions, WireSessionOptions,
};
