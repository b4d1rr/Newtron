//! WebSearch module (SearchManager architecture: `WebSearch`).
//!
//! Two backends live here:
//!   - `browser_fallback::DefaultBrowserFallback` — the active backend.
//!     Hands the query straight to the OS default browser. See its module
//!     docs. This is what `commands::open_web_search` calls.
//!   - `SearchEngine` (this file) + `bing`/`brave`/`duckduckgo`/`google` —
//!     the future `APIProvider` backend: an in-process provider chain with
//!     fallback and result caching, still fully functional and reachable
//!     via `commands::web_search`, but not wired into the default UI flow
//!     while web search stays intentionally simple.
//!
//! Providers below are attempted in priority order; a provider that errors
//! is skipped for `FAILURE_TTL` so a dead endpoint never adds latency to
//! every keystroke.

pub mod browser_fallback;
mod bing;
mod brave;
mod duckduckgo;
mod google;

use std::time::Duration;

use async_trait::async_trait;
use serde::Serialize;

use crate::cache::TtlCache;

/// How long successful search results are served from cache.
const RESULT_TTL: Duration = Duration::from_secs(300);
/// How long a failed provider is skipped before being retried.
const FAILURE_TTL: Duration = Duration::from_secs(120);
/// Per-request network timeout. Keeps the UI snappy even when a provider hangs.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(6);
/// Desktop browser UA; the HTML endpoints we use serve degraded or empty
/// markup to unknown agents.
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0 Safari/537.36";

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub title: String,
    pub snippet: String,
    pub url: String,
    pub favicon: Option<String>,
    /// Which provider produced this result (e.g. "DuckDuckGo").
    pub source: String,
}

#[async_trait]
pub trait SearchProvider: Send + Sync {
    /// Human-readable provider name, also used as the failure-cache key.
    fn name(&self) -> &'static str;
    /// Whether the provider is currently usable (e.g. has an API key).
    fn available(&self) -> bool {
        true
    }
    async fn search(&self, client: &reqwest::Client, query: &str) -> Result<Vec<SearchResult>, String>;
}

pub struct SearchEngine {
    client: reqwest::Client,
    providers: Vec<Box<dyn SearchProvider>>,
    results: TtlCache<String, Vec<SearchResult>>,
    failures: TtlCache<&'static str, ()>,
}

impl SearchEngine {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("failed to build http client");

        // Priority order. Google (official Custom Search API) runs first when
        // its key is configured, then Brave (API key), then the keyless
        // fallbacks: DuckDuckGo lite and finally the Bing HTML page. Keyed
        // providers silently skip themselves when unconfigured.
        let providers: Vec<Box<dyn SearchProvider>> = vec![
            Box::new(google::GoogleProvider::from_env()),
            Box::new(brave::BraveProvider::from_env()),
            Box::new(duckduckgo::DuckDuckGoProvider),
            Box::new(bing::BingProvider),
        ];

        Self {
            client,
            providers,
            results: TtlCache::new(RESULT_TTL),
            failures: TtlCache::new(FAILURE_TTL),
        }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>, String> {
        let key = query.trim().to_lowercase();
        if key.is_empty() {
            return Ok(vec![]);
        }
        if let Some(hit) = self.results.get(&key) {
            return Ok(hit);
        }

        let mut errors: Vec<String> = Vec::new();
        for provider in &self.providers {
            if !provider.available() || self.failures.contains(&provider.name()) {
                continue;
            }
            match provider.search(&self.client, query).await {
                Ok(results) if !results.is_empty() => {
                    self.results.put(key, results.clone());
                    return Ok(results);
                }
                Ok(_) => errors.push(format!("{}: no results", provider.name())),
                Err(e) => {
                    self.failures.put(provider.name(), ());
                    errors.push(format!("{}: {}", provider.name(), e));
                }
            }
        }
        Err(if errors.is_empty() {
            "no search providers available".to_string()
        } else {
            errors.join("; ")
        })
    }
}

/// Favicon URL for a domain, served by DuckDuckGo's icon proxy.
pub(crate) fn favicon_for(domain: &str) -> String {
    format!("https://external-content.duckduckgo.com/ip3/{domain}.ico")
}

/// Extract the host from a URL string without pulling in a URL crate.
pub(crate) fn host_of(url: &str) -> Option<String> {
    let rest = url.split_once("://").map(|(_, r)| r).unwrap_or(url);
    let host = rest.split(['/', '?', '#']).next()?;
    if host.is_empty() {
        None
    } else {
        Some(host.to_string())
    }
}
