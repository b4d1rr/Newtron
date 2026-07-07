//! Read-only browser history import.
//!
//! Strategy: browser history databases are SQLite files that are usually
//! locked while the browser runs, so we *copy* each database to a temp file
//! and read the copy. The originals are never opened for writing and never
//! modified. Every step is best-effort: a missing browser, locked file, or
//! schema surprise skips that source instead of failing the import.

use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};

/// Per-source cap keeps the index lean and the import fast.
const MAX_ROWS_PER_SOURCE: usize = 4000;
/// Chrome/WebKit epoch (1601-01-01) to Unix epoch offset in seconds.
const CHROME_EPOCH_OFFSET: i64 = 11_644_473_600;

pub struct HistoryRow {
    pub url: String,
    pub title: Option<String>,
    pub visit_count: i64,
    /// Unix seconds.
    pub last_visited: Option<i64>,
}

#[derive(Debug, Default, serde::Serialize)]
pub struct ImportStats {
    pub sources_found: usize,
    pub rows_imported: usize,
}

/// Import history from every supported browser found on this machine.
pub fn import_all(index: &super::UrlIndex) -> ImportStats {
    let mut stats = ImportStats::default();
    for db in find_history_databases() {
        let rows = match db.kind {
            BrowserKind::Chromium => read_chromium(&db.path),
            BrowserKind::Firefox => read_firefox(&db.path),
        };
        match rows {
            Ok(rows) if !rows.is_empty() => {
                stats.sources_found += 1;
                stats.rows_imported += index.merge_history(&rows);
            }
            Ok(_) => stats.sources_found += 1,
            Err(e) => log::debug!("history import skipped {:?}: {e}", db.path),
        }
    }
    index.mark_history_imported();
    stats
}

enum BrowserKind {
    Chromium,
    Firefox,
}

struct HistoryDb {
    kind: BrowserKind,
    path: PathBuf,
}

/// Locate history databases for Chrome, Edge, Brave, Arc, other Chromium
/// variants, and Firefox, across their per-profile directories.
fn find_history_databases() -> Vec<HistoryDb> {
    let mut found = Vec::new();
    let Some(local) = dirs::data_local_dir() else {
        return found;
    };
    let roaming = dirs::data_dir();

    // Chromium family: <root>/User Data/<profile>/History
    let chromium_roots = [
        local.join("Google/Chrome/User Data"),
        local.join("Microsoft/Edge/User Data"),
        local.join("BraveSoftware/Brave-Browser/User Data"),
        local.join("Chromium/User Data"),
        local.join("Vivaldi/User Data"),
        // Arc on Windows keeps a Chromium User Data dir under its package dir.
        local.join("Packages"),
    ];
    for root in chromium_roots {
        if root.ends_with("Packages") {
            // Best-effort Arc discovery: TheBrowserCompany.Arc_<hash>/LocalCache/Local/Arc/User Data
            if let Ok(entries) = std::fs::read_dir(&root) {
                for entry in entries.flatten() {
                    if entry.file_name().to_string_lossy().starts_with("TheBrowserCompany.Arc") {
                        let ud = entry.path().join("LocalCache/Local/Arc/User Data");
                        collect_chromium_profiles(&ud, &mut found);
                    }
                }
            }
        } else {
            collect_chromium_profiles(&root, &mut found);
        }
    }

    // Firefox: %APPDATA%/Mozilla/Firefox/Profiles/<profile>/places.sqlite
    if let Some(roaming) = roaming {
        let profiles = roaming.join("Mozilla/Firefox/Profiles");
        if let Ok(entries) = std::fs::read_dir(&profiles) {
            for entry in entries.flatten() {
                let places = entry.path().join("places.sqlite");
                if places.is_file() {
                    found.push(HistoryDb {
                        kind: BrowserKind::Firefox,
                        path: places,
                    });
                }
            }
        }
    }

    found
}

fn collect_chromium_profiles(user_data: &Path, found: &mut Vec<HistoryDb>) {
    let Ok(entries) = std::fs::read_dir(user_data) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "Default" || name.starts_with("Profile ") {
            let history = entry.path().join("History");
            if history.is_file() {
                found.push(HistoryDb {
                    kind: BrowserKind::Chromium,
                    path: history,
                });
            }
        }
    }
}

/// Copy a (possibly locked) database to temp and open the copy read-only.
/// The copy is deleted when the returned guard drops.
fn open_copy(path: &Path) -> Result<(Connection, TempFile), String> {
    let tmp = std::env::temp_dir().join(format!(
        "newtron-history-{}-{}.sqlite",
        std::process::id(),
        path.to_string_lossy().len() // cheap uniqueness across sources
    ));
    std::fs::copy(path, &tmp).map_err(|e| format!("copy failed: {e}"))?;
    let guard = TempFile(tmp.clone());
    let conn = Connection::open_with_flags(&tmp, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("open failed: {e}"))?;
    Ok((conn, guard))
}

struct TempFile(PathBuf);

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn read_chromium(path: &Path) -> Result<Vec<HistoryRow>, String> {
    let (conn, _guard) = open_copy(path)?;
    let mut stmt = conn
        .prepare(
            "SELECT url, title, visit_count, last_visit_time FROM urls
             WHERE visit_count > 0 AND (url LIKE 'http://%' OR url LIKE 'https://%')
             ORDER BY visit_count DESC LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([MAX_ROWS_PER_SOURCE as i64], |r| {
            Ok(HistoryRow {
                url: r.get(0)?,
                title: r.get::<_, Option<String>>(1)?.filter(|t| !t.is_empty()),
                visit_count: r.get(2)?,
                // Chromium stores microseconds since 1601-01-01.
                last_visited: r
                    .get::<_, Option<i64>>(3)?
                    .map(|t| t / 1_000_000 - CHROME_EPOCH_OFFSET)
                    .filter(|t| *t > 0),
            })
        })
        .map_err(|e| e.to_string())?
        .flatten()
        .collect();
    Ok(rows)
}

fn read_firefox(path: &Path) -> Result<Vec<HistoryRow>, String> {
    let (conn, _guard) = open_copy(path)?;
    let mut stmt = conn
        .prepare(
            "SELECT url, title, visit_count, last_visit_date FROM moz_places
             WHERE visit_count > 0 AND (url LIKE 'http://%' OR url LIKE 'https://%')
             ORDER BY visit_count DESC LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([MAX_ROWS_PER_SOURCE as i64], |r| {
            Ok(HistoryRow {
                url: r.get(0)?,
                title: r.get::<_, Option<String>>(1)?.filter(|t| !t.is_empty()),
                visit_count: r.get(2)?,
                // Firefox stores microseconds since the Unix epoch.
                last_visited: r.get::<_, Option<i64>>(3)?.map(|t| t / 1_000_000).filter(|t| *t > 0),
            })
        })
        .map_err(|e| e.to_string())?
        .flatten()
        .collect();
    Ok(rows)
}
