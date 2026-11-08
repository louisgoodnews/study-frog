//! Leeches and confusion pairs of the vault (STORY-0104): the review history, the time spent in runs, the
//! stability now, and for items that lapse often their words, tags and links.

use std::collections::HashMap;

use studyfrog_core::review::Rating;
use studyfrog_core::uid::Uid;
use studyfrog_report::confusion::{
    ConfusionPair, PairInput, confusion_pairs, pair_rules_default, tag_set, token_set,
};
use studyfrog_report::leech::{LeechInput, LeechItem, find_leeches, leech_rules_default};
use studyfrog_report::session_log::SessionSample;
use studyfrog_store::analytics_rows::{ItemFact, load_all_links, load_all_tags, load_item_facts};
use studyfrog_store::error::StoreError;
use studyfrog_store::schedule::load_all_mem_states;

use crate::analytics::History;
use crate::session::VaultSession;

/// Leeches shown at most.
pub const MAX_LEECHES: usize = 50;
/// Confusion pairs shown at most.
pub const MAX_PAIRS: usize = 30;

/// A leech with what the screen shows of it.
#[derive(Debug, Clone, PartialEq)]
pub struct LeechRow {
    /// The item.
    pub uid: Uid,
    /// Its title.
    pub title: String,
    /// The folder of its stack (`""` for a loose item).
    pub folder: String,
    /// The diagnosis.
    pub leech: LeechItem,
}

/// A confusion pair with what the screen shows of it.
#[derive(Debug, Clone, PartialEq)]
pub struct PairRow {
    /// The first item and its title.
    pub a: (Uid, String),
    /// The second item and its title.
    pub b: (Uid, String),
    /// The folders of their stacks.
    pub folders: (String, String),
    /// The diagnosis.
    pub pair: ConfusionPair,
}

fn day_of(ms: i64, offset: i32) -> i64 {
    (ms + i64::from(offset) * 60_000).div_euclid(86_400_000)
}

/// Ratings and lapse days of each item, in order.
fn histories(h: &History) -> HashMap<Uid, (Vec<Rating>, Vec<i64>)> {
    let mut by: HashMap<Uid, (Vec<Rating>, Vec<i64>)> = HashMap::new();
    for r in h.rows {
        let e = by.entry(r.item).or_default();
        if r.rating == Rating::Again && !e.0.is_empty() {
            let day = day_of(r.at_ms, (h.offset_of)(r.at_ms));
            if e.1.last() != Some(&day) {
                e.1.push(day);
            }
        }
        e.0.push(r.rating);
    }
    by
}

/// The words of an item's file without its front matter, or none when it cannot be read.
fn words(session: &VaultSession, f: &ItemFact) -> Vec<u32> {
    let text = std::fs::read_to_string(session.root.join(&f.path)).unwrap_or_default();
    let body = text
        .strip_prefix("---")
        .and_then(|t| t.split_once("\n---").map(|x| x.1))
        .unwrap_or(&text);
    token_set(&format!("{} {body}", f.title))
}

fn stem(path: &str) -> String {
    let file = path.rsplit('/').next().unwrap_or(path);
    file.trim_end_matches(".md").to_lowercase()
}

struct Lookup {
    by_name: HashMap<String, u64>,
    tags: HashMap<Uid, Vec<String>>,
    links: HashMap<Uid, Vec<String>>,
}

fn lookup(session: &VaultSession, facts: &[ItemFact]) -> Result<Lookup, StoreError> {
    let mut by_name = HashMap::new();
    for (i, f) in facts.iter().enumerate() {
        by_name.insert(f.title.to_lowercase(), i as u64);
        by_name.insert(stem(&f.path), i as u64);
    }
    let mut tags: HashMap<Uid, Vec<String>> = HashMap::new();
    for (u, t) in load_all_tags(&session.conn)? {
        tags.entry(u).or_default().push(t);
    }
    let mut links: HashMap<Uid, Vec<String>> = HashMap::new();
    for (u, t) in load_all_links(&session.conn)? {
        links.entry(u).or_default().push(t.to_lowercase());
    }
    Ok(Lookup {
        by_name,
        tags,
        links,
    })
}

fn pair_input(
    session: &VaultSession,
    l: &Lookup,
    i: usize,
    f: &ItemFact,
    group: u32,
    lapses: &[i64],
) -> PairInput {
    let mut links: Vec<u64> = l
        .links
        .get(&f.uid)
        .into_iter()
        .flatten()
        .filter_map(|t| l.by_name.get(t.split('#').next().unwrap_or(t)).copied())
        .collect();
    links.sort_unstable();
    links.dedup();
    let tags = l.tags.get(&f.uid).map(Vec::as_slice).unwrap_or_default();
    PairInput {
        item: i as u64,
        group,
        tokens: words(session, f),
        tags: tag_set(tags.iter().map(String::as_str)),
        links,
        lapse_days: lapses.to_vec(),
    }
}

/// The leeches and confusion pairs, strongest first.
pub fn leeches_of(
    session: &VaultSession,
    h: &History,
    samples: &[SessionSample],
) -> Result<(Vec<LeechRow>, Vec<PairRow>), StoreError> {
    let facts = load_item_facts(&session.conn)?;
    let mut folders: Vec<String> = facts
        .iter()
        .map(|f| f.stack_path.clone().unwrap_or_default())
        .collect();
    folders.sort();
    folders.dedup();
    let group = |f: &ItemFact| {
        folders
            .binary_search(&f.stack_path.clone().unwrap_or_default())
            .unwrap_or(0) as u32
    };
    let hist = histories(h);
    let mut time: HashMap<Uid, u64> = HashMap::new();
    for s in samples {
        *time.entry(s.item).or_default() += u64::from(s.item_ms);
    }
    let mem = load_all_mem_states(&session.conn)?;
    let inputs: Vec<LeechInput> = facts
        .iter()
        .enumerate()
        .filter_map(|(i, f)| {
            let (ratings, _) = hist.get(&f.uid)?;
            Some(LeechInput {
                item: i as u64,
                group: group(f),
                ratings: ratings.clone(),
                total_ms: time.get(&f.uid).copied().unwrap_or(0),
                stability: mem.get(&f.uid).map(|m| m.state.stability),
            })
        })
        .collect();
    let folder = |i: u64| facts[i as usize].stack_path.clone().unwrap_or_default();
    let leeches = find_leeches(&inputs, &leech_rules_default())
        .into_iter()
        .take(MAX_LEECHES)
        .map(|leech| LeechRow {
            uid: facts[leech.item as usize].uid,
            title: facts[leech.item as usize].title.clone(),
            folder: folder(leech.item),
            leech,
        })
        .collect();
    let rules = pair_rules_default();
    let l = lookup(session, &facts)?;
    let pair_inputs: Vec<PairInput> = facts
        .iter()
        .enumerate()
        .filter_map(|(i, f)| {
            let (_, lapses) = hist.get(&f.uid).filter(|x| x.1.len() >= rules.min_lapses)?;
            Some(pair_input(session, &l, i, f, group(f), lapses))
        })
        .collect();
    let named = |i: u64| (facts[i as usize].uid, facts[i as usize].title.clone());
    let pairs = confusion_pairs(&pair_inputs, &rules)
        .into_iter()
        .take(MAX_PAIRS)
        .map(|pair| PairRow {
            a: named(pair.a),
            b: named(pair.b),
            folders: (folder(pair.a), folder(pair.b)),
            pair,
        })
        .collect();
    Ok((leeches, pairs))
}
