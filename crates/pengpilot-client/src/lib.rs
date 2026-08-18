//! Transport for clients of `pengpilot-daemon`.
//!
//! This crate depends only on [`pengpilot_protocol`], so GUI clients cannot
//! reach daemon-owned filesystem, Git, database, or provider implementations.

mod client;

pub use client::DaemonClient;
pub use pengpilot_protocol::*;
