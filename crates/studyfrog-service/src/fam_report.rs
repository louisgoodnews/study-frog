//! The familiarity report: the spread of values, the least familiar items per stack and per subject, and the value of one
//! item over time. Each row says which inputs drove its value.

use studyfrog_core::status::{effective_status, status_from_name};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_report::familiarity::{
    FamContext, FamEvent, Familiarity, fam_params_default, familiarity, stack_familiarity,
};
use studyfrog_store::error::StoreError;
use studyfrog_store::item_meta::load_item_meta;
use studyfrog_store::overview::{StackRow, load_indexed_items, load_overview};
use studyfrog_store::reviews::load_all_reviews_timed;
use studyfrog_store::views::load_all_views;

use crate::fam_all::fam_context;
use crate::library_rows::{events_of, exams};
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

struct Scored {
    uid: Uid,
    title: String,
    folder: Option<String>,
    priority: studyfrog_core::priority::Priority,
    fam: Option<Familiarity>,
    why: String,
}

fn score_all(
    conn: &rusqlite::Connection,
    stacks: &[StackRow],
    now: UnixMs,
) -> Result<Vec<Scored>, StoreError> {
    let by_folder = exams(conn, stacks, now.0.div_euclid(DAY_MS))?;
    let (reviews, views) = (load_all_reviews_timed(conn)?, load_all_views(conn)?);
    let meta = load_item_meta(conn)?;
    let params = fam_params_default();
    let mut out = Vec::new();
    for item in load_indexed_items(conn)? {
        let Some(m) = meta.get(&item.uid) else {
            continue;
        };
        let status = effective_status(
            m.status.as_deref().and_then(status_from_name),
            m.reviews > 0,
        );
        let exam = m
            .stack
            .as_ref()
            .and_then(|f| by_folder.get(f))
            .map(|d| UnixMs(d * DAY_MS));
        let ctx = fam_context(&item, status, exam);
        let events = events_of(reviews.get(&item.uid), views.get(&item.uid));
        out.push(Scored {
            uid: item.uid,
            title: m.title.clone(),
            folder: m.stack.clone(),
            priority: item.priority,
            fam: familiarity(&events, now, &ctx, &params),
            why: drivers(&events, &ctx, now),
        });
    }
    Ok(out)
}

fn group(name: String, members: &[&Scored]) -> FamGroup {
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
        percent: stack_familiarity(&scored).map(|f| percent(f.value)),
        items: members.len(),
        least,
    }
}

fn inside(folder: &Option<String>, stack: &str) -> bool {
    folder
        .as_deref()
        .is_some_and(|f| f == stack || f.starts_with(&format!("{stack}/")))
}

/// The report at `now`.
pub fn familiarity_report(
    conn: &rusqlite::Connection,
    now: UnixMs,
) -> Result<FamReport, StoreError> {
    let stacks = load_overview(conn)?.stacks;
    let all = score_all(conn, &stacks, now)?;
    let mut spread = [0_u32; 10];
    for f in all.iter().filter_map(|s| s.fam) {
        spread[usize::from((percent(f.value) / 10).min(9))] += 1;
    }
    let per_stack = stacks
        .iter()
        .map(|st| {
            group(
                st.title.clone(),
                &all.iter()
                    .filter(|s| inside(&s.folder, &st.folder))
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
        unscored: all.iter().filter(|s| s.fam.is_none()).count(),
        stacks: per_stack,
        subjects: per_subject,
    })
}
