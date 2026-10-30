#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The calendar and board screens, headless, including the accessibility audit.

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_board::edit::add_card;
use studyfrog_board::model::{MoveMode, board_default, card_new};
use studyfrog_calendar::session::{Ends, Repeat, Session};
use studyfrog_core::civil::{LocalTime, date_new};
use studyfrog_core::uid::Uid;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::kanban::BoardMsg;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::planner::PlanMsg;
use studyfrog_ui::state::Screen;

fn session(id: u128, title: &str, repeat: Repeat) -> Session {
    Session {
        id: Uid(id),
        title: title.into(),
        stack: Some(Uid(1)),
        start: LocalTime {
            date: date_new(2026, 10, 12).unwrap(),
            minute: 9 * 60,
        },
        duration_min: 30,
        repeat,
        ends: Ends::Forever,
        cancelled: Vec::new(),
        notes: String::new(),
        kind: Default::default(),
        reminders: Vec::new(),
    }
}

fn calendar_model() -> Model {
    let mut m = open_model();
    m.state.screen = Screen::Calendar;
    let today = date_new(2026, 10, 12).unwrap();
    let sessions = vec![session(
        7,
        "French drills",
        Repeat::Weekly { every: 1, days: 0 },
    )];
    apply(&mut m, Msg::Plan(PlanMsg::Loaded(Ok((sessions, today)))));
    m
}

fn board_model() -> Model {
    let mut m = open_model();
    m.state.screen = Screen::Board;
    let ids = [Uid(21), Uid(22), Uid(23), Uid(24)];
    let mut b = board_default(ids);
    for (i, t) in ["Read", "Write", "Review notes"].into_iter().enumerate() {
        b = add_card(
            &b,
            ids[i.min(1)],
            card_new(Uid(100 + i as u128), t),
            MoveMode::Force,
        )
        .unwrap();
    }
    apply(&mut m, Msg::Board(BoardMsg::Loaded(Ok(b))));
    m
}

#[test]
fn the_month_grid_marks_days_with_sessions_including_repeats() {
    let mut h = harness(calendar_model());
    h.run();
    assert!(h.query_by_label("October 2026").is_some());
    assert!(h.query_by_label("12 · 1").is_some(), "the first occurrence");
    assert!(h.query_by_label("19 · 1").is_some(), "the weekly repeat");
    assert!(
        h.query_by_label("13").is_some(),
        "a day without sessions has no count"
    );
    assert!(h.query_by_label("Sessions on 2026-10-12").is_some());
    assert!(h.query_by_label("Edit French drills").is_some());
}

#[test]
fn month_buttons_and_day_selection_drive_the_state() {
    let mut h = harness(calendar_model());
    h.run();
    h.get_by_label("Next month").click();
    h.run();
    assert_eq!(h.state().0.state.planner.month, (2026, 11));
    h.get_by_label("Today").click();
    h.run();
    assert_eq!(h.state().0.state.planner.month, (2026, 10));
    h.get_by_label("19 · 1").click();
    h.run();
    assert!(h.query_by_label("Sessions on 2026-10-19").is_some());
}

#[test]
fn the_day_panel_offers_edit_delete_skip_and_rehearse() {
    let mut h = harness(calendar_model());
    h.run();
    h.get_by_label("Skip French drills today").click();
    h.run();
    assert!(h.state().1.contains(&Effect::CancelOccurrence(
        Uid(7),
        date_new(2026, 10, 12).unwrap()
    )));
    h.get_by_label("Delete French drills").click();
    h.run();
    assert!(h.state().1.contains(&Effect::DeleteSession(Uid(7))));
    h.get_by_label("Start rehearsal for French drills").click();
    h.run();
    assert_eq!(h.state().0.state.screen, Screen::Rehearse);
}

#[test]
fn the_session_form_sends_a_session_to_be_saved() {
    let mut h = harness(calendar_model());
    h.run();
    h.get_by_label("Add to calendar").click();
    h.run();
    h.get_by_label("Title").focus();
    h.get_by_label("Title").type_text("Maths");
    h.run();
    h.get_by_label("Weekly").click();
    h.run();
    assert!(h.query_by_label("Repeat every (interval)").is_some());
    h.get_by_label("Save entry").click();
    h.run();
    let saved = h.state().1.iter().find_map(|e| match e {
        Effect::SaveSession(s) => Some(s.clone()),
        _ => None,
    });
    let s = saved.expect("a SaveSession effect");
    assert_eq!((s.title.as_str(), s.id), ("Maths", Uid(0)));
    assert!(matches!(s.repeat, Repeat::Weekly { every: 1, .. }));
}

#[test]
fn the_board_shows_wip_counts_and_flags_a_column_over_its_limit() {
    let mut h = harness(board_model());
    h.run();
    assert!(h.query_by_label("Backlog 1").is_some());
    assert!(h.query_by_label("Doing 2/3").is_some());
    assert!(h.query_by_label("Over the WIP limit").is_none());

    let mut m = board_model();
    let b = m.state.kanban.board.as_mut().unwrap();
    b.columns[1].wip_limit = Some(1);
    let mut h = harness(m);
    h.run();
    assert!(h.query_by_label("Doing 2/1").is_some());
    assert!(h.query_by_label("Over the WIP limit").is_some());
}

#[test]
fn board_buttons_send_the_same_commands_a_drag_would() {
    let mut h = harness(board_model());
    h.run();
    h.get_by_label("Push Read").click();
    h.run();
    h.get_by_label("Move Read down").click();
    h.run();
    h.get_by_label("Delete Write").click();
    h.run();
    h.get_by_label("Edit Write").click();
    h.run();
    let effects = &h.state().1;
    assert!(effects.iter().any(|e| matches!(e, Effect::BoardCommand(_))));
    assert!(
        h.state().0.state.kanban.edit.is_some(),
        "Edit opens the card editor"
    );
    assert!(h.query_by_label("Card notes").is_some());
}

#[test]
fn both_new_screens_are_fully_named_for_assistive_technology() {
    let mut cal = calendar_model();
    apply(&mut cal, Msg::Plan(PlanMsg::New));
    let mut h = harness(cal);
    h.run();
    assert_all_named(&h, "calendar form");
    let mut plain = harness(calendar_model());
    plain.run();
    assert_all_named(&plain, "calendar");
    let mut board = board_model();
    apply(&mut board, Msg::Board(BoardMsg::Open(Uid(100))));
    let mut h = harness(board);
    h.run();
    assert_all_named(&h, "board with editor");
}
