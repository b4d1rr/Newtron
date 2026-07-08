//! Ranking for local file/app search.
//!
//! score = name-match weight (+ exact-match bonus baked into the weight)
//!       + frequency-of-use term
//!       + recency-of-use term
//!
//! Matching is layered so exact/prefix hits always outrank fuzzy ones, and
//! within a tier, something opened daily outranks something opened once a
//! year ago (see module docs on `index::local` for the two-stage query that
//! feeds this).

const DAY: f64 = 86_400.0;
const RECENCY_WINDOW_DAYS: f64 = 30.0;

const W_EXACT: f64 = 500.0;
const W_PREFIX: f64 = 300.0;
const W_WORD_PREFIX: f64 = 200.0;
const W_CONTAINS: f64 = 100.0;
/// Base weight for a fuzzy (subsequence) match; reduced per character gap.
const W_FUZZY_BASE: f64 = 70.0;
const W_FUZZY_GAP_PENALTY: f64 = 3.0;
const W_FUZZY_MIN: f64 = 5.0;

const W_FREQUENCY: f64 = 20.0;
const W_RECENCY: f64 = 40.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MatchKind {
    Exact,
    Prefix,
    /// Prefix of a later word, e.g. "shop" matching "Photoshop" splits on
    /// case/word boundaries too ("photo" + "shop").
    WordPrefix,
    Contains,
    /// Every query character appears in order in the name, with `gaps`
    /// total non-matching characters between consecutive matches.
    Fuzzy { gaps: u32 },
}

impl MatchKind {
    fn weight(self) -> f64 {
        match self {
            MatchKind::Exact => W_EXACT,
            MatchKind::Prefix => W_PREFIX,
            MatchKind::WordPrefix => W_WORD_PREFIX,
            MatchKind::Contains => W_CONTAINS,
            MatchKind::Fuzzy { gaps } => (W_FUZZY_BASE - W_FUZZY_GAP_PENALTY * gaps as f64).max(W_FUZZY_MIN),
        }
    }
}

/// Split `name` into word boundaries on whitespace, punctuation, and
/// camelCase/PascalCase transitions so "shop" matches "Photoshop" and
/// "proj" matches "My Project.docx".
fn word_starts(name_lower: &str) -> Vec<usize> {
    let chars: Vec<char> = name_lower.chars().collect();
    let mut starts = vec![0usize];
    for i in 1..chars.len() {
        let prev = chars[i - 1];
        let cur = chars[i];
        if !prev.is_alphanumeric() && cur.is_alphanumeric() {
            starts.push(i);
        }
    }
    starts
}

/// Classify how (already-lowercased) `query` matches (already-lowercased)
/// `name`. Returns `None` when there is no match at all.
pub fn classify(query: &str, name_lower: &str) -> Option<MatchKind> {
    if query.is_empty() {
        return None;
    }
    if name_lower == query {
        return Some(MatchKind::Exact);
    }
    if name_lower.starts_with(query) {
        return Some(MatchKind::Prefix);
    }
    let chars: Vec<char> = name_lower.chars().collect();
    for start in word_starts(name_lower) {
        if chars[start..].iter().collect::<String>().starts_with(query) {
            return Some(MatchKind::WordPrefix);
        }
    }
    if name_lower.contains(query) {
        return Some(MatchKind::Contains);
    }
    subsequence_gaps(query, name_lower).map(|gaps| MatchKind::Fuzzy { gaps })
}

/// Greedy in-order subsequence match: every character of `query` must
/// appear in `name` in the same order (not necessarily contiguous). Returns
/// the total number of skipped characters between matches, or `None` if the
/// full query cannot be matched.
fn subsequence_gaps(query: &str, name: &str) -> Option<u32> {
    let mut qi = query.chars().peekable();
    let mut gaps: u32 = 0;
    let mut last_match: Option<usize> = None;
    for (i, c) in name.chars().enumerate() {
        let Some(&qc) = qi.peek() else { break };
        if c == qc {
            if let Some(last) = last_match {
                gaps += (i - last - 1) as u32;
            }
            last_match = Some(i);
            qi.next();
        }
    }
    if qi.peek().is_none() {
        Some(gaps)
    } else {
        None
    }
}

pub struct Candidate {
    pub match_kind: MatchKind,
    /// Times opened/launched via Newtron.
    pub usage_count: i64,
    /// Unix seconds of last open/launch, if ever.
    pub last_used: Option<i64>,
}

pub fn score(c: &Candidate, now_unix: i64) -> f64 {
    let mut s = c.match_kind.weight();
    s += W_FREQUENCY * ((c.usage_count as f64) + 1.0).ln();
    if let Some(last) = c.last_used {
        let days_ago = ((now_unix - last).max(0) as f64) / DAY;
        s += W_RECENCY * (-days_ago / RECENCY_WINDOW_DAYS).exp();
    }
    s
}
