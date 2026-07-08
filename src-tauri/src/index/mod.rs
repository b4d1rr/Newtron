//! Persistent, adaptive URL index backed by SQLite.
//!
//! The index is seeded with a curated list of popular sites, enriched by
//! imported browser history, and continuously improved by recording the
//! user's own opens (`record_visit`). Suggestion ranking lives in `ranking`.

pub mod builtin;
pub mod history;
pub mod local;
mod local_ranking;
mod ranking;

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

use crate::search::favicon_for;

/// Maximum suggestion rows fetched from SQLite before ranking in memory.
const CANDIDATE_LIMIT: usize = 64;

#[derive(Debug, Clone, Serialize)]
pub struct Suggestion {
    pub url: String,
    pub domain: String,
    pub title: Option<String>,
    pub favicon: String,
    /// "builtin" | "history" | "user" | "alias"
    pub source: String,
    pub score: f64,
}

#[derive(Clone)]
pub struct UrlIndex {
    conn: Arc<Mutex<Connection>>,
}

impl UrlIndex {
    pub fn open(db_path: &Path) -> Result<Self, String> {
        if let Some(dir) = db_path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| e.to_string())?;
        let index = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        index.migrate()?;
        index.seed_builtin()?;
        Ok(index)
    }

    fn migrate(&self) -> Result<(), String> {
        self.conn
            .lock()
            .unwrap()
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS urls (
                    id INTEGER PRIMARY KEY,
                    url TEXT NOT NULL UNIQUE,
                    domain TEXT NOT NULL,
                    title TEXT,
                    favicon TEXT,
                    visit_count INTEGER NOT NULL DEFAULT 0,
                    last_visited INTEGER,
                    first_discovered INTEGER NOT NULL,
                    source TEXT NOT NULL DEFAULT 'user',
                    base_rank REAL NOT NULL DEFAULT 0,
                    category TEXT
                );
                CREATE INDEX IF NOT EXISTS idx_urls_domain ON urls(domain);
                CREATE TABLE IF NOT EXISTS aliases (
                    alias TEXT PRIMARY KEY,
                    url_id INTEGER NOT NULL REFERENCES urls(id) ON DELETE CASCADE
                );
                CREATE TABLE IF NOT EXISTS meta (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL
                );",
            )
            .map_err(|e| e.to_string())
    }

    /// Insert the curated site list once per seed version.
    fn seed_builtin(&self) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        let seeded: Option<String> = conn
            .query_row("SELECT value FROM meta WHERE key = 'seed_version'", [], |r| r.get(0))
            .optional()
            .map_err(|e| e.to_string())?;
        if seeded.as_deref() == Some(&builtin::SEED_VERSION.to_string()) {
            return Ok(());
        }

        let now = unix_now();
        {
            let mut stmt = conn
                .prepare(
                    "INSERT INTO urls (url, domain, title, favicon, first_discovered, source, base_rank, category)
                     VALUES (?1, ?2, ?3, ?4, ?5, 'builtin', ?6, ?7)
                     ON CONFLICT(url) DO UPDATE SET base_rank = excluded.base_rank, category = excluded.category",
                )
                .map_err(|e| e.to_string())?;
            for (domain, title, category, rank) in builtin::BUILTIN_SITES {
                let url = format!("https://{domain}");
                let host = domain.split('/').next().unwrap_or(domain);
                stmt.execute(params![url, domain, title, favicon_for(host), now, rank, category])
                    .map_err(|e| e.to_string())?;
            }
        }
        conn.execute(
            "INSERT INTO meta (key, value) VALUES ('seed_version', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![builtin::SEED_VERSION.to_string()],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Live URL suggestions for the (partial) text the user has typed.
    pub fn suggest(&self, query: &str, limit: usize) -> Vec<Suggestion> {
        let q = normalize_query(query);
        if q.is_empty() {
            return vec![];
        }
        let now = unix_now();
        let conn = self.conn.lock().unwrap();
        let mut out: Vec<Suggestion> = Vec::new();

        // 1. Alias exact match takes absolute priority.
        if let Ok(Some((url, domain, title, favicon))) = conn
            .query_row(
                "SELECT u.url, u.domain, u.title, u.favicon FROM aliases a
                 JOIN urls u ON u.id = a.url_id WHERE a.alias = ?1",
                params![q],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, Option<String>>(2)?,
                        r.get::<_, Option<String>>(3)?,
                    ))
                },
            )
            .optional()
        {
            let fav = favicon.unwrap_or_else(|| favicon_for(&domain));
            out.push(Suggestion {
                url,
                domain,
                title,
                favicon: fav,
                source: "alias".into(),
                score: ranking::score(
                    &ranking::Candidate {
                        match_kind: ranking::MatchKind::AliasExact,
                        visit_count: 0,
                        last_visited: None,
                        base_rank: 0.0,
                    },
                    now,
                ),
            });
        }

        // 2. Candidate rows from the index, ranked in memory.
        let pattern = format!("%{}%", q.replace('%', "").replace('_', ""));
        let mut stmt = match conn.prepare(
            "SELECT url, domain, title, favicon, visit_count, last_visited, source, base_rank
             FROM urls WHERE domain LIKE ?1
             ORDER BY visit_count DESC, base_rank DESC LIMIT ?2",
        ) {
            Ok(s) => s,
            Err(_) => return out,
        };
        let rows = stmt.query_map(params![pattern, CANDIDATE_LIMIT as i64], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, Option<i64>>(5)?,
                r.get::<_, String>(6)?,
                r.get::<_, f64>(7)?,
            ))
        });
        if let Ok(rows) = rows {
            for row in rows.flatten() {
                let (url, domain, title, favicon, visit_count, last_visited, source, base_rank) = row;
                let Some(match_kind) = ranking::classify(&q, &domain) else {
                    continue;
                };
                let score = ranking::score(
                    &ranking::Candidate {
                        match_kind,
                        visit_count,
                        last_visited,
                        base_rank,
                    },
                    now,
                );
                let fav = favicon.unwrap_or_else(|| favicon_for(&domain));
                out.push(Suggestion {
                    url,
                    domain,
                    title,
                    favicon: fav,
                    source,
                    score,
                });
            }
        }

        out.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        out.dedup_by(|a, b| a.domain == b.domain);
        out.truncate(limit);
        out
    }

    /// Record that the user opened a URL. This is the feedback loop that
    /// makes the index adapt: visited URLs climb the ranking.
    pub fn record_visit(&self, url: &str, title: Option<&str>) {
        let Some(domain) = crate::search::host_of(url) else {
            return;
        };
        let now = unix_now();
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute(
            "INSERT INTO urls (url, domain, title, favicon, visit_count, last_visited, first_discovered, source)
             VALUES (?1, ?2, ?3, ?4, 1, ?5, ?5, 'user')
             ON CONFLICT(url) DO UPDATE SET
                visit_count = visit_count + 1,
                last_visited = ?5,
                title = COALESCE(?3, title)",
            params![url, domain, title, favicon_for(&domain), now],
        );
    }

    /// Create or replace a user-defined alias (e.g. "yt" -> youtube.com).
    pub fn add_alias(&self, alias: &str, url: &str) -> Result<(), String> {
        let alias = normalize_query(alias);
        let conn = self.conn.lock().unwrap();
        let url_id: Option<i64> = conn
            .query_row("SELECT id FROM urls WHERE url = ?1", params![url], |r| r.get(0))
            .optional()
            .map_err(|e| e.to_string())?;
        let url_id = match url_id {
            Some(id) => id,
            None => {
                let domain = crate::search::host_of(url).ok_or("invalid url")?;
                conn.execute(
                    "INSERT INTO urls (url, domain, favicon, first_discovered, source)
                     VALUES (?1, ?2, ?3, ?4, 'user')",
                    params![url, domain, favicon_for(&domain), unix_now()],
                )
                .map_err(|e| e.to_string())?;
                conn.last_insert_rowid()
            }
        };
        conn.execute(
            "INSERT INTO aliases (alias, url_id) VALUES (?1, ?2)
             ON CONFLICT(alias) DO UPDATE SET url_id = excluded.url_id",
            params![alias, url_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Bulk-merge imported browser history rows. Visit counts from browsers
    /// are merged with MAX so re-imports don't inflate counts.
    pub fn merge_history(&self, rows: &[history::HistoryRow]) -> usize {
        let mut conn = self.conn.lock().unwrap();
        let Ok(tx) = conn.transaction() else {
            return 0;
        };
        let mut merged = 0usize;
        {
            let Ok(mut stmt) = tx.prepare(
                "INSERT INTO urls (url, domain, title, favicon, visit_count, last_visited, first_discovered, source)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'history')
                 ON CONFLICT(url) DO UPDATE SET
                    visit_count = MAX(urls.visit_count, excluded.visit_count),
                    last_visited = MAX(COALESCE(urls.last_visited, 0), COALESCE(excluded.last_visited, 0)),
                    title = COALESCE(urls.title, excluded.title)",
            ) else {
                return 0;
            };
            let now = unix_now();
            for row in rows {
                let Some(domain) = crate::search::host_of(&row.url) else {
                    continue;
                };
                if stmt
                    .execute(params![
                        row.url,
                        domain,
                        row.title,
                        favicon_for(&domain),
                        row.visit_count,
                        row.last_visited,
                        now
                    ])
                    .is_ok()
                {
                    merged += 1;
                }
            }
        }
        let _ = tx.commit();
        merged
    }

    /// Timestamp gate so history import runs at most once per interval.
    pub fn should_import_history(&self, min_interval_secs: i64) -> bool {
        let conn = self.conn.lock().unwrap();
        let last: Option<String> = conn
            .query_row("SELECT value FROM meta WHERE key = 'last_history_import'", [], |r| r.get(0))
            .optional()
            .ok()
            .flatten();
        match last.and_then(|v| v.parse::<i64>().ok()) {
            Some(t) => unix_now() - t > min_interval_secs,
            None => true,
        }
    }

    pub fn mark_history_imported(&self) {
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute(
            "INSERT INTO meta (key, value) VALUES ('last_history_import', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![unix_now().to_string()],
        );
    }
}

/// Lowercase, trim, and strip scheme/`www.` so "HTTPS://www.GitHub" and
/// "github" match the same rows.
fn normalize_query(query: &str) -> String {
    let q = query.trim().to_lowercase();
    let q = q.strip_prefix("https://").or_else(|| q.strip_prefix("http://")).unwrap_or(&q);
    let q = q.strip_prefix("www.").unwrap_or(q);
    q.trim_end_matches('/').to_string()
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
