#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Workload against desired retention from a fixture vault, and applying the recommendation (STORY-0106).

mod analytics_common;

use analytics_common::*;
use studyfrog_core::review::Rating::{Again, Good};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::analytics_cmrr::apply_retention;
use studyfrog_service::reports::build_reports;
use studyfrog_store::stack_scope::stack_homes;

fn deck(label: &str) -> (Dir, studyfrog_service::session::VaultSession) {
    let cards: Vec<Card> = (0..8)
        .map(|i| (10 + i, "chem", "C", "q\n???\na"))
        .chain((0..4).map(|i| (40 + i, "bio", "B", "q\n???\na")))
        .collect();
    let (d, s) = vault(
        label,
        &[("chem", 1, "Chemistry"), ("bio", 2, "Biology")],
        &cards,
    );
    let mut ratings = Vec::new();
    for i in 0..8 {
        ratings.push((10 + i, T0, Good));
        ratings.push((10 + i, T0 + 4 * DAY, if i % 3 == 0 { Again } else { Good }));
    }
    ratings.push((40, T0, Good));
    ratings.sort_by_key(|r| r.1);
    rate_all(&s, &ratings);
    (d, s)
}

#[test]
fn the_vault_and_each_stack_get_a_curve_over_the_twelve_targets() {
    let (_d, s) = deck("cmrr-curves");
    let r = build_reports(&s, UnixMs(T0 + 10 * DAY), 0).unwrap();
    let c = &r.analytics.plan.cmrr;
    assert_eq!(c.vault.points.len(), 12);
    assert!((c.current - 0.9).abs() < 1e-6, "the vault default");
    let best = c.vault.recommended.unwrap();
    assert!(best >= c.vault.min_workload_retention.unwrap());
    let folders: Vec<&str> = c.stacks.iter().map(|x| x.folder.as_str()).collect();
    assert_eq!(folders, ["bio", "chem"]);
    assert!(c.stacks.iter().all(|x| x.report.points.len() == 12));
}

#[test]
fn applying_writes_the_stack_or_vault_file_and_keeps_the_rest() {
    let (d, mut s) = deck("cmrr-apply");
    std::fs::write(
        d.0.join("chem/_stack.toml"),
        "# chemistry\ndaily_new_limit = 7\n",
    )
    .unwrap();
    std::fs::write(
        d.0.join("studyfrog.toml"),
        "# vault\n[srs]\nmaximum_interval_days = 400\n",
    )
    .unwrap();
    let info = apply_retention(&mut s, Some("chem"), 0.86).unwrap();
    let text = std::fs::read_to_string(d.0.join("chem/_stack.toml")).unwrap();
    assert_eq!(
        text,
        "# chemistry\ndaily_new_limit = 7\nrequest_retention = 0.86\n"
    );
    let homes = stack_homes(&s.conn).unwrap();
    assert_eq!(
        homes[&Uid(1)].request_retention,
        Some(0.86),
        "the index read it again"
    );
    assert!(info.settings_error.is_none());
    let info = apply_retention(&mut s, None, 0.88).unwrap();
    assert!((info.settings.request_retention - 0.88).abs() < 1e-6);
    assert_eq!(info.settings.maximum_interval_days, 400);
    let r = build_reports(&s, UnixMs(T0 + 10 * DAY), 0).unwrap();
    let chem = r
        .analytics
        .plan
        .cmrr
        .stacks
        .iter()
        .find(|x| x.folder == "chem")
        .unwrap();
    assert!((chem.current - 0.86).abs() < 1e-6);
    assert!(apply_retention(&mut s, None, 0.5).is_err(), "out of range");
}
