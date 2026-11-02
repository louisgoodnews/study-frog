#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The reviews the schedule planned, on the calendar: reading, dragging, moving by date, undo and the setting.

mod common;

use common::flow::{opened, send};
use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_calendar::review_plan::PlanEntry;
use studyfrog_core::civil::{date_new, date_to_days};
use studyfrog_core::memory::MemState;
use studyfrog_core::uid::Uid;
use studyfrog_service::review_plan::{Moved, Prior};
use studyfrog_store::due_dates::Dues;
use studyfrog_store::schedule::{StoredMem, load_mem_state, save_mem_state};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::planner::PlanMsg;
use studyfrog_ui::planner_reviews::{ReviewEffect, ReviewMsg};
use studyfrog_ui::state::{Screen, Selection};

fn review(m: ReviewMsg) -> Msg {
    Msg::Plan(PlanMsg::Reviews(m))
}

fn today() -> studyfrog_core::civil::Date {
    date_new(2026, 10, 12).unwrap()
}

fn entry(day_offset: i64, n: u128) -> PlanEntry {
    PlanEntry {
        stack: Some(Uid(1)),
        day: date_to_days(today()) + day_offset,
        items: (0..n).map(|i| Uid(10 + i)).collect(),
        stack_size: 10,
    }
}

fn model() -> Model {
    let mut m = open_model();
    m.state.screen = Screen::Calendar;
    apply(
        &mut m,
        Msg::Plan(PlanMsg::Loaded(Ok((Vec::new(), today())))),
    );
    apply(&mut m, review(ReviewMsg::Loaded(Ok(vec![entry(3, 4)]))));
    m
}

#[test]
fn the_calendar_asks_for_the_grid_and_the_setting_turns_it_off() {
    let mut m = open_model();
    let fx = apply(
        &mut m,
        Msg::Plan(PlanMsg::Loaded(Ok((Vec::new(), today())))),
    );
    let from = date_to_days(today());
    let Some(Effect::Reviews(ReviewEffect::Load(a, b))) = fx
        .iter()
        .find(|e| matches!(e, Effect::Reviews(ReviewEffect::Load(..))))
    else {
        panic!("{fx:?}")
    };
    assert_eq!(
        *a, from,
        "overdue reviews show on today, so the past is not asked for"
    );
    assert!(*b > from + 20);
    let fx = apply(&mut m, Msg::Plan(PlanMsg::Month(1)));
    assert!(
        fx.iter()
            .any(|e| matches!(e, Effect::Reviews(ReviewEffect::Load(..))))
    );

    apply(&mut m, review(ReviewMsg::Loaded(Ok(vec![entry(3, 4)]))));
    let fx = apply(&mut m, review(ReviewMsg::Show(false)));
    assert!(
        fx.iter()
            .any(|e| matches!(e, Effect::SaveAppSettings(s) if s.calendar_reviews_off))
    );
    assert!(m.state.planner.reviews.is_empty());
    assert!(
        !apply(&mut m, Msg::Plan(PlanMsg::Today))
            .iter()
            .any(|e| matches!(e, Effect::Reviews(ReviewEffect::Load(..))))
    );
}

#[test]
fn dropping_an_entry_moves_its_items_and_the_past_is_refused() {
    let mut m = model();
    let day = date_to_days(today());
    let drop = |to| {
        review(ReviewMsg::Drop {
            stack: Some(Uid(1)),
            from: day + 3,
            to,
        })
    };
    let fx = apply(&mut m, drop(day + 5));
    assert_eq!(
        fx,
        [Effect::Reviews(ReviewEffect::Move(
            (10..14).map(Uid).collect(),
            day + 5
        ))]
    );
    assert!(
        apply(&mut m, drop(day + 3)).is_empty(),
        "the same day is nothing to do"
    );
    assert!(apply(&mut m, drop(day - 1)).is_empty());
    assert!(m.state.notice.as_ref().unwrap().text.contains("later day"));
}

#[test]
fn a_typed_date_moves_a_result_shows_its_warning_and_undo_goes_back() {
    let mut m = model();
    let day = date_to_days(today());
    apply(&mut m, review(ReviewMsg::OpenMove(Some(Uid(1)), day + 3)));
    assert_eq!(m.state.planner.moving.as_ref().unwrap().text, "2026-10-15");
    apply(&mut m, review(ReviewMsg::MoveText("soon".into())));
    assert!(apply(&mut m, review(ReviewMsg::Confirm)).is_empty());
    apply(&mut m, review(ReviewMsg::MoveText("2026-10-20".into())));
    let fx = apply(&mut m, review(ReviewMsg::Confirm));
    assert_eq!(
        fx,
        [Effect::Reviews(ReviewEffect::Move(
            (10..14).map(Uid).collect(),
            day + 8
        ))]
    );

    let prior = Prior {
        item: Uid(10),
        dues: Dues {
            fsrs: Some(5),
            sm2: None,
        },
    };
    let done = Moved {
        moved: 4,
        undo: vec![prior],
        warning: Some("That is after the exam or due date of French.".into()),
    };
    let fx = apply(&mut m, review(ReviewMsg::Moved(Ok(Box::new(done)))));
    assert!(
        fx.iter()
            .any(|e| matches!(e, Effect::Reviews(ReviewEffect::Load(..))))
    );
    let text = m.state.notice.as_ref().unwrap().text.clone();
    assert!(
        text.contains("4 item") && text.contains("after the exam"),
        "{text}"
    );
    assert!(m.state.planner.moving.is_none() && m.state.planner.undo.is_some());
    let fx = apply(&mut m, review(ReviewMsg::Undo));
    assert_eq!(fx, [Effect::Reviews(ReviewEffect::Undo(vec![prior]))]);
    assert!(m.state.planner.undo.is_none());
}

#[test]
fn an_entry_is_told_apart_opens_its_stack_and_offers_the_move_without_dragging() {
    let mut m = model();
    apply(
        &mut m,
        Msg::Plan(PlanMsg::Select(date_new(2026, 10, 15).unwrap())),
    );
    let mut h = harness(m);
    h.run_steps(3);
    assert_all_named(&h, "calendar with reviews");
    let name = "Planned reviews, French: 4 of 10 due";
    assert!(h.query_by_label(name).is_some());
    h.get_by_label("Move French reviews…").click();
    h.run_steps(3);
    assert!(h.state().0.state.planner.moving.is_some());
    assert!(
        h.query_by_label("Move reviews to").is_some() && h.query_by_label("Cancel move").is_some()
    );
    h.get_by_label("Cancel move").click();
    h.run_steps(3);
    assert!(h.state().0.state.planner.moving.is_none());
    h.get_by_label(name).click();
    h.run_steps(3);
    let s = &h.state().0.state;
    assert_eq!(s.screen, Screen::Library);
    assert_eq!(s.browser.selection, Selection::Stack("french".into()));
}

#[test]
fn through_the_worker_a_drop_changes_the_due_date_only_and_undo_restores_it() {
    let (_vault, mut model, mut worker) = opened("review-move");
    let now_ms = common::NOW_NS / 1_000_000;
    let mem = |due: i64| StoredMem {
        state: MemState {
            stability: 9.0,
            difficulty: 4.0,
        },
        due: studyfrog_core::time::UnixMs(due),
        last_review: studyfrog_core::time::UnixMs(now_ms - 86_400_000),
    };
    let conn = &worker.session.as_ref().unwrap().conn;
    for id in [10, 11] {
        save_mem_state(conn, Uid(id), &mem(now_ms + 3 * 86_400_000)).unwrap();
    }
    send(&mut model, &mut worker, Msg::Navigate(Screen::Calendar));
    let p = &model.state.planner;
    let entry = p
        .reviews
        .iter()
        .find(|e| e.stack == Some(Uid(1)))
        .expect("an entry for French");
    assert_eq!((entry.items.len(), entry.stack_size), (2, 3));
    let (from, to) = (entry.day, entry.day + 4);

    send(
        &mut model,
        &mut worker,
        review(ReviewMsg::Drop {
            stack: Some(Uid(1)),
            from,
            to,
        }),
    );
    let conn = &worker.session.as_ref().unwrap().conn;
    let moved = load_mem_state(conn, Uid(10)).unwrap().unwrap();
    assert_eq!(
        moved.state,
        mem(0).state,
        "stability and difficulty are untouched"
    );
    assert!(moved.due.0 > now_ms + 6 * 86_400_000);
    assert!(
        model
            .state
            .planner
            .reviews
            .iter()
            .any(|e| e.day == to && e.items.len() == 2)
    );
    assert!(model.state.planner.undo.is_some());

    send(&mut model, &mut worker, review(ReviewMsg::Undo));
    let back = load_mem_state(&worker.session.as_ref().unwrap().conn, Uid(10))
        .unwrap()
        .unwrap();
    assert_eq!(back.due.0, now_ms + 3 * 86_400_000);
    assert!(model.state.planner.reviews.iter().any(|e| e.day == from));
}
