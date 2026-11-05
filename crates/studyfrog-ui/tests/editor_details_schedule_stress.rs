#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Stress tests for details notebook schedule tab interactions and countdown formatting.

mod common;

use common::views::{harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_calendar::session::{EventKind, Session};
use studyfrog_core::civil::{Date, add_days, date_new};
use studyfrog_core::uid::Uid;
use studyfrog_md::parse::parse_doc;
use studyfrog_service::document::LoadedDoc;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::plan_msg::PlanMsg;

const STACK_RAW: &str = "\
---
id: 00000000000000000000000001
kind: stack
title: Math Stack
---
# Math
";

fn stack_with_deadline(due: Date) -> Model {
    let mut m = open_model();
    let doc = parse_doc(STACK_RAW).unwrap();
    let ld = LoadedDoc {
        path: "math/_stack.md".into(),
        doc,
        hash: 1,
    };
    apply(&mut m, Msg::DocumentLoaded(Uid(1), Ok(ld)));
    let session = Session {
        id: Uid(100),
        title: "Due: Math Stack".into(),
        stack: Some(Uid(1)),
        start: studyfrog_core::civil::LocalTime {
            date: due,
            minute: 540,
        },
        duration_min: 15,
        repeat: studyfrog_calendar::session::Repeat::Never,
        ends: studyfrog_calendar::session::Ends::Forever,
        cancelled: Vec::new(),
        notes: String::new(),
        kind: EventKind::Deadline,
        reminders: vec![1440],
    };
    m.state.planner.sessions = vec![session];
    m.state.planner.loaded = true;
    m
}

#[test]
fn countdown_pill_matrix_formatting() {
    let today = date_new(2026, 6, 15).unwrap();
    let cases = [
        (today, "Due today"),
        (add_days(today, 1), "Due in 1 day"),
        (add_days(today, 5), "Due in 5 days"),
        (add_days(today, -1), "Overdue by 1 day"),
        (add_days(today, -7), "Overdue by 7 days"),
    ];

    for (target, expected_pill) in cases {
        let mut m = stack_with_deadline(target);
        m.state.planner.today = today;
        let mut h = harness(m);
        h.run_steps(2);
        h.get_by_label("Schedule").click();
        h.run_steps(2);
        assert!(
            h.query_by_label(expected_pill).is_some(),
            "expected countdown pill '{expected_pill}' for target {target:?} and today {today:?}"
        );
    }
}

#[test]
fn schedule_reminders_and_clear_mutations() {
    let today = date_new(2026, 6, 15).unwrap();
    let m = stack_with_deadline(add_days(today, 3));
    let mut h = harness(m);
    h.run_steps(2);
    h.get_by_label("Schedule").click();
    h.run_steps(2);

    assert!(h.query_by_label("Remind me:").is_some());
    h.get_by_label("1 hour before").click();
    h.run_steps(2);

    let save_eff = h
        .state()
        .1
        .iter()
        .find_map(|e| match e {
            Effect::SaveSession(s) => Some(s.clone()),
            _ => None,
        })
        .expect("SaveSession effect expected after clicking reminder");

    assert!(save_eff.reminders.contains(&60));
    assert!(save_eff.reminders.contains(&1440));

    apply(
        &mut h.state_mut().0,
        Msg::Plan(PlanMsg::Changed(Ok(vec![save_eff]))),
    );
    h.state_mut().1.clear();
    h.run_steps(2);

    h.get_by_label("1 day before").click();
    h.run_steps(2);

    let save_eff2 = h
        .state()
        .1
        .iter()
        .find_map(|e| match e {
            Effect::SaveSession(s) => Some(s.clone()),
            _ => None,
        })
        .expect("SaveSession effect expected after removing reminder");

    assert!(!save_eff2.reminders.contains(&1440));
    assert!(save_eff2.reminders.contains(&60));

    h.get_by_label("Clear due date").click();
    h.run_steps(2);

    let del_eff = h
        .state()
        .1
        .iter()
        .find_map(|e| match e {
            Effect::DeleteSession(id) => Some(*id),
            _ => None,
        })
        .expect("DeleteSession effect expected after clear");
    assert_eq!(del_eff, Uid(100));
}
