//! Calibration report builder in the service layer.

use std::collections::HashMap;

use rusqlite::Connection;
use studyfrog_core::civil::date_from_days;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_report::calibration::{
    CalibrationReport, CalibrationReview, DiaryRatingPoint, calibration_report,
};
use studyfrog_store::error::StoreError;
use studyfrog_store::reports::HistoryRow;

use crate::fam_refs::reference_scores;

const MS_PER_DAY: i64 = 86_400_000;

fn day_of(ms: i64, offset_minutes: i32) -> i64 {
    (ms + i64::from(offset_minutes) * 60_000).div_euclid(MS_PER_DAY)
}

/// Compute the calibration report from stored diary entries and review history.
pub fn build_calibration_report(
    conn: &Connection,
    rows: &[HistoryRow],
    now: UnixMs,
    offset_minutes: i32,
) -> Result<CalibrationReport, StoreError> {
    let diary_entries = studyfrog_store::diary::list_all_diary_entries(conn)?;
    let diary_points: Vec<DiaryRatingPoint> = diary_entries
        .into_iter()
        .map(|e| DiaryRatingPoint {
            date: e.date,
            ratings: e.ratings,
        })
        .collect();

    let fam_scores = reference_scores(conn, now).unwrap_or_default();
    let mut fam_map: HashMap<Uid, f32> = HashMap::new();
    for s in fam_scores {
        if let Some(fam) = s.fam {
            fam_map.entry(s.uid).or_insert(fam.value);
        }
    }

    let cal_reviews: Vec<CalibrationReview> = rows
        .iter()
        .map(|r| CalibrationReview {
            date: date_from_days(day_of(r.at_ms, offset_minutes)),
            rating: r.rating,
            familiarity: fam_map.get(&r.item).copied(),
        })
        .collect();

    Ok(calibration_report(&diary_points, &cal_reviews))
}
