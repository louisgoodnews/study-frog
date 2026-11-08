#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! True retention and forgetting curves from a real vault's history (STORY-0107).

mod analytics_common;

use analytics_common::*;
use studyfrog_core::review::Rating::{Again, Good};
use studyfrog_core::time::UnixMs;
use studyfrog_report::retention_survival::rate_of;
use studyfrog_service::reports::build_reports;

#[test]
fn a_vault_history_is_replayed_into_true_retention_and_tier_curves() {
    let (_d, s) = vault(
        "surv",
        &[],
        &[(10, "", "A", "q\n???\na"), (11, "", "B", "q\n???\na")],
    );
    // A: due-day reviews until mature; B: reviewed far too late and forgotten.
    let mut ratings = vec![(10, T0, Good), (11, T0, Good), (11, T0 + 40 * DAY, Again)];
    let mut day = 0;
    for gap in [3, 10, 30, 60, 150] {
        day += gap;
        ratings.push((10, T0 + day * DAY, Good));
    }
    ratings.sort_by_key(|r| r.1);
    rate_all(&s, &ratings);
    let r = build_reports(&s, UnixMs(T0 + 400 * DAY), 0).unwrap();
    let sv = &r.analytics.survival;
    assert_eq!(
        sv.young.reviews + sv.mature.reviews,
        6,
        "six spaced reviews"
    );
    assert!(
        sv.overdue.reviews >= 1,
        "B came 40 days after a first review"
    );
    assert!(sv.mature.reviews >= 1, "A matured");
    assert_eq!(rate_of(&sv.mature), Some(1.0));
    assert_eq!(sv.tiers.iter().map(|t| t.total.reviews).sum::<u32>(), 6);
    assert!(sv.model_half_life_days.unwrap() > 0.0);
}

#[test]
fn an_empty_vault_has_an_empty_survival_report() {
    let (_d, s) = vault("surv-empty", &[], &[]);
    let r = build_reports(&s, UnixMs(T0), 0).unwrap();
    assert_eq!(r.analytics.survival.true_retention.reviews, 0);
    assert_eq!(r.analytics.survival.half_life_days, None);
}
