//! Tauri command surface. Thin layer: validation + dispatch only; all
//! business logic lives in `search` and `index`.

use tauri::State;

use crate::index::{history::ImportStats, Suggestion};
use crate::search::SearchResult;
use crate::AppState;

/// Embedded web search across the provider fallback chain.
#[tauri::command]
pub async fn web_search(state: State<'_, AppState>, query: String) -> Result<Vec<SearchResult>, String> {
    state.engine.search(&query).await
}

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
