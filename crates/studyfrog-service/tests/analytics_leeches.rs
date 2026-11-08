#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Leeches, confusion pairs and the contrastive drill on a fixture vault (STORY-0104).

mod analytics_common;

use analytics_common::*;
use studyfrog_core::review::Rating::{self, Again, Good};
use studyfrog_core::run::RunMode;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_report::leech::LeechReason;
use studyfrog_service::create::fresh_id_source;
use studyfrog_service::day::DayWindow;
use studyfrog_service::drill::start_drill;
use studyfrog_service::reports::build_reports;

fn chemistry(label: &str) -> (Dir, studyfrog_service::session::VaultSession) {
    let (d, s) = vault(
        label,
        &[("chem", 1, "Chemistry"), ("bio", 2, "Biology")],
        &[
            (
                10,
                "chem",
                "Ionic bond",
                "What is an ionic bond?\n???\nElectrons are transferred between atoms.",
            ),
            (
                11,
                "chem",
                "Covalent bond",
                "What is a covalent bond?\n???\nElectrons are shared between atoms.",
            ),
            (
                12,
                "chem",
                "Molar mass",
                "Molar mass of water?\n???\n18 g/mol",
            ),
            (13, "bio", "Cell", "What is a cell?\n???\nThe unit of life."),
        ],
    );
    let day = |n: i64| T0 + n * DAY;
    let mut ratings: Vec<(u128, i64, Rating)> = Vec::new();
    // 10 and 11 are forgotten on the same days; 12 is forgotten on other days; 13 is fine.
    for (n, r) in [
        (0, Good),
        (2, Again),
        (3, Good),
        (6, Again),
        (7, Good),
        (12, Again),
        (13, Again),
    ] {
        ratings.push((10, day(n), r));
        ratings.push((11, day(n) + 60_000, r));
    }
    for (n, r) in [(0, Good), (4, Again), (9, Again), (20, Good)] {
        ratings.push((12, day(n), r));
    }
    for n in [0, 3, 10, 30] {
        ratings.push((13, day(n), Good));
    }
    ratings.sort_by_key(|r| r.1);
    rate_all(&s, &ratings);
    (d, s)
}

#[test]
fn the_items_forgotten_most_are_leeches_and_the_look_alikes_a_pair() {
    let (_d, s) = chemistry("leech");
    let r = build_reports(&s, UnixMs(T0 + 40 * DAY), 0).unwrap();
    let a = &r.analytics;
    let ids: Vec<Uid> = a.leeches.iter().map(|l| l.uid).collect();
    assert!(
        ids.contains(&Uid(10)) && ids.contains(&Uid(11)),
        "{:?}",
        a.leeches
    );
    assert!(!ids.contains(&Uid(13)) && !ids.contains(&Uid(12)));
    let ten = a.leeches.iter().find(|l| l.uid == Uid(10)).unwrap();
    assert_eq!(
        (ten.leech.lapses, ten.leech.streak, ten.folder.as_str()),
        (4, 2, "chem")
    );
    assert_eq!(ten.leech.reasons[0], LeechReason::RepeatedLapses);
    assert_eq!(a.pairs.len(), 1, "{:?}", a.pairs);
    let p = &a.pairs[0];
    assert_eq!((p.a.0, p.b.0), (Uid(10), Uid(11)));
    assert_eq!(
        (p.a.1.as_str(), p.pair.co_lapses, p.pair.same_group),
        ("Ionic bond", 4, true)
    );
    assert!(p.pair.similarity >= 0.3, "{}", p.pair.similarity);
}

#[test]
fn a_drill_is_a_practice_run_of_exactly_the_items_given() {
    let (_d, s) = chemistry("drill");
    let now = UnixMs(T0 + 40 * DAY);
    let window = DayWindow {
        start: UnixMs(now.0 - now.0 % DAY),
        end: UnixMs(now.0 - now.0 % DAY + DAY),
    };
    let ids = fresh_id_source(1).unwrap();
    let run = start_drill(
        &s,
        &[Uid(11), Uid(10), Uid(999)],
        RunMode::Flip,
        (now, window),
        ids,
    )
    .unwrap();
    assert_eq!(
        run.queue,
        [Uid(11), Uid(10)],
        "in the order given, unknown items left out"
    );
    assert_eq!(
        run.plan.effect,
        studyfrog_core::plan::ScheduleEffect::PracticeOnly
    );
    assert!(run.plan.requeue_again);
    let stored: u32 = s
        .conn
        .query_row("SELECT COUNT(*) FROM run_queue", [], |r| r.get(0))
        .unwrap();
    assert_eq!(stored, 2);
    let none = start_drill(
        &s,
        &[Uid(999)],
        RunMode::Flip,
        (now, window),
        fresh_id_source(2).unwrap(),
    );
    assert!(none.is_err());
}
