//! Desktop ownership of the PengPilot daemon process.

use std::path::PathBuf;

use anyhow::{anyhow, bail, Context as _};

pub fn start_process() -> anyhow::Result<pengpilot_client::DaemonSupervisor> {
    let address = std::env::var(pengpilot_client::DAEMON_ADDRESS_ENV)
        .ok()
        .filter(|value| !value.trim().is_empty());
    let token = std::env::var(pengpilot_client::DAEMON_TOKEN_ENV)
        .ok()
        .filter(|value| !value.is_empty());
    match (address, token) {
        (Some(address), Some(token)) => {
            return pengpilot_client::DaemonSupervisor::connect(address.trim(), token);
        }
        (Some(_), None) => bail!(
            "{} is set but {} is missing",
            pengpilot_client::DAEMON_ADDRESS_ENV,
            pengpilot_client::DAEMON_TOKEN_ENV
        ),
        (None, Some(_)) => bail!(
            "{} is set but {} is missing",
            pengpilot_client::DAEMON_TOKEN_ENV,
            pengpilot_client::DAEMON_ADDRESS_ENV
        ),
        (None, None) => {}
    }
    pengpilot_client::DaemonSupervisor::spawn_configured(
        &daemon_executable_path()?,
        cfg!(debug_assertions),
        pengpilot_client::DaemonExposureSettings::default(),
    )
}

fn daemon_executable_path() -> anyhow::Result<PathBuf> {
    if let Some(path) = std::env::var_os("PENGPILOT_DAEMON_PATH").filter(|path| !path.is_empty()) {
        return Ok(path.into());
    }
    let executable = format!("pengpilot-daemon{}", std::env::consts::EXE_SUFFIX);
    let current = std::env::current_exe().context("could not locate the PengPilot executable")?;

    // Development keeps the daemon beside Cargo's debug artifacts rather than
    // inside PengPilot Debug.app. The supervisor watches this file and swaps
    // only the daemon when the development watcher relinks it.
    #[cfg(debug_assertions)]
    if let Some(debug_directory) = current
        .ancestors()
        .find(|candidate| candidate.file_name().is_some_and(|name| name == "debug"))
    {
        let external = debug_directory.join(&executable);
        if external.is_file() {
            return Ok(external);
        }
    }

    let sibling = current
        .parent()
        .map(|directory| directory.join(&executable))
        .ok_or_else(|| anyhow!("PengPilot executable has no parent directory"))?;
    if sibling.is_file() {
        return Ok(sibling);
    }
    #[cfg(debug_assertions)]
    bail!(
        "PengPilot daemon was not found in Cargo's debug directory or next to the app executable: {}",
        sibling.display(),
    );
    #[cfg(not(debug_assertions))]
    bail!(
        "PengPilot daemon is missing next to the app executable: {}",
        sibling.display(),
    )
}
