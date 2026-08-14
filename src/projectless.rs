//! Private workspaces for tasks that are not attached to a user project.
//!
//! Codex allocates ordinary projectless chats beneath a per-user root using
//! `<root>/<local date>/<prompt slug>`, with numeric collision suffixes and a
//! random fallback. PengPilot mirrors that layout beneath `~/.pengpilot`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use chrono::{Local, NaiveDate};
use uuid::Uuid;

const DEFAULT_SLUG: &str = "new-chat";
const MAX_SLUG_BYTES: usize = 80;
const MAX_NUMBERED_CANDIDATES: usize = 100;
const MAX_RANDOM_CANDIDATES: usize = 5;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Workspace {
    pub cwd: PathBuf,
    pub workspace_root: PathBuf,
}

/// The root is cached because `Project::is_projectless` is reached from row
/// builders and render paths. Those callers must only perform path compares.
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

pub fn create_workspace(prompt: Option<&str>) -> io::Result<Workspace> {
    let root = workspace_root().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "could not locate the home directory for ~/.pengpilot",
        )
    })?;
    create_workspace_in(root, Local::now().date_naive(), None, prompt)
}

fn workspace_slug(directory_name: Option<&str>, prompt: Option<&str>) -> String {
    let directory_name = directory_name.filter(|value| !value.trim().is_empty());
    let prompt = prompt.filter(|value| !value.trim().is_empty());
    let source = directory_name.or(prompt).unwrap_or_default().to_lowercase();
    let mut words = source
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty());
    let maximum_words = if directory_name.is_some() {
        usize::MAX
    } else {
        6
    };
    let mut slug = words
        .by_ref()
        .take(maximum_words)
        .collect::<Vec<_>>()
        .join("-");
    slug.truncate(MAX_SLUG_BYTES);
    if slug.is_empty() {
        DEFAULT_SLUG.to_owned()
    } else {
        slug
    }
}

fn create_workspace_in(
    root: &Path,
    date: NaiveDate,
    directory_name: Option<&str>,
    prompt: Option<&str>,
) -> io::Result<Workspace> {
    ensure_real_directory(root)?;
    let date_directory = root.join(date.format("%Y-%m-%d").to_string());
    ensure_real_directory(&date_directory)?;
    let slug = workspace_slug(directory_name, prompt);

    for index in 0..MAX_NUMBERED_CANDIDATES {
        let name = if index == 0 {
            slug.clone()
        } else {
            format!("{slug}-{}", index + 1)
        };
        if let Some(workspace) = try_create_workspace(root, &date_directory, &name)? {
            return Ok(workspace);
        }
    }

    for _ in 0..MAX_RANDOM_CANDIDATES {
        let name = format!("{slug}-{}", Uuid::new_v4());
        if let Some(workspace) = try_create_workspace(root, &date_directory, &name)? {
            return Ok(workspace);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "unable to create a unique projectless task directory",
    ))
}

fn try_create_workspace(
    root: &Path,
    date_directory: &Path,
    name: &str,
) -> io::Result<Option<Workspace>> {
    let cwd = date_directory.join(name);
    match fs::create_dir(&cwd) {
        Ok(()) => Ok(Some(Workspace {
            cwd,
            workspace_root: root.to_owned(),
        })),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            validate_real_directory(&cwd)?;
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

fn ensure_real_directory(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(_) => validate_real_directory(path),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            fs::create_dir_all(path)?;
            validate_real_directory(path)
        }
        Err(error) => Err(error),
    }
}

fn validate_real_directory(path: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "projectless task directory must be a real directory",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_root() -> PathBuf {
        std::env::temp_dir().join(format!("waku-projectless-{}", Uuid::new_v4()))
    }

    #[test]
    fn prompt_slug_matches_codex_word_and_length_rules() {
        assert_eq!(
            workspace_slug(
                None,
                Some("Build A polished, LOCAL coding agent please now")
            ),
            "build-a-polished-local-coding-agent"
        );
        assert_eq!(workspace_slug(None, Some("你好 👋")), "new-chat");
        assert_eq!(
            workspace_slug(Some("Release Candidate Number 12"), Some("ignored prompt")),
            "release-candidate-number-12"
        );
        assert_eq!(
            workspace_slug(Some("  "), Some("Prompt fallback")),
            "prompt-fallback"
        );
        assert_eq!(
            workspace_slug(Some(&"a".repeat(100)), None).len(),
            MAX_SLUG_BYTES
        );
    }

    #[test]
    fn creates_date_and_unique_task_directories_without_split_folders() {
        let root = test_root();
        let date = NaiveDate::from_ymd_opt(2026, 8, 8).unwrap();

        let first = create_workspace_in(&root, date, None, Some("Fix projectless sessions"))
            .expect("first workspace");
        let second = create_workspace_in(&root, date, None, Some("Fix projectless sessions"))
            .expect("second workspace");

        assert_eq!(first.cwd, root.join("2026-08-08/fix-projectless-sessions"));
        assert_eq!(
            second.cwd,
            root.join("2026-08-08/fix-projectless-sessions-2")
        );
        assert_eq!(first.workspace_root, root);
        assert_eq!(fs::read_dir(&first.cwd).unwrap().count(), 0);

        fs::remove_dir_all(&root).ok();
    }

    #[cfg(unix)]
    #[test]
    fn refuses_a_symlinked_workspace_root() {
        use std::os::unix::fs::symlink;

        let parent = test_root();
        let target = parent.join("target");
        let root = parent.join("root");
        fs::create_dir_all(&target).unwrap();
        symlink(&target, &root).unwrap();

        let error = create_workspace_in(
            &root,
            NaiveDate::from_ymd_opt(2026, 8, 8).unwrap(),
            None,
            None,
        )
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);

        fs::remove_dir_all(&parent).ok();
    }
}
