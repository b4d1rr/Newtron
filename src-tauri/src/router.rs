//! Command Router — single entry point for Mode 1 ("search files and web").
//!
//! By the time a query reaches here it is already known *not* to be a URL
//! or a raw system command (the frontend filters those before invoking any
//! backend search — see `lib/api.ts::looksLikeUrl`). What's left is a plain
//! query, which the router fans out to `LocalSearch` (files), `AppSearch`
//! (installed apps), and known-destination `WebSearch` "go to" suggestions,
//! merges by score, and appends a trailing `WebSearch::DefaultBrowserFallback`
//! affordance so there's always a way to fall through to the browser.

use serde::Serialize;
use tauri::State;

use crate::index::local::{AppEntry, FileEntry};
use crate::index::Suggestion;
use crate::AppState;

const MAX_FILES: usize = 6;
const MAX_APPS: usize = 4;
const MAX_SUGGESTIONS: usize = 4;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RouteItem {
    App(AppEntry),
    File(FileEntry),
    GoTo(Suggestion),
    WebSearch { query: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct RouteResult {
    pub items: Vec<RouteItem>,
}

/// Combine local app/file matches (ranked together by score) with URL
/// "go to" suggestions and a trailing "search the web" row.
pub fn search_all(state: &State<'_, AppState>, query: &str) -> RouteResult {
    let q = query.trim();
    if q.is_empty() {
        return RouteResult { items: vec![] };
    }

    let mut local: Vec<(f64, RouteItem)> = Vec::new();
    for app in state.local_index.search_apps(q, MAX_APPS) {
        local.push((app.score, RouteItem::App(app)));
    }
    for file in state.local_index.search_files(q, MAX_FILES) {
        local.push((file.score, RouteItem::File(file)));
    }
    local.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut items: Vec<RouteItem> = local.into_iter().map(|(_, item)| item).collect();
    for s in state.index.suggest(q, MAX_SUGGESTIONS) {
        items.push(RouteItem::GoTo(s));
    }
    items.push(RouteItem::WebSearch { query: q.to_string() });

    RouteResult { items }
}
