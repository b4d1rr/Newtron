//! `WebSearch::DefaultBrowserFallback` — the active web search backend.
//!
//! Per the current product direction, Newtron does not fetch or render web
//! results itself yet. Instead it hands the query straight to the user's
//! default browser, using a configurable search-engine URL template. This
//! keeps web search dead simple and dependency-free while the real
//! AI-powered retrieval layer (`APIProvider`, see `providers`) is designed
//! properly.
//!
//! This module is intentionally the *only* thing the frontend talks to for
//! web search (`commands::open_web_search`). Swapping in `providers`, a
//! different browser-launch strategy, or an agent-based retrieval backend
//! later only touches this file.

use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

/// Search engine URL template; `{query}` is replaced with the percent-
/// encoded query. This will become user-configurable (settings) once
/// Newtron has a settings surface — for now it defaults to Google.
const DEFAULT_ENGINE_TEMPLATE: &str = "https://www.google.com/search?q={query}";

pub struct DefaultBrowserFallback;

impl DefaultBrowserFallback {
    /// Build the search URL for `query` using the configured template.
    pub fn build_url(query: &str) -> String {
        DEFAULT_ENGINE_TEMPLATE.replace("{query}", &urlencoding::encode(query.trim()))
    }

    /// Hand the query off to the OS default browser. This is fire-and-forget
    /// from Newtron's point of view — once the browser opens, the query is
    /// entirely out of our hands.
    pub fn open(app: &AppHandle, query: &str) -> Result<(), String> {
        let query = query.trim();
        if query.is_empty() {
            return Err("empty query".into());
        }
        let url = Self::build_url(query);
        app.opener().open_url(url, None::<&str>).map_err(|e| e.to_string())
    }
}
