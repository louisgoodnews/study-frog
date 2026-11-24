#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Classroom assignments, part 1 (EPIC-068, TASK-0294): the panel's tasks and status, the group thread,
//! the drafted answer and its Turn in, and the refusals. Work runs through the real worker and service with
//! the fake transport; no network and no real model.
use classroom_common::{
    chat_harness, classroom, group_task, online, open, solo_task, started, stored_turns,
};
use common::flow::send;
use egui_kittest::kittest::Queryable;
use studyfrog_core::classroom::Owner;
use studyfrog_core::classroom_status::{WorkStatus, submission_for, task_status};
use studyfrog_ui::classroom_types::ClassroomMsg;
use studyfrog_ui::msg::Msg;

mod classroom_common;
mod common;

#[test]
fn the_panel_lists_solo_and_group_tasks_with_status_badges_and_rosters() {
    let (_vault, mut m, mut w) = started();
    let _cfg = online(&mut w);
    let solo = solo_task(&mut m, &mut w);
    let group = group_task(&mut m, &mut w);
    open(&mut m, &mut w, solo);
    classroom(&mut m, ClassroomMsg::DraftEdit("Plants make sugar.".into()));
    send(&mut m, &mut w, Msg::Classroom(ClassroomMsg::TurnIn));
    send(&mut m, &mut w, Msg::Classroom(ClassroomMsg::AskGrade));
    assert!(
        m.state.classroom.error.is_none(),
        "{:?}",
        m.state.classroom.error
    );

    let s = m.state.classroom.session.as_ref().unwrap();
    assert_eq!(task_status(s, solo, Some(1)), WorkStatus::Graded);
    assert_eq!(task_status(s, group, Some(1)), WorkStatus::Assigned);

    let h = chat_harness(m);
    for label in [
        "Active",
        "Completed",
        "Solo task",
        "Group task",
        "Status: graded",
        "Status: assigned",
        "Assignee: You",
        "Group 1: You, Ada, Ben",
    ] {
        assert!(
            h.query_all_by_label(label).next().is_some(),
            "{label} is shown"
        );
    }
}

#[test]
fn the_group_thread_carries_the_users_message_and_the_teammates_answers() {
    let (_vault, mut m, mut w) = started();
    let _cfg = online(&mut w);
    let group = group_task(&mut m, &mut w);
    open(&mut m, &mut w, group);
    classroom(
        &mut m,
        ClassroomMsg::GroupInput("Let's split the work.".into()),
    );
    send(&mut m, &mut w, Msg::Classroom(ClassroomMsg::SendGroup));
    assert!(
        m.state.classroom.error.is_none(),
        "{:?}",
        m.state.classroom.error
    );
    assert_eq!(m.state.classroom.group_input, "", "the composer is cleared");
    assert_eq!(
        stored_turns(&m),
        [
            (1, "Let's split the work.".to_owned()),
            (2, "Plants turn light into sugar.".to_owned()),
            (3, "Plants turn light into sugar.".to_owned()),
        ],
        "the user's message, then Ada and Ben answer"
    );
}

#[test]
fn the_draft_is_kept_and_turn_in_stores_it_locked_for_the_user() {
    let (_vault, mut m, mut w) = started();
    let _cfg = online(&mut w);
    let solo = solo_task(&mut m, &mut w);
    open(&mut m, &mut w, solo);
    classroom(
        &mut m,
        ClassroomMsg::DraftEdit("  Plants make sugar.  ".into()),
    );
    assert_eq!(
        studyfrog_ui::classroom_work_ui::draft_of(&m.state.classroom.drafts, solo),
        "  Plants make sugar.  ",
        "the draft is kept while it is edited"
    );
    send(&mut m, &mut w, Msg::Classroom(ClassroomMsg::TurnIn));
    assert!(
        m.state.classroom.error.is_none(),
        "{:?}",
        m.state.classroom.error
    );
    let s = m.state.classroom.session.as_ref().unwrap();
    let sub = submission_for(s, solo, Owner::Participant(1)).expect("turned in");
    assert_eq!(sub.text, "Plants make sugar.", "trimmed and stored");
    assert_eq!(task_status(s, solo, Some(1)), WorkStatus::Submitted);

    send(&mut m, &mut w, Msg::Classroom(ClassroomMsg::TurnIn));
    assert_eq!(
        m.state.classroom.error.as_deref(),
        Some("This answer is already turned in."),
        "a second turn-in is refused and the answer stays"
    );
}

#[test]
fn turning_in_needs_no_model_and_works_without_a_network() {
    let (_vault, mut m, mut w) = started();
    let _cfg = online(&mut w);
    let solo = solo_task(&mut m, &mut w);
    open(&mut m, &mut w, solo);
    classroom(&mut m, ClassroomMsg::DraftEdit("Offline answer.".into()));
    w.transport = None;
    send(&mut m, &mut w, Msg::Classroom(ClassroomMsg::TurnIn));
    assert!(
        m.state.classroom.error.is_none(),
        "{:?}",
        m.state.classroom.error
    );
    let s = m.state.classroom.session.as_ref().unwrap();
    assert!(submission_for(s, solo, Owner::Participant(1)).is_some());
}

#[test]
fn a_blank_draft_or_a_task_outside_the_users_work_is_refused_before_any_effect() {
    let (_vault, mut m, mut w) = started();
    let _cfg = online(&mut w);
    let solo = solo_task(&mut m, &mut w);
    open(&mut m, &mut w, solo);
    classroom(&mut m, ClassroomMsg::DraftEdit("   ".into()));
    let fx = classroom(&mut m, ClassroomMsg::TurnIn);
    assert!(fx.is_empty());
    assert_eq!(
        m.state.classroom.error.as_deref(),
        Some("Write your answer before you turn it in.")
    );

    classroom(&mut m, ClassroomMsg::GroupInput("Hello group".into()));
    let fx = classroom(&mut m, ClassroomMsg::SendGroup);
    assert!(fx.is_empty(), "a solo task has no group thread");
    assert_eq!(
        m.state.classroom.error.as_deref(),
        Some("Only a member of a group task can write to its group.")
    );

    let fx = classroom(&mut m, ClassroomMsg::AskGrade);
    assert!(fx.is_empty());
    assert_eq!(
        m.state.classroom.error.as_deref(),
        Some("Turn in your answer before the teacher grades it.")
    );
}
