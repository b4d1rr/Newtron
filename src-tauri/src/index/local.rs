//! Local file & application index — SQLite-backed metadata store.
//!
//! Stores metadata only, never file contents (see `versionPlan`/roadmap:
//! full-text indexing is a later, Tantivy-backed feature). Three tables:
//!   - `files`: one row per indexed file under a scanned folder
//!   - `apps`: one row per discovered application (Start Menu shortcuts)
//!   - `indexed_folders`: user-configured scan roots (Desktop/Documents/... by
//!     default, extendable from settings)
//!
//! Shares the on-disk database with `UrlIndex` (same `newtron.db` file, its
//! own `Connection`) but is a fully independent module so the file indexer
//! can evolve without touching URL/web-search code.

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

use super::local_ranking::{self, Candidate};

/// Rows pulled from SQLite before in-memory ranking. Keeps a single query
/// fast even against a large index.
const SQL_CANDIDATE_LIMIT: usize = 200;
/// Cap on the bounded fallback full-table fuzzy scan (see `search_files`).
const FUZZY_SCAN_LIMIT: usize = 4000;
/// If the fast substring query returns fewer rows than this, fall back to
/// the (bounded) fuzzy scan so typo-tolerant queries still find something.
const FUZZY_FALLBACK_THRESHOLD: usize = 6;

#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    pub id: i64,
    pub name: String,
    pub full_path: String,
    pub extension: Option<String>,
    pub size: i64,
    pub created_date: i64,
    pub modified_date: i64,
    pub last_accessed: Option<i64>,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppEntry {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub icon: Option<String>,
    pub usage_count: i64,
    pub last_opened: Option<i64>,
    pub score: f64,
}

/// Metadata the indexer has just read from disk for one file.
pub struct NewFile {
    pub name: String,
    pub full_path: String,
    pub extension: Option<String>,
    pub size: i64,
    pub created_date: i64,
    pub modified_date: i64,
}

/// Metadata the app-discovery scanner has just read for one shortcut.
pub struct NewApp {
    pub name: String,
    pub path: String,
    pub icon: Option<String>,
}

#[derive(Clone)]
pub struct LocalIndex {
    conn: Arc<Mutex<Connection>>,
}

impl LocalIndex {
    pub fn open(db_path: &Path) -> Result<Self, String> {
        if let Some(dir) = db_path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
        conn.pragma_update(None, "journal_mode", "WAL").map_err(|e| e.to_string())?;
        conn.busy_timeout(std::time::Duration::from_secs(5)).map_err(|e| e.to_string())?;
        let index = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        index.migrate()?;
        Ok(index)
    }

    fn migrate(&self) -> Result<(), String> {
        self.conn
            .lock()
            .unwrap()
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS files (
                    id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL,
                    name_lower TEXT NOT NULL,
                    full_path TEXT NOT NULL UNIQUE,
                    extension TEXT,
                    size INTEGER NOT NULL DEFAULT 0,
                    created_date INTEGER NOT NULL DEFAULT 0,
                    modified_date INTEGER NOT NULL DEFAULT 0,
                    last_accessed INTEGER,
                    open_count INTEGER NOT NULL DEFAULT 0,
                    last_seen_scan INTEGER NOT NULL DEFAULT 0
                );
                CREATE INDEX IF NOT EXISTS idx_files_name_lower ON files(name_lower);
                CREATE INDEX IF NOT EXISTS idx_files_path ON files(full_path);

                CREATE TABLE IF NOT EXISTS apps (
                    id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL,
                    name_lower TEXT NOT NULL,
                    path TEXT NOT NULL UNIQUE,
                    icon TEXT,
                    usage_count INTEGER NOT NULL DEFAULT 0,
                    last_opened INTEGER,
                    last_seen_scan INTEGER NOT NULL DEFAULT 0
                );
                CREATE INDEX IF NOT EXISTS idx_apps_name_lower ON apps(name_lower);

                CREATE TABLE IF NOT EXISTS indexed_folders (
                    path TEXT PRIMARY KEY,
                    added_at INTEGER NOT NULL,
                    is_default INTEGER NOT NULL DEFAULT 0
                );

                CREATE TABLE IF NOT EXISTS local_meta (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL
                );",
            )
            .map_err(|e| e.to_string())
    }

    // ---------------------------------------------------------------
    // Indexed folders (settings)
    // ---------------------------------------------------------------

    pub fn add_folder(&self, path: &str, is_default: bool) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO indexed_folders (path, added_at, is_default) VALUES (?1, ?2, ?3)
             ON CONFLICT(path) DO NOTHING",
            params![path, unix_now(), is_default as i64],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn remove_folder(&self, path: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM indexed_folders WHERE path = ?1", params![path])
            .map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM files WHERE full_path = ?1 OR full_path LIKE ?2",
            params![path, format!("{}%", with_trailing_sep(path))],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_folders(&self) -> Vec<String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = match conn.prepare("SELECT path FROM indexed_folders ORDER BY added_at ASC") {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        stmt.query_map([], |r| r.get::<_, String>(0))
            .map(|rows| rows.flatten().collect())
            .unwrap_or_default()
    }

    // ---------------------------------------------------------------
    // Files
    // ---------------------------------------------------------------

    /// Insert or update a file row, stamping it with the current scan id so
    /// `prune_stale` can later tell it apart from deleted files.
    pub fn upsert_file(&self, f: &NewFile, scan_id: i64) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO files (name, name_lower, full_path, extension, size, created_date, modified_date, last_seen_scan)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(full_path) DO UPDATE SET
                name = excluded.name,
                name_lower = excluded.name_lower,
                extension = excluded.extension,
                size = excluded.size,
                modified_date = excluded.modified_date,
                last_seen_scan = excluded.last_seen_scan",
            params![
                f.name,
                f.name.to_lowercase(),
                f.full_path,
                f.extension,
                f.size,
                f.created_date,
                f.modified_date,
                scan_id
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Delete files under `root` that were not touched by scan `scan_id`
    /// (i.e. they disappeared from disk since the previous pass).
    pub fn prune_stale_files(&self, root: &str, scan_id: i64) -> usize {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM files WHERE last_seen_scan != ?1 AND (full_path = ?2 OR full_path LIKE ?3)",
            params![scan_id, root, format!("{}%", with_trailing_sep(root))],
        )
        .unwrap_or(0)
    }

    pub fn remove_file_path(&self, path: &str) -> usize {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM files WHERE full_path = ?1", params![path]).unwrap_or(0)
    }

    pub fn record_file_access(&self, path: &str) {
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute(
            "UPDATE files SET open_count = open_count + 1, last_accessed = ?2 WHERE full_path = ?1",
            params![path, unix_now()],
        );
    }

    /// Fuzzy/partial file search, ranked by name match + recency + frequency.
    pub fn search_files(&self, query: &str, limit: usize) -> Vec<FileEntry> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return vec![];
        }
        let now = unix_now();
        let conn = self.conn.lock().unwrap();
        let mut seen = std::collections::HashSet::new();
        let mut out: Vec<FileEntry> = Vec::new();

        let pattern = format!("%{}%", escape_like(&q));
        if let Ok(mut stmt) = conn.prepare(
            "SELECT id, name, full_path, extension, size, created_date, modified_date, last_accessed, open_count, name_lower
             FROM files WHERE name_lower LIKE ?1 ESCAPE '\\'
             ORDER BY open_count DESC, last_accessed DESC LIMIT ?2",
        ) {
            if let Ok(rows) = stmt.query_map(params![pattern, SQL_CANDIDATE_LIMIT as i64], map_file_row) {
                for row in rows.flatten() {
                    push_scored_file(&mut out, &mut seen, row, &q, now);
                }
            }
        }

        if out.len() < FUZZY_FALLBACK_THRESHOLD {
            if let Ok(mut stmt) = conn.prepare(
                "SELECT id, name, full_path, extension, size, created_date, modified_date, last_accessed, open_count, name_lower
                 FROM files ORDER BY last_accessed DESC, modified_date DESC LIMIT ?1",
            ) {
                if let Ok(rows) = stmt.query_map(params![FUZZY_SCAN_LIMIT as i64], map_file_row) {
                    for row in rows.flatten() {
                        push_scored_file(&mut out, &mut seen, row, &q, now);
                    }
                }
            }
        }

        out.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        out.truncate(limit);
        out
    }

    // ---------------------------------------------------------------
    // Apps
    // ---------------------------------------------------------------

    pub fn upsert_app(&self, a: &NewApp, scan_id: i64) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO apps (name, name_lower, path, icon, last_seen_scan)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(path) DO UPDATE SET
                name = excluded.name,
                name_lower = excluded.name_lower,
                icon = excluded.icon,
                last_seen_scan = excluded.last_seen_scan",
            params![a.name, a.name.to_lowercase(), a.path, a.icon, scan_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn prune_stale_apps(&self, scan_id: i64) -> usize {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM apps WHERE last_seen_scan != ?1", params![scan_id])
            .unwrap_or(0)
    }

    pub fn record_app_launch(&self, path: &str) {
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute(
            "UPDATE apps SET usage_count = usage_count + 1, last_opened = ?2 WHERE path = ?1",
            params![path, unix_now()],
        );
    }

    /// App table is small (hundreds of rows at most) so we just rank the
    /// whole thing in memory on every query instead of round-tripping
    /// through a SQL prefilter.
    pub fn search_apps(&self, query: &str, limit: usize) -> Vec<AppEntry> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return vec![];
        }
        let now = unix_now();
        let conn = self.conn.lock().unwrap();
        let mut out: Vec<AppEntry> = Vec::new();
        let Ok(mut stmt) = conn.prepare("SELECT id, name, path, icon, usage_count, last_opened, name_lower FROM apps") else {
            return out;
        };
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, Option<i64>>(5)?,
                r.get::<_, String>(6)?,
            ))
        });
        if let Ok(rows) = rows {
            for row in rows.flatten() {
                let (id, name, path, icon, usage_count, last_opened, name_lower) = row;
                let Some(match_kind) = local_ranking::classify(&q, &name_lower) else {
                    continue;
                };
                let score = local_ranking::score(
                    &Candidate {
                        match_kind,
                        usage_count,
                        last_used: last_opened,
                    },
                    now,
                );
                out.push(AppEntry {
                    id,
                    name,
                    path,
                    icon,
                    usage_count,
                    last_opened,
                    score,
                });
            }
        }
        out.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        out.truncate(limit);
        out
    }

    pub fn should_reindex(&self, key: &str, min_interval_secs: i64) -> bool {
        let conn = self.conn.lock().unwrap();
        let last: Option<String> = conn
            .query_row("SELECT value FROM local_meta WHERE key = ?1", params![key], |r| r.get(0))
            .optional()
            .ok()
            .flatten();
        match last.and_then(|v| v.parse::<i64>().ok()) {
            Some(t) => unix_now() - t > min_interval_secs,
            None => true,
        }
    }

    pub fn mark_reindexed(&self, key: &str) {
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute(
            "INSERT INTO local_meta (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, unix_now().to_string()],
        );
    }

    pub fn next_scan_id(&self) -> i64 {
        unix_now()
    }
}

type FileRow = (i64, String, String, Option<String>, i64, i64, i64, Option<i64>, i64, String);

fn map_file_row(r: &rusqlite::Row) -> rusqlite::Result<FileRow> {
    Ok((
        r.get(0)?,
        r.get(1)?,
        r.get(2)?,
        r.get(3)?,
        r.get(4)?,
        r.get(5)?,
        r.get(6)?,
        r.get(7)?,
        r.get(8)?,
        r.get(9)?,
    ))
}

fn push_scored_file(
    out: &mut Vec<FileEntry>,
    seen: &mut std::collections::HashSet<i64>,
    row: FileRow,
    query: &str,
    now: i64,
) {
    let (id, name, full_path, extension, size, created_date, modified_date, last_accessed, open_count, name_lower) = row;
    if !seen.insert(id) {
        return;
    }
    let Some(match_kind) = local_ranking::classify(query, &name_lower) else {
        return;
    };
    let score = local_ranking::score(
        &Candidate {
            match_kind,
            usage_count: open_count,
            last_used: last_accessed,
        },
        now,
    );
    out.push(FileEntry {
        id,
        name,
        full_path,
        extension,
        size,
        created_date,
        modified_date,
        last_accessed,
        score,
    });
}

/// Escape `%`, `_`, and `\` for a `LIKE ... ESCAPE '\'` pattern.
fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
}

fn with_trailing_sep(path: &str) -> String {
    if path.ends_with('\\') || path.ends_with('/') {
        path.to_string()
    } else {
        format!("{path}\\")
    }
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
