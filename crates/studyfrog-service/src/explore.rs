//! The report explorer: how the user did on the whole vault, on one stack or on one item, over a
//! period, with the stacks and items one level down.

use std::collections::{HashMap, HashSet};

use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_report::explore::{Perf, ReviewPoint, daily, perf_of};
use studyfrog_report::latency::{FlipSample, flip_samples};
use studyfrog_store::home::{IndexRow, item_index, row_in_stack};
use studyfrog_store::overview::{StackRow, load_overview};
use studyfrog_store::reports::{load_all_events, load_review_history};

pub use crate::explore_types::{Explorer, Period, ReviewLine, Row, RowKind, Scope};
use crate::session::{ServiceError, VaultSession};
use crate::subjects::stack_subjects;

const MS_PER_DAY: i64 = 86_400_000;
/// Days of the activity chart.
pub const CHART_DAYS: usize = 30;
/// Most review lines listed for one item.
pub const MAX_HISTORY: usize = 100;

fn items_in<'a>(scope: &Scope, index: &'a [IndexRow]) -> Vec<&'a IndexRow> {
    index
        .iter()
        .filter(|i| match scope {
            Scope::Vault => true,
            Scope::Stack(f) => row_in_stack(i, f),
            Scope::Item(u) => i.uid == *u,
        })
        .collect()
}

fn perf_for(items: &[&IndexRow], reviews: &[ReviewPoint], samples: &[FlipSample]) -> Perf {
    let uids: HashSet<Uid> = items.iter().map(|i| i.uid).collect();
    let mine: Vec<ReviewPoint> = reviews
        .iter()
        .filter(|r| uids.contains(&r.item))
        .copied()
        .collect();
    perf_of(&mine, samples)
}

fn crumbs(scope: &Scope, stacks: &[StackRow], index: &[IndexRow]) -> Vec<(String, Scope)> {
    let mut out = vec![("All".to_owned(), Scope::Vault)];
    let folder = match scope {
        Scope::Vault => return out,
        Scope::Stack(f) => Some(f.clone()),
        Scope::Item(u) => index
            .iter()
            .find(|i| i.uid == *u)
            .and_then(|i| i.stack.clone()),
    };
    let mut chain = Vec::new();
    let mut at = folder.and_then(|f| stacks.iter().find(|s| s.folder == f));
    while let Some(s) = at {
        chain.push((s.title.clone(), Scope::Stack(s.folder.clone())));
        at = s
            .parent
            .as_deref()
            .and_then(|p| stacks.iter().find(|x| x.folder == p));
    }
    out.extend(chain.into_iter().rev());
    if let Scope::Item(u) = scope {
        let title = index
            .iter()
            .find(|i| i.uid == *u)
            .map_or("Item", |i| i.title.as_str());
        out.push((title.to_owned(), scope.clone()));
    }
    out
}

fn rows_for(
    scope: &Scope,
    stacks: &[StackRow],
    index: &[IndexRow],
    data: (&[ReviewPoint], &[FlipSample]),
) -> Vec<Row> {
    let (reviews, samples) = data;
    let mut rows = Vec::new();
    let child_stacks: Vec<&StackRow> = match scope {
        Scope::Vault => stacks.iter().filter(|s| s.parent.is_none()).collect(),
        Scope::Stack(f) => stacks
            .iter()
            .filter(|s| s.parent.as_deref() == Some(f.as_str()))
            .collect(),
        Scope::Item(_) => Vec::new(),
    };
    for s in child_stacks {
        let target = Scope::Stack(s.folder.clone());
        let perf = perf_for(&items_in(&target, index), reviews, samples);
        rows.push(Row {
            label: s.title.clone(),
            kind: RowKind::Stack,
            target,
            perf,
        });
    }
    if let Scope::Stack(f) = scope {
        for i in index
            .iter()
            .filter(|i| i.stack.as_deref() == Some(f.as_str()) || i.also_in.iter().any(|x| x == f))
        {
            let perf = perf_for(&[i], reviews, samples);
            rows.push(Row {
                label: i.title.clone(),
                kind: RowKind::Item(i.kind),
                target: Scope::Item(i.uid),
                perf,
            });
        }
    }
    rows
}

fn subject_perfs(
    stacks: &[StackRow],
    index: &[IndexRow],
    data: (&[ReviewPoint], &[FlipSample]),
) -> Vec<(String, Perf)> {
    let mut by_subject: HashMap<String, Vec<&IndexRow>> = HashMap::new();
    for i in index {
        let names = i
            .stack
            .as_deref()
            .map_or_else(Vec::new, |f| stack_subjects(stacks, f));
        for n in names {
            by_subject.entry(n).or_default().push(i);
        }
    }
    let mut out: Vec<(String, Perf)> = by_subject
        .into_iter()
        .map(|(n, items)| (n, perf_for(&items, data.0, data.1)))
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Compute the explorer for `scope` and `period` at `now`; `offset_minutes` is the local UTC offset.
pub fn explore(
    session: &VaultSession,
    (scope, period): (Scope, Period),
    now: UnixMs,
    offset_minutes: i32,
) -> Result<Explorer, ServiceError> {
    let store = ServiceError::Store;
    let conn = &session.conn;
    let (stacks, index) = (
        load_overview(conn).map_err(store)?.stacks,
        item_index(conn).map_err(store)?,
    );
    let since = match period {
        Period::All => i64::MIN,
        Period::Days(n) => now.0 - i64::from(n) * MS_PER_DAY,
    };
    let reviews: Vec<ReviewPoint> = load_review_history(conn)
        .map_err(store)?
        .into_iter()
        .filter(|r| r.at_ms >= since)
        .map(|r| ReviewPoint {
            item: r.item,
            rating: r.rating,
            at_ms: r.at_ms,
        })
        .collect();
    let samples = flip_samples(&load_all_events(conn).map_err(store)?);
    let items = items_in(&scope, &index);
    let perf = perf_for(&items, &reviews, &samples);
    let uids: HashSet<Uid> = items.iter().map(|i| i.uid).collect();
    let mine: Vec<ReviewPoint> = reviews
        .iter()
        .filter(|r| uids.contains(&r.item))
        .copied()
        .collect();
    let today = (now.0 + i64::from(offset_minutes) * 60_000).div_euclid(MS_PER_DAY);
    let mut history: Vec<ReviewLine> = match scope {
        Scope::Item(_) => mine
            .iter()
            .map(|r| ReviewLine {
                at_ms: r.at_ms,
                rating: studyfrog_core::review::rating_value(r.rating),
            })
            .collect(),
        _ => Vec::new(),
    };
    history.sort_by_key(|l| std::cmp::Reverse(l.at_ms));
    history.truncate(MAX_HISTORY);
    let data = (reviews.as_slice(), samples.as_slice());
    Ok(Explorer {
        crumbs: crumbs(&scope, &stacks, &index),
        chart: daily(&mine, today, CHART_DAYS, offset_minutes),
        rows: rows_for(&scope, &stacks, &index, data),
        subjects: if scope == Scope::Vault {
            subject_perfs(&stacks, &index, data)
        } else {
            Vec::new()
        },
        history,
        perf,
        scope,
    })
}
