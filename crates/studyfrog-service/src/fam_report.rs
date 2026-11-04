//! The familiarity report: the spread of values, the least familiar items per stack and per subject, and the value of one
//! item over time. Each row says which inputs drove its value.

use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_report::familiarity::{FamContext, FamEvent, Familiarity, stack_familiarity};
use studyfrog_store::error::StoreError;
use studyfrog_store::item_meta::load_item_meta;
use studyfrog_store::overview::{StackRow, load_indexed_items, load_overview};

use crate::fam_refs::{average_over, reference_scores};
use crate::subjects::stack_subjects;

const DAY_MS: i64 = 86_400_000;
/// How many of the least familiar items each group lists.
const LEAST: usize = 3;

/// One item of the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FamRow {
    /// The item.
    pub uid: Uid,
    /// Its title.
    pub title: String,
    /// Its familiarity in percent.
    pub percent: u8,
    /// The inputs behind the value, in words.
    pub drivers: String,
}

/// A stack or a subject with its roll-up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FamGroup {
    /// Its name.
    pub name: String,
    /// Its folder for a stack; empty for a subject.
    pub folder: String,
    /// The roll-up in percent, `None` when no item has a history.
    pub percent: Option<u8>,
    /// Items in it.
    pub items: usize,
    /// The least familiar items.
    pub least: Vec<FamRow>,
}

/// The whole report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FamReport {
    /// Items per tenth of the scale (0-9 %, 10-19 %, ... 90-100 %).
    pub spread: [u32; 10],
    /// Items with no history yet.
    pub unscored: usize,
    /// Per stack.
    pub stacks: Vec<FamGroup>,
    /// Per subject.
    pub subjects: Vec<FamGroup>,
}

pub(crate) fn percent(v: f32) -> u8 {
    (v * 100.0).round().clamp(0.0, 100.0) as u8
}

pub(crate) fn drivers(events: &[FamEvent], ctx: &FamContext, now: UnixMs) -> String {
    let ratings: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            FamEvent::Rating { at, rating, .. } => Some((*at, *rating)),
            _ => None,
        })
        .collect();
    let views = events
        .iter()
        .filter(|e| matches!(e, FamEvent::View { .. }))
        .count();
    let mut parts = Vec::new();
    if let Some((when, rating)) = ratings.iter().max_by_key(|(t, _)| *t) {
        let ago = (now.0 - when.0).div_euclid(DAY_MS);
        let last = format!("{rating:?}").to_lowercase();
        parts.push(format!(
            "{} rating(s), the last {last} {ago} day(s) ago",
            ratings.len()
        ));
    }
    if views > 0 {
        parts.push(format!("{views} view(s)"));
    }
    if let Some(exam) = ctx.exam.filter(|e| e.0 >= now.0) {
        parts.push(format!(
            "exam in {} day(s)",
            (exam.0 - now.0).div_euclid(DAY_MS)
        ));
    }
    if ctx.done {
        parts.push("finished, so it does not fall".to_owned());
    }
    parts.join("; ")
}

pub(crate) struct Scored {
    pub(crate) uid: Uid,
    pub(crate) title: String,
    pub(crate) folder: Option<String>,
    pub(crate) also_in: Vec<String>,
    pub(crate) priority: studyfrog_core::priority::Priority,
    pub(crate) fam: Option<Familiarity>,
    pub(crate) why: String,
}

/// One entry for every (item, stack): the stack's own value of the item. `also_in` is empty, the stack is `folder`.
pub(crate) fn score_all(
    conn: &rusqlite::Connection,
    _stacks: &[StackRow],
    now: UnixMs,
) -> Result<Vec<Scored>, StoreError> {
    let meta = load_item_meta(conn)?;
    let priority: std::collections::HashMap<Uid, _> = load_indexed_items(conn)?
        .into_iter()
        .map(|i| (i.uid, i.priority))
        .collect();
    let mut out = Vec::new();
    for s in reference_scores(conn, now)? {
        let (Some(m), Some(priority)) = (meta.get(&s.uid), priority.get(&s.uid)) else {
            continue;
        };
        out.push(Scored {
            uid: s.uid,
            title: m.title.clone(),
            folder: s.folder,
            also_in: Vec::new(),
            priority: *priority,
            fam: s.fam,
            why: drivers(&s.events, &s.ctx, now),
        });
    }
    Ok(out)
}

pub(crate) fn group(name: String, members: &[&Scored]) -> FamGroup {
    let scored: Vec<(_, Familiarity)> = members
        .iter()
        .filter_map(|s| s.fam.map(|f| (s.priority, f)))
        .collect();
    let mut rows: Vec<&Scored> = members
        .iter()
        .copied()
        .filter(|s| s.fam.is_some())
        .collect();
    rows.sort_by(|a, b| {
        let (fa, fb) = (
            a.fam.map_or(0.0, |f| f.value),
            b.fam.map_or(0.0, |f| f.value),
        );
        fa.total_cmp(&fb).then(a.uid.cmp(&b.uid))
    });
    let least = rows
        .into_iter()
        .take(LEAST)
        .map(|s| FamRow {
            uid: s.uid,
            title: s.title.clone(),
            percent: percent(s.fam.map_or(0.0, |f| f.value)),
            drivers: s.why.clone(),
        })
        .collect();
    FamGroup {
        name,
        folder: String::new(),
        percent: stack_familiarity(&scored).map(|f| percent(f.value)),
        items: members.len(),
        least,
    }
}

/// Whether the scored item belongs to the stack at `stack` or inside it, through its folder or a reference.
pub(crate) fn in_scored(s: &Scored, stack: &str) -> bool {
    let below = |f: &str| f == stack || f.starts_with(&format!("{stack}/"));
    s.folder.as_deref().is_some_and(below) || s.also_in.iter().any(|f| below(f))
}

/// The report at `now`.
pub fn familiarity_report(
    conn: &rusqlite::Connection,
    now: UnixMs,
) -> Result<FamReport, StoreError> {
    let stacks = load_overview(conn)?.stacks;
    let all = score_all(conn, &stacks, now)?;
    let mut spread = [0_u32; 10];
    let mut by_item: std::collections::BTreeMap<Uid, Vec<f32>> = std::collections::BTreeMap::new();
    for s in &all {
        let values = by_item.entry(s.uid).or_default();
        values.extend(s.fam.map(|f| f.value));
    }
    let averages: Vec<Option<f32>> = by_item
        .values()
        .map(|v| average_over(v.iter().copied()))
        .collect();
    for v in averages.iter().flatten() {
        spread[usize::from((percent(*v) / 10).min(9))] += 1;
    }
    let per_stack = stacks
        .iter()
        .map(|st| FamGroup {
            folder: st.folder.clone(),
            ..group(
                st.title.clone(),
                &all.iter()
                    .filter(|s| in_scored(s, &st.folder))
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    let mut names: Vec<String> = stacks
        .iter()
        .flat_map(|s| stack_subjects(&stacks, &s.folder))
        .collect();
    names.sort();
    names.dedup();
    let per_subject = names
        .into_iter()
        .map(|n| {
            let members: Vec<&Scored> = all
                .iter()
                .filter(|s| {
                    s.folder
                        .as_deref()
                        .is_some_and(|f| stack_subjects(&stacks, f).contains(&n))
                })
                .collect();
            group(n, &members)
        })
        .collect();
    Ok(FamReport {
        spread,
        unscored: averages.iter().filter(|v| v.is_none()).count(),
        stacks: per_stack,
        subjects: per_subject,
    })
}
