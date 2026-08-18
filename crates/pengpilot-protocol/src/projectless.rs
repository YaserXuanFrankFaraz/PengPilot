//! Path predicates for the per-user `~/.pengpilot` workspace root.
//!
//! These are env-only reads plus path compares. Creating or migrating
//! directories stays in the app.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// The root is cached because `Project::is_projectless` is reached from row
/// builders and render paths. Those callers must only perform path compares.
pub fn home_directory() -> Option<PathBuf> {
    let mut home = dirs::home_dir()?;
    home.push(".pengpilot");
    Some(home)
}

pub fn workspace_root() -> Option<&'static Path> {
    static ROOT: OnceLock<Option<PathBuf>> = OnceLock::new();
    ROOT.get_or_init(|| dirs::home_dir().map(|home| home.join(".pengpilot")))
        .as_deref()
}

/// Includes the root itself so the short-lived root-level implementation can
/// be recognized and migrated, although new workspaces are always descendants.
pub fn is_projectless_path(path: &Path) -> bool {
    workspace_root().is_some_and(|root| path.starts_with(root))
}

pub fn is_legacy_root_path(path: &Path) -> bool {
    workspace_root().is_some_and(|root| path == root)
}
