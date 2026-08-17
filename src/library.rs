//! User-curated media library: copy satisfied Imagine outputs into owned storage.
//!
//! Grok session images live under `~/.grok/sessions/…/images/` and can vanish
//! with the session. Saving copies bytes into the app data `library/` folder
//! and indexes them in SQLite so the sidebar Library page can browse them.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LibraryAsset {
    pub id: String,
    pub filename: String,
    pub prompt: Option<String>,
    pub source_path: Option<String>,
    pub session_id: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub created_at: u64,
}

#[derive(Clone, Debug)]
pub struct SaveLibraryAsset {
    pub source_path: PathBuf,
    pub prompt: Option<String>,
    pub session_id: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
}

impl LibraryAsset {
    pub fn path_in(&self, root: &Path) -> PathBuf {
        root.join(&self.filename)
    }
}

pub fn library_root_for_db(db_path: &Path) -> PathBuf {
    db_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("library")
}

pub fn list_assets(connection: &Connection) -> io::Result<Vec<LibraryAsset>> {
    let mut statement = connection
        .prepare(
            "SELECT id, filename, prompt, source_path, session_id, provider, model, created_at
             FROM library_assets
             ORDER BY created_at DESC, id DESC",
        )
        .map_err(to_io)?;
    let rows = statement
        .query_map([], |row| {
            Ok(LibraryAsset {
                id: row.get(0)?,
                filename: row.get(1)?,
                prompt: row.get(2)?,
                source_path: row.get(3)?,
                session_id: row.get(4)?,
                provider: row.get(5)?,
                model: row.get(6)?,
                created_at: row.get::<_, i64>(7)? as u64,
            })
        })
        .map_err(to_io)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(to_io)
}

pub fn find_by_source_path(
    connection: &Connection,
    source_path: &Path,
) -> io::Result<Option<LibraryAsset>> {
    let source = source_path.to_string_lossy();
    connection
        .query_row(
            "SELECT id, filename, prompt, source_path, session_id, provider, model, created_at
             FROM library_assets
             WHERE source_path = ?1
             ORDER BY created_at DESC
             LIMIT 1",
            params![source.as_ref()],
            |row| {
                Ok(LibraryAsset {
                    id: row.get(0)?,
                    filename: row.get(1)?,
                    prompt: row.get(2)?,
                    source_path: row.get(3)?,
                    session_id: row.get(4)?,
                    provider: row.get(5)?,
                    model: row.get(6)?,
                    created_at: row.get::<_, i64>(7)? as u64,
                })
            },
        )
        .optional()
        .map_err(to_io)
}

pub fn save_asset(
    connection: &Connection,
    root: &Path,
    request: SaveLibraryAsset,
) -> io::Result<LibraryAsset> {
    if !request.source_path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("source image missing: {}", request.source_path.display()),
        ));
    }
    if let Some(existing) = find_by_source_path(connection, &request.source_path)? {
        return Ok(existing);
    }

    fs::create_dir_all(root)?;
    let id = Uuid::new_v4().to_string();
    let extension = request
        .source_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("png");
    let filename = format!("{id}.{extension}");
    let destination = root.join(&filename);
    fs::copy(&request.source_path, &destination)?;

    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    connection
        .execute(
            "INSERT INTO library_assets
             (id, filename, prompt, source_path, session_id, provider, model, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                id,
                filename,
                request.prompt,
                request.source_path.to_string_lossy().as_ref(),
                request.session_id,
                request.provider,
                request.model,
                created_at as i64,
            ],
        )
        .map_err(to_io)?;

    Ok(LibraryAsset {
        id,
        filename,
        prompt: request.prompt,
        source_path: Some(request.source_path.to_string_lossy().into_owned()),
        session_id: request.session_id,
        provider: request.provider,
        model: request.model,
        created_at,
    })
}

pub fn delete_asset(connection: &Connection, root: &Path, id: &str) -> io::Result<()> {
    let filename: String = connection
        .query_row(
            "SELECT filename FROM library_assets WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(to_io)?;
    connection
        .execute("DELETE FROM library_assets WHERE id = ?1", params![id])
        .map_err(to_io)?;
    let path = root.join(filename);
    if path.is_file() {
        let _ = fs::remove_file(path);
    }
    Ok(())
}

fn to_io(error: impl std::fmt::Display) -> io::Error {
    io::Error::other(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::apply_migrations;

    fn open_db() -> (PathBuf, Connection, PathBuf) {
        let dir = std::env::temp_dir().join(format!("pengpilot-library-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("app.db");
        let connection = Connection::open(&db_path).unwrap();
        apply_migrations(&connection).unwrap();
        let root = library_root_for_db(&db_path);
        (dir, connection, root)
    }

    #[test]
    fn save_copies_file_and_lists_newest_first() {
        let (dir, connection, root) = open_db();
        let source = dir.join("source.jpg");
        fs::write(&source, b"jpeg-bytes").unwrap();

        let saved = save_asset(
            &connection,
            &root,
            SaveLibraryAsset {
                source_path: source.clone(),
                prompt: Some("a red bicycle".into()),
                session_id: None,
                provider: Some("grok".into()),
                model: None,
            },
        )
        .unwrap();
        assert!(saved.path_in(&root).is_file());
        assert_eq!(fs::read(saved.path_in(&root)).unwrap(), b"jpeg-bytes");

        let again = save_asset(
            &connection,
            &root,
            SaveLibraryAsset {
                source_path: source,
                prompt: Some("ignored".into()),
                session_id: None,
                provider: None,
                model: None,
            },
        )
        .unwrap();
        assert_eq!(again.id, saved.id);

        let listed = list_assets(&connection).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].prompt.as_deref(), Some("a red bicycle"));
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn delete_removes_row_and_file() {
        let (dir, connection, root) = open_db();
        let source = dir.join("gone.png");
        fs::write(&source, b"png").unwrap();
        let saved = save_asset(
            &connection,
            &root,
            SaveLibraryAsset {
                source_path: source,
                prompt: None,
                session_id: None,
                provider: None,
                model: None,
            },
        )
        .unwrap();
        let path = saved.path_in(&root);
        delete_asset(&connection, &root, &saved.id).unwrap();
        assert!(!path.exists());
        assert!(list_assets(&connection).unwrap().is_empty());
        fs::remove_dir_all(dir).ok();
    }
}
