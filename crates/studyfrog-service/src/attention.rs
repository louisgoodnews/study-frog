//! Items that need attention: those the user knows least, with the reasons.

use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_store::error::StoreError;
use studyfrog_store::item_meta::load_item_meta;
use studyfrog_store::overview::{IndexedItem, load_overview};

use crate::fam_all::item_familiarities;
use crate::library_rows::exams;

const DAY_MS: i64 = 86_400_000;
/// Below this familiarity an item is listed.
const LOW: f32 = 0.5;
/// With an exam this close (days) the bar is higher.
const EXAM_DAYS: i64 = 14;
/// Familiarity under which an item with a close exam is listed.
const LOW_BEFORE_EXAM: f32 = 0.7;
/// How many items the card lists.
const SHOWN: usize = 5;

/// An item that needs attention.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attention {
    /// The item.
    pub uid: Uid,
    /// Its title.
    pub title: String,
    /// How familiar the user is with it, in percent.
    pub percent: u8,
    /// Why it is listed.
    pub reasons: Vec<String>,
}

/// The least familiar items, each with its reasons: the low value, being overdue, a close exam.
pub fn needs_attention(
    conn: &rusqlite::Connection,
    items: &[IndexedItem],
    now: UnixMs,
) -> Result<Vec<Attention>, StoreError> {
    let today = now.0.div_euclid(DAY_MS);
    let fams = item_familiarities(conn, items, now)?;
    let stacks = load_overview(conn)?.stacks;
    let by_folder = exams(conn, &stacks, today)?;
    let meta = load_item_meta(conn)?;
    let mut list = Vec::new();
    for (uid, fam) in fams {
        let Some(m) = meta.get(&uid) else { continue };
        let days_to_exam = m
            .stack
            .as_ref()
            .and_then(|f| by_folder.get(f))
            .map(|d| d - today)
            .filter(|d| (0..=EXAM_DAYS).contains(d));
        let bar = if days_to_exam.is_some() {
            LOW_BEFORE_EXAM
        } else {
            LOW
        };
        if fam >= bar {
            continue;
        }
        let mut reasons = vec![format!("Familiarity {} %", (fam * 100.0).round())];
        if let Some(due) = m.due_ms.filter(|d| *d < now.0) {
            reasons.push(format!(
                "{} day(s) overdue",
                (now.0 - due).div_euclid(DAY_MS)
            ));
        }
        if let Some(d) = days_to_exam {
            reasons.push(format!("Exam in {d} day(s)"));
        }
        let percent = (fam * 100.0).round() as u8;
        let title = m.title.clone();
        list.push((
            fam,
            Attention {
                uid,
                title,
                percent,
                reasons,
            },
        ));
    }
    list.sort_by(|(fa, a), (fb, b)| fa.total_cmp(fb).then(a.uid.cmp(&b.uid)));
    list.truncate(SHOWN);
    Ok(list.into_iter().map(|(_, a)| a).collect())
}
