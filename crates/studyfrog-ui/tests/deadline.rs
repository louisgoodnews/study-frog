#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Setting and changing a stack's due date from the library.

mod common;

use common::views::{harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_calendar::session::{Ends, EventKind, Repeat, Session};
use studyfrog_core::civil::{LocalTime, date_new};
use studyfrog_core::uid::Uid;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::planner::PlanMsg;
use studyfrog_ui::state::{Screen, Selection};

fn model(with_deadline: bool) -> Model {
    let mut m = open_model();
    let stack = m.state.vault.as_ref().unwrap().overview.stacks[0].clone();
    apply(&mut m, Msg::Select(Selection::Stack(stack.folder)));
    let sessions = if with_deadline {
        vec![Session {
            id: Uid(77),
            title: "Due: French".into(),
            stack: Some(stack.uid),
            start: LocalTime {
                date: date_new(2027, 3, 1).unwrap(),
                minute: 540,
            },
            duration_min: 15,
            repeat: Repeat::Never,
            ends: Ends::Forever,
            cancelled: Vec::new(),
            notes: String::new(),
            kind: EventKind::Deadline,
            reminders: vec![1440],
        }]
    } else {
        Vec::new()
    };
    apply(
        &mut m,
        Msg::Plan(PlanMsg::Loaded(Ok((
            sessions,
            date_new(2026, 10, 30).unwrap(),
        )))),
    );
    m
}

#[test]
fn a_stack_without_a_due_date_offers_to_set_one() {
    let mut h = harness(model(false));
    h.run();
    assert!(h.query_by_label("No due date").is_some());
    h.get_by_label("Set due date").click();
    h.run();
    let s = &h.state().0.state;
    assert_eq!(s.screen, Screen::Calendar);
    let f = s.planner.form.as_ref().unwrap();
    assert_eq!(
        (f.kind, f.title.as_str(), f.date.as_str()),
        (EventKind::Deadline, "Due: French", "2026-11-06")
    );
    assert!(f.stack.is_some() && f.editing.is_none());
}

#[test]
fn a_stack_with_a_due_date_shows_it_and_opens_that_entry_to_change_it() {
    let mut h = harness(model(true));
    h.run();
    assert!(h.query_by_label("Due 2027-03-01").is_some());
    h.get_by_label("Change due date").click();
    h.run();
    let f = h.state().0.state.planner.form.clone().unwrap();
    assert_eq!((f.editing, f.date.as_str()), (Some(Uid(77)), "2027-03-01"));
    assert!(
        h.query_by_label("Pick date").is_some(),
        "the date picker is there"
    );
}
