#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Deleting a homework item (STORY-0167): the confirmation in the dialog and the outcome of the delete.
//! Pure update, no I/O.

use studyfrog_calendar::homework::{ChecklistItem, HomeworkItem, HomeworkPriority, HomeworkStatus};
use studyfrog_core::civil::{Date, LocalTime};
use studyfrog_core::uid::Uid;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::homework_msg::HomeworkEffect;
use studyfrog_ui::homework_msg::HomeworkMsg;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::State;
use studyfrog_ui::update::update;

fn day(d: u8) -> Date {
    Date {
        year: 2026,
        month: 11,
        day: d,
    }
}

fn clock() -> LocalTime {
    LocalTime {
        date: day(21),
        minute: 12 * 60,
    }
}

fn item(id: u128, due_day: u8) -> HomeworkItem {
    HomeworkItem {
        id: Uid(id),
        title: format!("Essay {id}"),
        course: None,
        start: None,
        due: LocalTime {
            date: day(due_day),
            minute: 18 * 60,
        },
        estimate_min: 90,
        priority: HomeworkPriority::High,
        status: HomeworkStatus::NotStarted,
        checklist: vec![ChecklistItem {
            text: "Read".into(),
            done: true,
        }],
        notes: String::new(),
    }
}

fn send(state: State, m: HomeworkMsg) -> (State, Vec<Effect>) {
    update(state, Msg::Homework(m))
}

fn loaded(items: Vec<HomeworkItem>) -> State {
    send(State::default(), HomeworkMsg::Changed(Ok((items, clock())))).0
}

#[test]
fn deleting_asks_first_and_only_a_confirmed_delete_reaches_the_worker() {
    let (s, _) = send(loaded(vec![item(7, 24)]), HomeworkMsg::EditItem(Uid(7)));
    let (s, fx) = send(s, HomeworkMsg::AskDelete);
    assert!(fx.is_empty(), "asking deletes nothing");
    assert!(s.planner.homework_ui.confirm_delete);
    let (s, fx) = send(s, HomeworkMsg::KeepItem);
    assert!(fx.is_empty());
    assert!(
        !s.planner.homework_ui.confirm_delete,
        "keeping clears the question"
    );
    let (s, _) = send(s, HomeworkMsg::AskDelete);
    let (s, fx) = send(s, HomeworkMsg::Delete(Uid(7)));
    assert_eq!(fx, vec![Effect::Homework(HomeworkEffect::Delete(Uid(7)))],);
    assert!(
        s.planner.homework_ui.form.is_some(),
        "the dialog stays until the store answers"
    );
}

#[test]
fn a_deleted_item_closes_the_dialog_and_refreshes_the_list() {
    let (s, _) = send(
        loaded(vec![item(7, 24), item(8, 25)]),
        HomeworkMsg::EditItem(Uid(7)),
    );
    let (s, _) = send(s, HomeworkMsg::AskDelete);
    let (s, _) = send(s, HomeworkMsg::Deleted(Ok((vec![item(8, 25)], clock()))));
    assert!(s.planner.homework_ui.form.is_none());
    assert!(!s.planner.homework_ui.confirm_delete);
    assert_eq!(s.planner.homework.len(), 1);
    assert_eq!(s.planner.homework[0].id, Uid(8));
}

#[test]
fn a_refused_delete_keeps_the_dialog_open_with_the_reason() {
    let (s, _) = send(loaded(vec![item(7, 24)]), HomeworkMsg::EditItem(Uid(7)));
    let (s, _) = send(s, HomeworkMsg::AskDelete);
    let (s, _) = send(
        s,
        HomeworkMsg::Deleted(Err("that homework item no longer exists".into())),
    );
    assert!(s.planner.homework_ui.form.is_some());
    assert!(!s.planner.homework_ui.confirm_delete);
    assert_eq!(
        s.planner.homework_ui.error.as_deref(),
        Some("that homework item no longer exists")
    );
}
