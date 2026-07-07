//! Brave Search provider (official Web Search API).
//!
//! Requires an API key; free tier available at https://brave.com/search/api/.
//! The key is read from the `BRAVE_SEARCH_API_KEY` environment variable. When
//! no key is present the provider reports itself unavailable and the engine
//! falls through to DuckDuckGo.

use async_trait::async_trait;
use serde::Deserialize;

use super::{favicon_for, host_of, SearchProvider, SearchResult};

const ENDPOINT: &str = "https://api.search.brave.com/res/v1/web/search";
const MAX_RESULTS: usize = 8;

pub struct BraveProvider {
    api_key: Option<String>,
}

impl BraveProvider {
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("BRAVE_SEARCH_API_KEY").ok().filter(|k| !k.is_empty()),
        }
    }
}

#[derive(Deserialize)]
struct BraveResponse {
    web: Option<BraveWeb>,
}

#[derive(Deserialize)]
struct BraveWeb {
    results: Vec<BraveResult>,
}

#[derive(Deserialize)]
struct BraveResult {
    title: String,
    url: String,
    #[serde(default)]
    description: String,
}

#[async_trait]
impl SearchProvider for BraveProvider {
    fn name(&self) -> &'static str {
        "Brave"
    }

    fn available(&self) -> bool {
        self.api_key.is_some()
    }

    async fn search(&self, client: &reqwest::Client, query: &str) -> Result<Vec<SearchResult>, String> {
        let key = self.api_key.as_deref().ok_or("no API key configured")?;
        let resp: BraveResponse = client
            .get(ENDPOINT)
            .query(&[("q", query)])
            .header("X-Subscription-Token", key)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        Ok(resp
            .web
            .map(|w| w.results)
            .unwrap_or_default()
            .into_iter()
            .take(MAX_RESULTS)
            .map(|r| {
                let favicon = host_of(&r.url).map(|h| favicon_for(&h));
                SearchResult {
                    title: r.title,
                    snippet: r.description,
                    url: r.url,
                    favicon,
                    source: "Brave".to_string(),
                }
            })
            .collect())
    }
}
