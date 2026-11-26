//! Extract real prompt/answer pairs from vault flashcards and questions for Match mode (EPIC-075).
//!
//! Complies with Invariant P1: zero inherent impl blocks; all logic lives in pure free functions.

use std::cmp::Ordering;
use std::collections::HashSet;

use studyfrog_core::plan::ItemKind;
use studyfrog_core::time::UnixMs;
use studyfrog_md::doc::Doc;

use crate::content::{load_doc, unescape_newlines};
use crate::library_rows::{LibraryItem, load_library_rows};
use crate::session::{ServiceError, VaultSession};

const DAY_MS: i64 = 86_400_000;

/// How Match mode prioritizes cards and questions from the selected scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MatchFilter {
    /// Draw from every card and question in scope.
    #[default]
    All,
    /// Favor due and overdue items first.
    DueFirst,
    /// Favor unstudied and lowest-familiarity items first.
    LeastFamiliar,
}

/// Parameters for loading Match mode pairs from the open vault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchPairRequest {
    /// Vault-relative stack folder, or `None` for the entire vault.
    pub stack_folder: Option<String>,
    /// Ordering / prioritization mode.
    pub filter: MatchFilter,
    /// Current wall-clock timestamp used to evaluate due status and familiarity.
    pub now: UnixMs,
}

/// Loaded pool of cleaned `(prompt, answer)` pairs ready for round sampling.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MatchPairPool {
    /// Cleaned, deduplicated `(prompt, answer)` pairs ordered by the active filter.
    pub pairs: Vec<(String, String)>,
    /// Total card/question items scanned in the chosen scope before deduplication.
    pub total_candidates: usize,
}

fn next_xorshift64(state: &mut u64) -> u64 {
    let mut x = if *state == 0 {
        0x853c_49e6_748f_ea9b
    } else {
        *state
    };
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

fn replace_wikilinks(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(start) = rest.find("[[") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        if let Some(end) = after.find("]]") {
            let inner = &after[..end];
            let shown = inner.split_once('|').map_or(inner, |(_, alias)| alias);
            out.push_str(shown.trim());
            rest = &after[end + 2..];
        } else {
            out.push_str("[[");
            rest = after;
        }
    }
    out.push_str(rest);
    out
}

fn replace_md_links(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let is_img = bytes[i] == b'!' && i + 1 < bytes.len() && bytes[i + 1] == b'[';
        let open = if is_img { i + 1 } else { i };
        if bytes[open] == b'['
            && let Some(close_rel) = input[open + 1..].find("](")
        {
            let label_end = open + 1 + close_rel;
            let url_start = label_end + 2;
            if let Some(paren_rel) = input[url_start..].find(')') {
                out.push_str(&input[open + 1..label_end]);
                i = url_start + paren_rel + 1;
                continue;
            }
        }
        let ch = input[i..].chars().next().unwrap_or(' ');
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// Strip common Markdown syntax into clean, single-spaced plain text suitable for Match tiles.
pub fn strip_markdown(text: &str) -> String {
    let unescaped = unescape_newlines(text);
    let no_wiki = replace_wikilinks(&unescaped);
    let no_links = replace_md_links(&no_wiki);
    let mut cleaned = String::with_capacity(no_links.len());
    for raw_line in no_links.lines() {
        let mut line = raw_line.trim();
        if line.starts_with("```") {
            continue;
        }
        line = line.trim_start_matches(['#', '>', ' ']);
        if let Some(rest) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
            line = rest.trim_start();
        }
        if line.is_empty() {
            continue;
        }
        if !cleaned.is_empty() {
            cleaned.push(' ');
        }
        for ch in line.chars() {
            if !matches!(ch, '*' | '_' | '`' | '~') {
                cleaned.push(ch);
            }
        }
    }
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn doc_pair(doc: &Doc) -> Option<(String, String)> {
    let (raw_p, raw_a) = match doc {
        Doc::Card(c) => (c.front.as_str(), c.back.as_str()),
        Doc::Question(q) => (q.prompt.as_str(), q.reference.as_str()),
        Doc::Note(_) | Doc::Stack(_) | Doc::Diary(_) => return None,
    };
    let prompt = strip_markdown(raw_p);
    let answer = strip_markdown(raw_a);
    if prompt.is_empty() || answer.is_empty() || prompt.eq_ignore_ascii_case(&answer) {
        return None;
    }
    Some((prompt, answer))
}

fn in_scope(item: &LibraryItem, stack_folder: Option<&str>) -> bool {
    if !matches!(item.row.kind, ItemKind::Card | ItemKind::Question) {
        return false;
    }
    let Some(folder) = stack_folder else {
        return true;
    };
    if folder.is_empty() {
        return true;
    }
    item.row.stacks.iter().any(|s| s == folder)
}

fn cmp_items(a: &LibraryItem, b: &LibraryItem, filter: MatchFilter, today: i64) -> Ordering {
    match filter {
        MatchFilter::All => a.row.path.cmp(&b.row.path),
        MatchFilter::DueFirst => {
            let key = |it: &LibraryItem| match it.row.due_day {
                Some(d) if d <= today => (0u8, d),
                Some(d) => (1u8, d),
                None => (2u8, i64::MAX),
            };
            key(a)
                .cmp(&key(b))
                .then_with(|| a.row.path.cmp(&b.row.path))
        }
        MatchFilter::LeastFamiliar => {
            let fa = a.row.familiarity.unwrap_or(-1.0);
            let fb = b.row.familiarity.unwrap_or(-1.0);
            fa.total_cmp(&fb).then_with(|| a.row.path.cmp(&b.row.path))
        }
    }
}

/// Load cleaned, deduplicated `(prompt, answer)` pairs from the open vault for Match mode.
pub fn load_match_pairs(
    session: &VaultSession,
    req: &MatchPairRequest,
) -> Result<MatchPairPool, ServiceError> {
    let mut items: Vec<LibraryItem> = load_library_rows(session, req.now)?
        .into_iter()
        .filter(|it| in_scope(it, req.stack_folder.as_deref()))
        .collect();
    let today = req.now.0.div_euclid(DAY_MS);
    items.sort_by(|a, b| cmp_items(a, b, req.filter, today));

    let total_candidates = items.len();
    let mut seen = HashSet::new();
    let mut pairs = Vec::new();
    for item in items {
        let Ok(doc) = load_doc(session, item.uid) else {
            continue;
        };
        let Some((prompt, answer)) = doc_pair(&doc) else {
            continue;
        };
        let key = (prompt.to_lowercase(), answer.to_lowercase());
        if seen.insert(key) {
            pairs.push((prompt, answer));
        }
    }
    Ok(MatchPairPool {
        pairs,
        total_candidates,
    })
}

/// Deterministically sample up to `count` pairs from `pool` using `seed`.
pub fn sample_match_round(
    pool: &[(String, String)],
    count: usize,
    seed: u64,
    filter: MatchFilter,
) -> Vec<(String, String)> {
    if pool.is_empty() || count == 0 {
        return Vec::new();
    }
    let window = match filter {
        MatchFilter::All => pool.len(),
        MatchFilter::DueFirst | MatchFilter::LeastFamiliar => (count * 2).min(pool.len()),
    };
    let mut indices: Vec<usize> = (0..window).collect();
    let mut rng = seed;
    for i in (1..indices.len()).rev() {
        let j = (next_xorshift64(&mut rng) as usize) % (i + 1);
        indices.swap(i, j);
    }
    indices
        .into_iter()
        .take(count.min(pool.len()))
        .map(|i| pool[i].clone())
        .collect()
}
