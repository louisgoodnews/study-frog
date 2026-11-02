//! The rows of the library list: every item of the vault as a query sees it.

use std::collections::HashMap;

use studyfrog_calendar::session::EventKind;
use studyfrog_core::civil::date_to_days;
use studyfrog_core::plan::ItemKind;
use studyfrog_core::query_eval::Row;
use studyfrog_core::status::{Status, effective_status, status_from_name};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_report::familiarity::{FamContext, FamEvent, fam_params_default, familiarity};
use studyfrog_store::item_meta::{ItemMeta, load_item_meta};
use studyfrog_store::items::{IndexedItem, load_indexed_items};
use studyfrog_store::overview::{StackRow, load_overview};
use studyfrog_store::reviews::{TimedReview, load_all_reviews_timed};
use studyfrog_store::sessions::load_sessions;
use studyfrog_store::views::load_all_views;

use crate::familiarity::pick_exam;
use crate::session::{ServiceError, VaultSession};
use crate::subjects::{load_subjects, stack_subjects, subjects_with_teachers};
use studyfrog_store::error::StoreError;

const DAY_MS: i64 = 86_400_000;

/// An item of the library with the row the query language sees.
#[derive(Debug, Clone, PartialEq)]
pub struct LibraryItem {
    /// The item.
    pub uid: Uid,
    /// What a query can ask about it.
    pub row: Row,
}

/// A stack's folder and title and those of every stack above it, for `stack = ...`.
fn stack_chain(stacks: &[StackRow], folder: Option<&str>) -> Vec<String> {
    let Some(folder) = folder else {
        return Vec::new();
    };
    let mut chain = Vec::new();
    let mut upto = String::new();
    for part in folder.split('/') {
        upto = if upto.is_empty() {
            part.to_owned()
        } else {
            format!("{upto}/{part}")
        };
        chain.push(upto.clone());
        let title = stacks
            .iter()
            .find(|s| s.folder == upto)
            .map(|s| s.title.clone());
        chain.push(title.unwrap_or_else(|| upto.clone()));
    }
    chain
}

/// The exam or deadline that matters for each stack: its own or one above it.
pub(crate) fn exams(
    conn: &rusqlite::Connection,
    stacks: &[StackRow],
    today: i64,
) -> Result<HashMap<String, i64>, StoreError> {
    let mut direct: HashMap<Uid, Vec<i64>> = HashMap::new();
    for s in load_sessions(conn)? {
        if let (true, Some(stack)) = (
            matches!(s.kind, EventKind::Exam | EventKind::Deadline),
            s.stack,
        ) {
            direct
                .entry(stack)
                .or_default()
                .push(date_to_days(s.start.date));
        }
    }
    let mut out = HashMap::new();
    for stack in stacks {
        let days: Vec<i64> = stacks
            .iter()
            .filter(|a| {
                stack.folder == a.folder || stack.folder.starts_with(&format!("{}/", a.folder))
            })
            .flat_map(|a| direct.get(&a.uid).cloned().unwrap_or_default())
            .collect();
        if let Some(d) = pick_exam(&days, today) {
            out.insert(stack.folder.clone(), d);
        }
    }
    Ok(out)
}

fn events_of(
    reviews: Option<&Vec<TimedReview>>,
    views: Option<&Vec<studyfrog_core::views::ViewEvent>>,
) -> Vec<FamEvent> {
    let rated = reviews
        .into_iter()
        .flatten()
        .map(|(r, t)| FamEvent::Rating {
            at: r.at,
            rating: r.rating,
            answer_ms: t.map(|t| t.0),
            expected_ms: t.map(|t| t.1),
        });
    let seen = views.into_iter().flatten().map(|v| FamEvent::View {
        at: v.at,
        active_ms: v.active_ms,
        expected_ms: v.expected_ms,
    });
    rated.chain(seen).collect()
}

fn day_of(ms: Option<i64>) -> Option<i64> {
    ms.map(|m| m.div_euclid(DAY_MS))
}

fn row_of((item, meta): (&IndexedItem, &ItemMeta), shared: &Shared, now: UnixMs) -> Row {
    let status = effective_status(
        meta.status.as_deref().and_then(status_from_name),
        meta.reviews > 0,
    );
    let subjects = stack_subjects(shared.stacks, meta.stack.as_deref().unwrap_or(""));
    let teachers = subjects_with_teachers(&shared.book, &subjects)
        .into_iter()
        .filter_map(|(_, t)| t)
        .collect();
    let exam = meta
        .stack
        .as_ref()
        .and_then(|f| shared.exams.get(f))
        .map(|d| UnixMs(d * DAY_MS));
    let ctx = FamContext {
        is_note: item.kind == ItemKind::Note,
        mark: item.difficulty,
        done: status == Status::Done,
        exam,
    };
    let events = events_of(shared.reviews.get(&item.uid), shared.views.get(&item.uid));
    Row {
        path: meta.path.clone(),
        title: meta.title.clone(),
        body: meta.body.clone(),
        kind: item.kind,
        status,
        stacks: stack_chain(shared.stacks, meta.stack.as_deref()),
        subjects,
        teachers,
        tags: item.tags.clone(),
        priority: item.priority,
        difficulty: item.difficulty,
        familiarity: familiarity(&events, now, &ctx, &fam_params_default()).map(|f| f.value),
        due_day: day_of(meta.due_ms),
        reviews: meta.reviews,
        created_day: day_of(meta.created_ms),
        updated_day: day_of(meta.updated_ms),
    }
}

/// What every row needs of the rest of the vault.
struct Shared<'a> {
    stacks: &'a [StackRow],
    book: studyfrog_config::subjects::SubjectBook,
    exams: HashMap<String, i64>,
    reviews: HashMap<Uid, Vec<TimedReview>>,
    views: HashMap<Uid, Vec<studyfrog_core::views::ViewEvent>>,
}

/// Every item of the vault (stacks excluded) as a row, in path order.
pub fn load_library_rows(
    session: &VaultSession,
    now: UnixMs,
) -> Result<Vec<LibraryItem>, ServiceError> {
    rows_of(session, now).map_err(ServiceError::Store)
}

fn rows_of(session: &VaultSession, now: UnixMs) -> Result<Vec<LibraryItem>, StoreError> {
    let conn = &session.conn;
    let stacks = load_overview(conn)?.stacks;
    let shared = Shared {
        exams: exams(conn, &stacks, now.0.div_euclid(DAY_MS))?,
        book: load_subjects(session).0,
        reviews: load_all_reviews_timed(conn)?,
        views: load_all_views(conn)?,
        stacks: &stacks,
    };
    let meta = load_item_meta(conn)?;
    let rows = load_indexed_items(conn)?
        .iter()
        .filter_map(|item| {
            meta.get(&item.uid).map(|m| LibraryItem {
                uid: item.uid,
                row: row_of((item, m), &shared, now),
            })
        })
        .collect();
    Ok(rows)
}
