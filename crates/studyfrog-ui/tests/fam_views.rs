#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Familiarity on the item page, read through the real worker.

mod common;

use common::flow::{opened, send};
use common::views::{assert_all_named, harness};
use egui_kittest::kittest::Queryable;
use studyfrog_core::review::{Rating, Review};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_store::schedule::insert_review_timed;
use studyfrog_ui::msg::Msg;

const NOW_MS: i64 = common::NOW_NS / 1_000_000;
const DAY: i64 = 86_400_000;

#[test]
fn the_item_page_shows_the_value_its_drivers_and_the_curve() {
    let (_vault, mut model, mut worker) = opened("fam-item");
    let conn = &worker.session.as_ref().unwrap().conn;
    for d in [20, 10, 2] {
        let r = Review {
            item: Uid(10),
            rating: Rating::Good,
            at: UnixMs(NOW_MS - d * DAY),
            elapsed_days: 1.0,
        };
        insert_review_timed(conn, &r, None).unwrap();
    }
    send(&mut model, &mut worker, Msg::OpenItem(Uid(10)));
    let curve = model
        .state
        .editor
        .as_ref()
        .unwrap()
        .curve
        .clone()
        .expect("curve loaded");
    assert_eq!(curve.points.len(), 4, "three ratings and now");
    let mut h = harness(model);
    h.run_steps(4);
    assert!(h.query_by_label_contains("Familiarity ").is_some());
    assert!(h.query_by_label_contains("3 rating(s)").is_some());
    assert!(h.query_by_label_contains("Over time:").is_some());
    assert_all_named(&h, "item page with familiarity");

    let (_v2, mut m2, mut w2) = opened("fam-none");
    send(&mut m2, &mut w2, Msg::OpenItem(Uid(11)));
    let mut h = harness(m2);
    h.run_steps(4);
    assert!(
        h.query_by_label("Over time:").is_none()
            && h.query_by_label_contains("Over time").is_none()
    );
}
