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

pub mod model;
pub mod session;
pub mod work;
