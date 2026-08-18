//! Shared application identity used by the daemon and desktop client.

#[cfg(debug_assertions)]
pub const APP_NAME: &str = "PengPilot Debug";
#[cfg(not(debug_assertions))]
pub const APP_NAME: &str = "PengPilot";

#[cfg(debug_assertions)]
pub const APP_ID: &str = "com.pengpilot.app.debug";
#[cfg(not(debug_assertions))]
pub const APP_ID: &str = "com.pengpilot.app";

#[cfg(debug_assertions)]
pub const DATA_DIRECTORY_NAME: &str = "PengPilot Debug";
#[cfg(not(debug_assertions))]
pub const DATA_DIRECTORY_NAME: &str = "PengPilot";
