//! Suggestion scoring.
//!
//! Weights are tuned so that real user behavior (visits, recency) dominates
//! the built-in popularity prior once a site has been used a handful of
//! times, matching the priority order:
//!   alias > exact history match > frequency > recency > builtin > heuristics

/// Seconds in a day, used for recency decay.
const DAY: f64 = 86_400.0;
/// Recency contribution half-life-ish window (days).
const RECENCY_WINDOW_DAYS: f64 = 30.0;

const W_ALIAS: f64 = 1_000.0;
const W_EXACT_DOMAIN: f64 = 400.0;
const W_DOMAIN_PREFIX: f64 = 200.0;
const W_WORD_PREFIX: f64 = 140.0;
const W_CONTAINS: f64 = 40.0;
const W_FREQUENCY: f64 = 25.0;
const W_RECENCY: f64 = 60.0;

/// How strongly the typed text matches a candidate domain/url.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MatchKind {
    AliasExact,
    ExactDomain,
    DomainPrefix,
    /// Prefix of a later label, e.g. "wiki" matching "en.wikipedia.org".
    WordPrefix,
    Contains,
}

impl MatchKind {
    fn weight(self) -> f64 {
        match self {
            MatchKind::AliasExact => W_ALIAS,
            MatchKind::ExactDomain => W_EXACT_DOMAIN,
            MatchKind::DomainPrefix => W_DOMAIN_PREFIX,
            MatchKind::WordPrefix => W_WORD_PREFIX,
            MatchKind::Contains => W_CONTAINS,
        }
    }
}

/// Classify how `query` (already normalized) matches `domain`.
/// Returns None when it does not match at all.
pub fn classify(query: &str, domain: &str) -> Option<MatchKind> {
    let bare = domain.strip_prefix("www.").unwrap_or(domain);
    if bare == query || domain == query {
        return Some(MatchKind::ExactDomain);
    }
    if bare.starts_with(query) || domain.starts_with(query) {
        return Some(MatchKind::DomainPrefix);
    }
    // Match at the start of any dot-separated label: "wiki" -> en.WIKIpedia.org is
    // deliberately NOT a word prefix ("wikipedia" is a single label), but
    // "docs" -> docs.google.com and "google" -> maps.GOOGLE.com are.
    if bare.split('.').any(|label| label.starts_with(query)) {
        return Some(MatchKind::WordPrefix);
    }
    if bare.contains(query) {
        return Some(MatchKind::Contains);
    }
    None
}

pub struct Candidate {
    pub match_kind: MatchKind,
    pub visit_count: i64,
    /// Unix seconds of last visit, if ever visited.
    pub last_visited: Option<i64>,
    /// Popularity prior from the built-in index (0-30), 0 for history entries.
    pub base_rank: f64,
}

pub fn score(c: &Candidate, now_unix: i64) -> f64 {
    let mut s = c.match_kind.weight() + c.base_rank;
    s += W_FREQUENCY * ((c.visit_count as f64) + 1.0).ln();
    if let Some(last) = c.last_visited {
        let days_ago = ((now_unix - last).max(0) as f64) / DAY;
        s += W_RECENCY * (-days_ago / RECENCY_WINDOW_DAYS).exp();
    }
    s
}
