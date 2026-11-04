//! The value of one item over time, for the item page.

use studyfrog_core::status::{effective_status, status_from_name};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_report::familiarity::{FamEvent, fam_params_default, familiarity};
use studyfrog_store::error::StoreError;
use studyfrog_store::item_meta::load_item_meta;
use studyfrog_store::overview::{load_indexed_items, load_overview};
use studyfrog_store::reviews::load_all_reviews_timed;
use studyfrog_store::views::load_all_views;

use crate::fam_all::fam_context;
use crate::fam_refs::{RefView, reference_scores, views_by_item};
use crate::fam_report::{drivers, percent};
use crate::library_rows::{events_of, exams};

const DAY_MS: i64 = 86_400_000;

/// A point of an item's curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurvePoint {
    /// The day, counted from 1970-01-01 (UTC).
    pub day: i64,
    /// The value in percent.
    pub percent: u8,
}

/// An item's value now, over time, and what drove it.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemCurve {
    /// After each event, and now.
    pub points: Vec<CurvePoint>,
    /// The inputs behind the value now.
    pub drivers: String,
    /// What each stack that holds the item makes of it; the value now is their average.
    pub references: Vec<RefView>,
}

fn at(e: &FamEvent) -> UnixMs {
    match e {
        FamEvent::Rating { at, .. } | FamEvent::View { at, .. } | FamEvent::Edit { at } => *at,
    }
}

/// The value of one item after each event and now, with what drove it.
pub fn item_curve(
    conn: &rusqlite::Connection,
    uid: Uid,
    now: UnixMs,
) -> Result<ItemCurve, StoreError> {
    let stacks = load_overview(conn)?.stacks;
    let by_folder = exams(conn, &stacks, now.0.div_euclid(DAY_MS))?;
    let meta = load_item_meta(conn)?;
    let (Some(m), Some(item)) = (
        meta.get(&uid),
        load_indexed_items(conn)?.into_iter().find(|i| i.uid == uid),
    ) else {
        return Ok(ItemCurve {
            points: Vec::new(),
            drivers: String::new(),
            references: Vec::new(),
        });
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
    let (reviews, views) = (load_all_reviews_timed(conn)?, load_all_views(conn)?);
    let mut events = events_of(reviews.get(&uid), views.get(&uid));
    events.sort_by_key(at);
    let params = fam_params_default();
    let mut points: Vec<CurvePoint> = (0..events.len())
        .filter_map(|k| {
            let when = at(&events[k]);
            familiarity(&events[..=k], when, &ctx, &params).map(|f| CurvePoint {
                day: when.0.div_euclid(DAY_MS),
                percent: percent(f.value),
            })
        })
        .collect();
    if let Some(f) = familiarity(&events, now, &ctx, &params) {
        points.push(CurvePoint {
            day: now.0.div_euclid(DAY_MS),
            percent: percent(f.value),
        });
    }
    let scores: Vec<_> = reference_scores(conn, now)?
        .into_iter()
        .filter(|s| s.uid == uid)
        .collect();
    let references = views_by_item(&scores, &stacks)
        .remove(&uid)
        .unwrap_or_default();
    Ok(ItemCurve {
        points,
        drivers: drivers(&events, &ctx, now),
        references,
    })
}
