//! Google provider via the official Custom Search JSON API.
//!
//! Google no longer serves a scrapeable no-JS results page and its ToS
//! prohibits automated scraping, so this uses the sanctioned API instead.
//! Setup (free tier ~100 queries/day):
//!   1. Create an API key: https://developers.google.com/custom-search/v1/introduction
//!   2. Create a Programmable Search Engine that searches the entire web
//!      and copy its "cx" id: https://programmablesearchengine.google.com/
//!   3. Set env vars GOOGLE_SEARCH_API_KEY and GOOGLE_SEARCH_CX.
//! Without both variables the provider reports unavailable and the engine
//! falls through to the keyless providers.

use async_trait::async_trait;
use serde::Deserialize;

use super::{favicon_for, host_of, SearchProvider, SearchResult};

const ENDPOINT: &str = "https://www.googleapis.com/customsearch/v1";
const MAX_RESULTS: usize = 8;

pub struct GoogleProvider {
    api_key: Option<String>,
    cx: Option<String>,
}

impl GoogleProvider {
    pub fn from_env() -> Self {
        let get = |name: &str| std::env::var(name).ok().filter(|v| !v.is_empty());
        Self {
            api_key: get("GOOGLE_SEARCH_API_KEY"),
            cx: get("GOOGLE_SEARCH_CX"),
        }
    }
}

#[derive(Deserialize)]
struct GoogleResponse {
    #[serde(default)]
    items: Vec<GoogleItem>,
}

#[derive(Deserialize)]
struct GoogleItem {
    title: String,
    link: String,
    #[serde(default)]
    snippet: String,
}

#[async_trait]
impl SearchProvider for GoogleProvider {
    fn name(&self) -> &'static str {
        "Google"
    }

    fn available(&self) -> bool {
        self.api_key.is_some() && self.cx.is_some()
    }

    async fn search(&self, client: &reqwest::Client, query: &str) -> Result<Vec<SearchResult>, String> {
        let (Some(key), Some(cx)) = (self.api_key.as_deref(), self.cx.as_deref()) else {
            return Err("no API key configured".into());
        };
        let resp: GoogleResponse = client
            .get(ENDPOINT)
            .query(&[("key", key), ("cx", cx), ("q", query), ("num", "8")])
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        Ok(resp
            .items
            .into_iter()
            .take(MAX_RESULTS)
            .map(|r| {
                let favicon = host_of(&r.link).map(|h| favicon_for(&h));
                SearchResult {
                    title: r.title,
                    snippet: r.snippet,
                    url: r.link,
                    favicon,
                    source: "Google".to_string(),
                }
            })
            .collect())
    }
}
