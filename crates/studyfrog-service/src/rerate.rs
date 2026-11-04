//! Changing the rating of an item's latest review after the fact, and rebuilding its schedule.
//!
//! The schedule is a pure fold over the review history, so replacing one rating and folding again
//! gives exactly the state a different rating would have produced at the time.

use rusqlite::Connection;
use studyfrog_core::review::{Rating, Review};
use studyfrog_core::scheduler::Scheduler;
use studyfrog_core::settings::VaultSettings;
use studyfrog_core::uid::Uid;
use studyfrog_srs::params::Params;
use studyfrog_store::error::{StoreError, sql_err};
use studyfrog_store::schedule::{
    StoredMem, StoredSm2, load_reviews, load_reviews_timed_in, save_mem_state_in,
    save_sm2_state_in, set_latest_review_rating,
};
use studyfrog_store::stack_scope::{review_stacks, stack_home};

use crate::schedule::{apply_rating, apply_rating_sm2};
use crate::session::{ServiceError, VaultSession, current_settings};
use crate::settings::{params_for, scheduler_for};

/// The FSRS state after replaying `reviews` from the start.
pub fn replay_fsrs(reviews: &[Review], params: &Params) -> Option<StoredMem> {
    reviews.iter().fold(None, |prev, r| {
        Some(apply_rating(prev.as_ref(), r.item, r.rating, r.at, params).1)
    })
}

/// The SM-2 state after replaying `reviews` from the start.
pub fn replay_sm2(reviews: &[Review], max_interval_days: u32) -> Option<StoredSm2> {
    reviews.iter().fold(None, |prev, r| {
        Some(apply_rating_sm2(prev.as_ref(), r.item, r.rating, r.at, max_interval_days).1)
    })
}

/// Fold the reviews each stack holds of the item again, with that stack's own scheduler, and store the result.
fn rebuild(conn: &Connection, item: Uid, vault: &VaultSettings) -> Result<(), StoreError> {
    for stack in review_stacks(conn, item)? {
        let reviews: Vec<Review> = load_reviews_timed_in(conn, (item, stack))?
            .into_iter()
            .map(|(r, _)| r)
            .collect();
        let home = stack_home(conn, stack)?;
        match scheduler_for(vault, home.as_ref()) {
            Scheduler::Fsrs => {
                let params = params_for(vault, home.and_then(|h| h.request_retention));
                replay_fsrs(&reviews, &params)
                    .map_or(Ok(()), |m| save_mem_state_in(conn, (item, stack), &m))?;
            }
            Scheduler::Sm2 => {
                replay_sm2(&reviews, vault.maximum_interval_days)
                    .map_or(Ok(()), |m| save_sm2_state_in(conn, (item, stack), &m))?;
            }
        }
    }
    Ok(())
}

/// Replace the rating of `item`'s latest review, which must currently be `expected`, with `to`,
/// and rebuild the item's schedule.
pub fn rerate_latest(
    session: &mut VaultSession,
    item: Uid,
    expected: Rating,
    to: Rating,
) -> Result<(), ServiceError> {
    let vault = current_settings(session);
    let tx = session
        .conn
        .transaction()
        .map_err(|e| ServiceError::Store(sql_err(e)))?;
    let mut reviews = load_reviews(&tx, item).map_err(ServiceError::Store)?;
    match reviews.last_mut() {
        Some(last) if last.rating == expected => last.rating = to,
        _ => {
            return Err(ServiceError::Invalid(
                "that review has changed since the run; nothing was changed".to_owned(),
            ));
        }
    }
    set_latest_review_rating(&tx, item, to).map_err(ServiceError::Store)?;
    rebuild(&tx, item, &vault).map_err(ServiceError::Store)?;
    tx.commit().map_err(|e| ServiceError::Store(sql_err(e)))
}
