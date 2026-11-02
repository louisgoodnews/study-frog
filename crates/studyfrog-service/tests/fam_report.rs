#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The familiarity report: spread, least familiar per stack and subject, drivers and the curve of one item.

use std::fs;

use studyfrog_core::review::{Rating, Review};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::fam_curve::item_curve;
use studyfrog_service::fam_report::familiarity_report;
use studyfrog_service::session::open_vault;
use studyfrog_store::schedule::insert_review_timed;

const DAY: i64 = 86_400_000;
const NOW: i64 = 1_800_000_000_000;

#[test]
fn the_report_ranks_the_least_familiar_with_their_drivers_and_the_curve_rises_with_good_ratings() {
    let dir = std::env::temp_dir().join(format!("studyfrog-famrep-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("french")).unwrap();
    fs::write(dir.join("french/_stack.md"), "---\nid: 00000000000000000000000001\nkind: stack\ntitle: French\nsubjects: [Languages]\n---\n").unwrap();
    fs::write(
        dir.join("subjects.toml"),
        "[[subject]]\nname = \"Languages\"\n",
    )
    .unwrap();
    for (id, name) in [('A', "weak"), ('B', "strong"), ('C', "unseen")] {
        let text = format!(
            "---\nid: 0000000000000000000000000{id}\nkind: card\ntitle: {name}\n---\nq\n???\na\n"
        );
        fs::write(dir.join(format!("french/{name}.md")), text).unwrap();
    }
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    let rate = |item: u128, rating, days_ago: i64| {
        let r = Review {
            item: Uid(item),
            rating,
            at: UnixMs(NOW - days_ago * DAY),
            elapsed_days: 1.0,
        };
        insert_review_timed(&s.conn, &r, None).unwrap();
    };
    rate(0xA, Rating::Again, 3);
    for d in [40, 30, 20, 10, 1] {
        rate(0xB, Rating::Easy, d);
    }
    let report = familiarity_report(&s.conn, UnixMs(NOW)).unwrap();
    assert_eq!(report.unscored, 1, "the unseen card has no value");
    assert_eq!(report.spread.iter().sum::<u32>(), 2);
    assert_eq!(report.spread[0], 1, "the lapse is in the lowest tenth");
    let french = report.stacks.iter().find(|g| g.name == "French").unwrap();
    assert_eq!((french.items, french.least[0].title.as_str()), (3, "weak"));
    assert!(
        french.least[0]
            .drivers
            .contains("1 rating(s), the last again 3 day(s) ago"),
        "{}",
        french.least[0].drivers
    );
    assert!(french.percent.unwrap() > 0);
    let subject = report
        .subjects
        .iter()
        .find(|g| g.name == "Languages")
        .unwrap();
    assert_eq!(subject.items, 3);

    let curve = item_curve(&s.conn, Uid(0xB), UnixMs(NOW)).unwrap();
    let values: Vec<u8> = curve.points.iter().map(|p| p.percent).collect();
    assert_eq!(values.len(), 6, "five ratings and now");
    assert!(
        values[..5].windows(2).all(|w| w[0] <= w[1]),
        "easy ratings only raise it: {values:?}"
    );
    assert!(curve.drivers.contains("5 rating(s)"));
    assert!(
        item_curve(&s.conn, Uid(0xC), UnixMs(NOW))
            .unwrap()
            .points
            .is_empty()
    );
    let _ = fs::remove_dir_all(&dir);
}
