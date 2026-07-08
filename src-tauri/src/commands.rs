//! Tauri command surface. Thin layer: validation + dispatch only; all
//! business logic lives in `search`, `index`, and `router`.

use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use crate::index::{history::ImportStats, Suggestion};
use crate::router::{self, RouteResult};
use crate::search::SearchResult;
use crate::AppState;

/// Live URL autocomplete for the typed prefix.
#[tauri::command]
pub fn url_suggest(state: State<'_, AppState>, query: String, limit: Option<usize>) -> Vec<Suggestion> {
    state.index.suggest(&query, limit.unwrap_or(6).min(20))
}

/// Feedback loop: called right before a URL is opened so ranking adapts.
#[tauri::command]
pub fn record_visit(state: State<'_, AppState>, url: String, title: Option<String>) {
    state.index.record_visit(&url, title.as_deref());
}

/// User-defined shortcut, e.g. alias "yt" -> https://youtube.com.
#[tauri::command]
pub fn add_alias(state: State<'_, AppState>, alias: String, url: String) -> Result<(), String> {
    state.index.add_alias(&alias, &url)
}

/// Manual trigger for browser history import (also runs on startup).
#[tauri::command]
pub async fn import_history(state: State<'_, AppState>) -> Result<ImportStats, String> {
    let index = state.index.clone();
    tauri::async_runtime::spawn_blocking(move || crate::index::history::import_all(&index))
        .await
        .map_err(|e| e.to_string())
}

/// Embedded web search across the provider fallback chain. Kept as the
/// future `APIProvider` path (see `search::providers` docs) — not called by
/// the default UI flow, which uses `open_web_search` instead.
#[tauri::command]
pub async fn web_search(state: State<'_, AppState>, query: String) -> Result<Vec<SearchResult>, String> {
    state.engine.search(&query).await
}

/// Mode 1 ("search files and web"): local files + apps + go-to suggestions,
/// merged and ranked, plus a trailing web-search affordance.
#[tauri::command]
pub fn search_all(state: State<'_, AppState>, query: String) -> RouteResult {
    router::search_all(&state, &query)
}

/// Hand a plain query straight to the user's default browser
/// (`WebSearch::DefaultBrowserFallback` — see `search::browser_fallback`).
#[tauri::command]
pub fn open_web_search(app: AppHandle, query: String) -> Result<(), String> {
    crate::search::browser_fallback::DefaultBrowserFallback::open(&app, &query)
}

/// Open a locally-indexed file or app with the OS default handler, and
/// record the open so ranking adapts (frequency/recency).
#[tauri::command]
pub fn open_local_item(app: AppHandle, state: State<'_, AppState>, path: String, kind: String) -> Result<(), String> {
    if kind == "app" {
        state.local_index.record_app_launch(&path);
    } else {
        state.local_index.record_file_access(&path);
    }
    app.opener().open_path(path, None::<&str>).map_err(|e| e.to_string())
}

/// Add a folder to the local index's scan roots and index it immediately
/// (in the background) rather than waiting for the next periodic scan.
#[tauri::command]
pub fn add_indexed_folder(state: State<'_, AppState>, path: String) -> Result<(), String> {
    state.local_index.add_folder(&path, false)?;
    let index = state.local_index.clone();
    let folder = path.clone();
    std::thread::spawn(move || crate::indexer::index_folder_now(&index, &folder));
    Ok(())
}

/// Remove a folder from the scan roots and drop everything indexed under it.
#[tauri::command]
pub fn remove_indexed_folder(state: State<'_, AppState>, path: String) -> Result<(), String> {
    state.local_index.remove_folder(&path)
}

#[tauri::command]
pub fn list_indexed_folders(state: State<'_, AppState>) -> Vec<String> {
    state.local_index.list_folders()
}
