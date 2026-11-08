#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The simulated forecast and the exam pacer from a fixture vault (STORY-0105).

mod analytics_common;

use analytics_common::*;
use studyfrog_calendar::session::{Ends, EventKind, Repeat, Session};
use studyfrog_core::civil::{LocalTime, date_from_days};
use studyfrog_core::review::Rating::Good;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_report::forecast_sim::load_total;
use studyfrog_service::analytics_plan::SIM_DAYS;
use studyfrog_service::planning::save_session_checked;
use studyfrog_service::reports::build_reports;

fn exam(id: u128, stack: u128, day: i64) -> Session {
    Session {
        id: Uid(id),
        title: "Exam".into(),
        stack: Some(Uid(stack)),
        start: LocalTime {
            date: date_from_days(day),
            minute: 540,
        },
        duration_min: 60,
        repeat: Repeat::Never,
        ends: Ends::Forever,
        cancelled: Vec::new(),
        notes: String::new(),
        kind: EventKind::Exam,
        reminders: Vec::new(),
    }
}

/// Chemistry has an exam in 20 days, two cards seen and four never; Biology has no exam.
fn vault_with_exam(label: &str) -> (Dir, studyfrog_service::session::VaultSession, i64) {
    let cards: Vec<Card> = (0..6)
        .map(|i| (10 + i, "chem", "Chem", "q\n???\na"))
        .chain([(30, "bio", "Bio", "q\n???\na")])
        .collect();
    let (d, mut s) = vault(
        label,
        &[("chem", 1, "Chemistry"), ("bio", 2, "Biology")],
        &cards,
    );
    rate_all(
        &s,
        &[
            (10, T0, Good),
            (11, T0, Good),
            (30, T0, Good),
            (10, T0 + 3 * DAY, Good),
        ],
    );
    let today = T0.div_euclid(DAY) + 5;
    save_session_checked(&mut s, exam(500, 1, today + 20)).unwrap();
    (d, s, today)
}

#[test]
fn the_forecast_simulates_a_year_with_stages_band_and_minutes() {
    let (_d, s, today) = vault_with_exam("plan-forecast");
    let r = build_reports(&s, UnixMs(today * DAY + 3_600_000), 0).unwrap();
    let plan = &r.analytics.plan;
    assert_eq!(plan.forecast.len(), SIM_DAYS);
    let total: f32 = plan.forecast.iter().map(load_total).sum();
    assert!(total > 3.0, "the three seen cards come back: {total}");
    assert!(
        plan.forecast
            .iter()
            .all(|d| d.p10 <= d.p50 && d.p50 <= d.p90)
    );
    let minutes: f32 = plan.forecast.iter().map(|d| d.minutes).sum();
    assert!(minutes > 0.0, "reviews take time");
    assert!(
        plan.new_per_day > 0.0,
        "three cards started in the last two weeks"
    );
    assert!(plan.mix.is_some());
}

#[test]
fn a_stack_with_an_exam_ahead_gets_a_pacer_and_one_without_does_not() {
    let (_d, s, today) = vault_with_exam("plan-pacer");
    let r = build_reports(&s, UnixMs(today * DAY + 3_600_000), 0).unwrap();
    let pacers = &r.analytics.plan.pacers;
    assert_eq!(pacers.len(), 1);
    let p = &pacers[0];
    assert_eq!((p.folder.as_str(), p.exam_day), ("chem", today + 20));
    assert_eq!((p.pace.days_left, p.pace.study_days), (20, 17));
    assert_eq!((p.pace.unseen_items, p.pace.required_new_per_day), (4, 1));
    assert!(p.pace.on_track, "{p:?}");
}
