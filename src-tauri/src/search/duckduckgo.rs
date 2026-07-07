//! DuckDuckGo provider backed by the keyless `html.duckduckgo.com` endpoint.

use async_trait::async_trait;
use scraper::{Html, Selector};

use super::{favicon_for, host_of, SearchProvider, SearchResult};

const ENDPOINT: &str = "https://html.duckduckgo.com/html/";
const MAX_RESULTS: usize = 8;

pub struct DuckDuckGoProvider;

#[async_trait]
impl SearchProvider for DuckDuckGoProvider {
    fn name(&self) -> &'static str {
        "DuckDuckGo"
    }

    async fn search(&self, client: &reqwest::Client, query: &str) -> Result<Vec<SearchResult>, String> {
        let body = client
            .get(ENDPOINT)
            .query(&[("q", query)])
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;

        // Parsing happens on a blocking thread: `Html` is cheap for one page
        // but this keeps the async runtime free on principle.
        tauri::async_runtime::spawn_blocking(move || parse(&body))
            .await
            .map_err(|e| e.to_string())
    }
}

fn parse(body: &str) -> Vec<SearchResult> {
    // Selectors are static strings; unwrap is safe.
    let result_sel = Selector::parse("div.result__body").unwrap();
    let title_sel = Selector::parse("h2.result__title a.result__a").unwrap();
    let snippet_sel = Selector::parse("a.result__snippet, div.result__snippet").unwrap();

    let doc = Html::parse_document(body);
    let mut out = Vec::new();

    for item in doc.select(&result_sel).take(MAX_RESULTS) {
        let Some(link) = item.select(&title_sel).next() else {
            continue;
        };
        let title = link.text().collect::<String>().trim().to_string();
        let Some(url) = link.value().attr("href").and_then(decode_redirect) else {
            continue;
        };
        if title.is_empty() {
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
            source: "DuckDuckGo".to_string(),
        });
    }
    out
}

/// Result links point at DDG's redirect (`//duckduckgo.com/l/?uddg=<real-url>`);
/// unwrap them so we store and open the destination directly.
fn decode_redirect(href: &str) -> Option<String> {
    if let Some(idx) = href.find("uddg=") {
        let encoded = &href[idx + 5..];
        let encoded = encoded.split('&').next()?;
        return urlencoding::decode(encoded).ok().map(|c| c.into_owned());
    }
    // Some result types link directly.
    if href.starts_with("http") {
        return Some(href.to_string());
    }
    if href.starts_with("//") {
        return Some(format!("https:{href}"));
    }
    None
}
