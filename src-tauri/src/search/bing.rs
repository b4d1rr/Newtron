//! Bing provider backed by the public HTML results page.
//!
//! Best-effort scraper used as a late fallback: selectors follow Bing's
//! long-stable `li.b_algo` result markup. When the markup changes or Bing
//! serves a consent/challenge page this simply yields no results and the
//! engine reports the chain's combined error.

use async_trait::async_trait;
use scraper::{Html, Selector};

use super::{favicon_for, host_of, SearchProvider, SearchResult};

const ENDPOINT: &str = "https://www.bing.com/search";
const MAX_RESULTS: usize = 8;

pub struct BingProvider;

#[async_trait]
impl SearchProvider for BingProvider {
    fn name(&self) -> &'static str {
        "Bing"
    }

    async fn search(&self, client: &reqwest::Client, query: &str) -> Result<Vec<SearchResult>, String> {
        let body = client
            .get(ENDPOINT)
            .query(&[("q", query), ("count", "10")])
            .header("Accept-Language", "en-US,en;q=0.9")
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;

        tauri::async_runtime::spawn_blocking(move || parse(&body))
            .await
            .map_err(|e| e.to_string())
    }
}

fn parse(body: &str) -> Vec<SearchResult> {
    let item_sel = Selector::parse("li.b_algo").unwrap();
    let title_sel = Selector::parse("h2 a").unwrap();
    let snippet_sel = Selector::parse(".b_caption p, p.b_lineclamp2, .b_caption .b_paractl").unwrap();

    let doc = Html::parse_document(body);
    let mut out = Vec::new();

    for item in doc.select(&item_sel).take(MAX_RESULTS) {
        let Some(link) = item.select(&title_sel).next() else {
            continue;
        };
        let title = link.text().collect::<String>().trim().to_string();
        let Some(url) = link.value().attr("href").map(str::to_string) else {
            continue;
        };
        if title.is_empty() || !url.starts_with("http") {
            continue;
        }
        let snippet = item
            .select(&snippet_sel)
            .next()
            .map(|s| s.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        let favicon = host_of(&url).map(|h| favicon_for(&h));

        out.push(SearchResult {
            title,
            snippet,
            url,
            favicon,
            source: "Bing".to_string(),
        });
    }
    out
}
