//! DuckDuckGo provider backed by the keyless `lite.duckduckgo.com` endpoint.
//!
//! The lite page is a plain HTML table designed for text browsers — cheaper
//! to fetch and parse than the full HTML endpoint. DDG rate-limits automated
//! traffic; when it serves a challenge page instead of results we surface an
//! error (not an empty result set) so the engine's failure cache backs off
//! and the next provider takes over.

use async_trait::async_trait;
use scraper::{Html, Selector};

use super::{favicon_for, host_of, SearchProvider, SearchResult};

const ENDPOINT: &str = "https://lite.duckduckgo.com/lite/";
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

        if body.contains("anomaly") || body.contains("challenge") {
            return Err("rate-limited (bot challenge)".into());
        }

        tauri::async_runtime::spawn_blocking(move || parse(&body))
            .await
            .map_err(|e| e.to_string())
    }
}

fn parse(body: &str) -> Vec<SearchResult> {
    let link_sel = Selector::parse("a.result-link").unwrap();
    let snippet_sel = Selector::parse("td.result-snippet").unwrap();

    let doc = Html::parse_document(body);
    let snippets: Vec<String> = doc
        .select(&snippet_sel)
        .map(|s| s.text().collect::<String>().trim().to_string())
        .collect();

    let mut out = Vec::new();
    for (i, link) in doc.select(&link_sel).take(MAX_RESULTS).enumerate() {
        let title = link.text().collect::<String>().trim().to_string();
        let Some(url) = link.value().attr("href").and_then(decode_redirect) else {
            continue;
        };
        if title.is_empty() {
            continue;
        }
        let favicon = host_of(&url).map(|h| favicon_for(&h));
        out.push(SearchResult {
            title,
            snippet: snippets.get(i).cloned().unwrap_or_default(),
            url,
            favicon,
            source: "DuckDuckGo".to_string(),
        });
    }
    out
}

/// Result links may point at DDG's redirect (`//duckduckgo.com/l/?uddg=<url>`);
/// unwrap them so we store and open the destination directly.
fn decode_redirect(href: &str) -> Option<String> {
    if let Some(idx) = href.find("uddg=") {
        let encoded = &href[idx + 5..];
        let encoded = encoded.split('&').next()?;
        return urlencoding::decode(encoded).ok().map(|c| c.into_owned());
    }
    if href.starts_with("http") {
        return Some(href.to_string());
    }
    if href.starts_with("//") {
        return Some(format!("https:{href}"));
    }
    None
}
