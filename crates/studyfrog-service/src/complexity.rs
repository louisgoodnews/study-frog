//! Service helper computing the user's retained-complexity baseline from the open vault (EPIC-077).

use studyfrog_core::time::UnixMs;
use studyfrog_report::complexity_baseline::{
    ItemRetentionSample, RetentionComplexityProfile, estimate_complexity_baseline,
    score_text_complexity,
};

use crate::library_rows::load_library_rows;
use crate::session::{ServiceError, VaultSession};

/// Compute the user's personalized retained-complexity profile from reviewed vault items.
pub fn load_retention_complexity_profile(
    session: &VaultSession,
    now: UnixMs,
) -> Result<Option<RetentionComplexityProfile>, ServiceError> {
    let rows = load_library_rows(session, now)?;
    let samples: Vec<ItemRetentionSample> = rows
        .into_iter()
        .filter(|it| it.row.reviews > 0)
        .map(|it| {
            let scored = score_text_complexity(&it.row.body, "");
            ItemRetentionSample {
                word_count: scored.word_count,
                list_items: scored.list_items,
                familiarity: it.row.familiarity.unwrap_or(0.5),
                reviews: it.row.reviews,
            }
        })
        .collect();
    Ok(estimate_complexity_baseline(&samples))
}
