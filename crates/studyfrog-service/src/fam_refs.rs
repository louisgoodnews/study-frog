//! Familiarity for each (item, stack) the item belongs to: a stack counts the reviews it made and its own exam, and
//! the value of the item as a whole is the average over its references (ADR 0011, TASK-0219).

use std::collections::HashMap;

use studyfrog_core::civil::{date_from_days, format_date};
use studyfrog_core::status::{Status, effective_status, status_from_name};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_report::familiarity::{
    FamContext, FamEvent, Familiarity, fam_params_default, familiarity,
};
use studyfrog_store::error::StoreError;
use studyfrog_store::item_meta::load_item_meta;
use studyfrog_store::overview::{StackRow, load_indexed_items, load_overview};
use studyfrog_store::reviews_by_stack::load_all_reviews_by_stack;
use studyfrog_store::schedule_stacks::load_stack_schedules;
use studyfrog_store::stack_scope::stack_homes;
use studyfrog_store::views::load_all_views;

use crate::fam_all::fam_context;
use crate::library_rows::{events_of, exams};
use crate::settings::scheduler_for;

const DAY_MS: i64 = 86_400_000;

/// What one stack makes of one item.
#[derive(Debug, Clone, PartialEq)]
pub struct RefScore {
    /// The item.
    pub uid: Uid,
    /// The stack (`Uid(0)` for the Inbox).
    pub stack: Uid,
    /// The stack's folder, `None` for the Inbox.
    pub folder: Option<String>,
    /// Whether this is the item's primary stack (its folder) rather than a reference.
    pub primary: bool,
    /// What the history of this stack says, `None` without one.
    pub fam: Option<Familiarity>,
    /// The history behind it, for explaining the value.
    pub events: Vec<FamEvent>,
    /// What the value depends on besides the history.
    pub ctx: FamContext,
    /// The FSRS difficulty of the item in this stack, if it has FSRS state there.
    pub difficulty: Option<f32>,
    /// When the item is next due in this stack.
    pub due: Option<UnixMs>,
    /// How many reviews this stack has of it.
    pub reviews: u32,
}

/// The average of the values, `None` for none.
pub fn average_over(values: impl IntoIterator<Item = f32>) -> Option<f32> {
    let (sum, n) = values
        .into_iter()
        .fold((0.0_f32, 0_u32), |(s, n), v| (s + v, n + 1));
    (n > 0).then(|| sum / n as f32)
}

fn folder_of(stacks: &[StackRow], uid: Uid) -> Option<String> {
    stacks
        .iter()
        .find(|s| s.uid == uid)
        .map(|s| s.folder.clone())
}

/// The score of every (item, stack) at `now`, items in path order and each item's primary stack first.
pub fn reference_scores(
    conn: &rusqlite::Connection,
    now: UnixMs,
) -> Result<Vec<RefScore>, StoreError> {
    let stacks = load_overview(conn)?.stacks;
    let by_folder = exams(conn, &stacks, now.0.div_euclid(DAY_MS))?;
    let (reviews, views) = (load_all_reviews_by_stack(conn)?, load_all_views(conn)?);
    let meta = load_item_meta(conn)?;
    let homes = stack_homes(conn)?;
    let vault = studyfrog_core::settings::vault_settings_default();
    let sched = load_stack_schedules(conn, |s| scheduler_for(&vault, homes.get(&s)))?;
    let params = fam_params_default();
    let mut out = Vec::new();
    for item in load_indexed_items(conn)? {
        let Some(m) = meta.get(&item.uid) else {
            continue;
        };
        let status: Status = effective_status(
            m.status.as_deref().and_then(status_from_name),
            m.reviews > 0,
        );
        let holders =
            std::iter::once((item.stack, true)).chain(item.also_in.iter().map(|s| (*s, false)));
        for (stack, primary) in holders {
            let folder = folder_of(&stacks, stack);
            let exam = folder
                .as_ref()
                .and_then(|f| by_folder.get(f))
                .map(|d| UnixMs(d * DAY_MS));
            let own = reviews.get(&(item.uid, stack));
            let events = events_of(own, views.get(&item.uid));
            let ctx = fam_context(&item, status, exam);
            let kept = sched.get(&(item.uid, stack));
            out.push(RefScore {
                uid: item.uid,
                stack,
                folder,
                primary,
                fam: familiarity(&events, now, &ctx, &params),
                events,
                ctx,
                difficulty: kept.and_then(|s| s.memory).map(|m| m.difficulty),
                due: kept.map(|s| s.due),
                reviews: own.map_or(0, |r| r.len() as u32),
            });
        }
    }
    Ok(out)
}

/// The value of every item that has one, as the average over its references.
pub fn item_averages(scores: &[RefScore]) -> HashMap<Uid, f32> {
    let mut by_item: HashMap<Uid, Vec<f32>> = HashMap::new();
    for s in scores {
        if let Some(f) = s.fam {
            by_item.entry(s.uid).or_default().push(f.value);
        }
    }
    by_item
        .into_iter()
        .filter_map(|(uid, v)| average_over(v).map(|a| (uid, a)))
        .collect()
}

/// What one stack makes of an item, for the item page and the library's tooltip.
#[derive(Debug, Clone, PartialEq)]
pub struct RefView {
    /// The stack (`Uid(0)` for the Inbox).
    pub stack: Uid,
    /// The stack's title, "Inbox" for loose items.
    pub title: String,
    /// The source: the item's own stack (`true`) or a stack that references it.
    pub primary: bool,
    /// Familiarity from 0 to 1, `None` without a history in this stack.
    pub value: Option<f32>,
    /// The same in percent.
    pub percent: Option<u8>,
    /// The FSRS difficulty in this stack.
    pub difficulty: Option<f32>,
    /// When it is next due in this stack, in milliseconds.
    pub due_ms: Option<i64>,
    /// How many reviews this stack has of it.
    pub reviews: u32,
}

/// The references of each item, primary stack first.
pub fn views_by_item(scores: &[RefScore], stacks: &[StackRow]) -> HashMap<Uid, Vec<RefView>> {
    let mut out: HashMap<Uid, Vec<RefView>> = HashMap::new();
    for s in scores {
        let title = stacks
            .iter()
            .find(|st| st.uid == s.stack)
            .map_or_else(|| "Inbox".to_owned(), |st| st.title.clone());
        out.entry(s.uid).or_default().push(RefView {
            stack: s.stack,
            title,
            primary: s.primary,
            value: s.fam.map(|f| f.value),
            percent: s.fam.map(|f| ((f.value * 100.0).round() as u8).min(100)),
            difficulty: s.difficulty,
            due_ms: s.due.map(|d| d.0),
            reviews: s.reviews,
        });
    }
    out
}

/// The average familiarity of an item over its references (0 to 1) and the average FSRS difficulty.
pub fn overall(refs: &[RefView]) -> (Option<f32>, Option<f32>) {
    let fam = average_over(refs.iter().filter_map(|r| r.value));
    (fam, average_over(refs.iter().filter_map(|r| r.difficulty)))
}

/// One line for each stack that holds the item: its title, whether it is the item's own stack, familiarity,
/// difficulty and due date.
pub fn per_stack(refs: &[RefView]) -> String {
    refs.iter()
        .map(|r| {
            let source = if r.primary { "own" } else { "referenced" };
            let fam = r.percent.map_or("–".to_owned(), |p| format!("{p} %"));
            let diff = r.difficulty.map_or("–".to_owned(), |d| format!("{d:.1}"));
            let due = r.due_ms.map_or("–".to_owned(), |ms| {
                format_date(date_from_days(ms.div_euclid(86_400_000)))
            });
            format!(
                "{} ({source}): familiarity {fam}, difficulty {diff}, due {due}",
                r.title
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
