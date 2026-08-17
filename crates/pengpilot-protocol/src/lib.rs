//! Transport-neutral wire contract shared by the PengPilot desktop app and its
//! daemon. Serializable value types only — no I/O. Mirrors waku's
//! `waku-protocol`: a future JSON-RPC-over-WebSocket protocol serializes
//! these types, and both sides depend on this crate.

pub mod model;
