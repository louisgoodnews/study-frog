#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The homework commands of the control protocol (STORY-0190): words, policy, plan and answer.

use serde_json::json;
use studyfrog_calendar::homework::HomeworkPriority;
use studyfrog_calendar::homework_query::HomeworkTab;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_ui::control_cmd::{Class, Command, HomeworkCmd, HomeworkNew, class_of};
use studyfrog_ui::control_parse::parse_words;
use studyfrog_ui::control_plan::plan_for;
use studyfrog_ui::control_reply::reply_line;
use studyfrog_ui::homework_msg::HomeworkMsg;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::State;

fn id() -> String {
    format_ulid(Uid(7))
}

#[test]
fn a_new_item_takes_its_fields_as_flags() {
    let line = r#"homework new "Essay 2" --due 2026-12-02 --time 09:00 --estimate 90 --priority high --stack french --checklist "Read; Draft ;""#;
    let Command::Homework(HomeworkCmd::New(n)) = parse_words(line).unwrap() else {
        panic!("expected a new homework item");
    };
    assert_eq!(n.title, "Essay 2");
    assert_eq!(n.due_date.as_deref(), Some("2026-12-02"));
    assert_eq!(n.due_time.as_deref(), Some("09:00"));
    assert_eq!(n.estimate, Some(90));
    assert_eq!(n.priority, Some(HomeworkPriority::High));
    assert_eq!(n.stack.as_deref(), Some("french"));
    assert_eq!(n.checklist, ["Read", "Draft"], "blank steps are dropped");
}

#[test]
fn advance_tick_and_tab_name_their_item_and_tab() {
    assert_eq!(
        parse_words(&format!("homework advance {}", id())).unwrap(),
        Command::Homework(HomeworkCmd::Advance(Uid(7)))
    );
    assert_eq!(
        parse_words(&format!("homework tick {} 2", id())).unwrap(),
        Command::Homework(HomeworkCmd::Tick(Uid(7), 2))
    );
    assert_eq!(
        parse_words("homework tab due-today").unwrap(),
        Command::Homework(HomeworkCmd::Tab(HomeworkTab::DueToday))
    );
}

#[test]
fn bad_homework_words_say_what_is_wrong() {
    assert!(
        parse_words("homework tick nope 1")
            .unwrap_err()
            .contains("ULID")
    );
    assert!(
        parse_words(&format!("homework tick {} 0", id()))
            .unwrap_err()
            .contains("from 1")
    );
    assert!(
        parse_words("homework tab later")
            .unwrap_err()
            .contains("unknown tab")
    );
    assert!(
        parse_words("homework new X --priority urgent")
            .unwrap_err()
            .contains("unknown priority")
    );
    assert!(
        parse_words("homework new X --estimate lots")
            .unwrap_err()
            .contains("minutes")
    );
}

#[test]
fn reading_is_allowed_and_changing_is_a_write() {
    assert_eq!(class_of(&Command::Homework(HomeworkCmd::List)), Class::Read);
    assert_eq!(
        class_of(&Command::Homework(HomeworkCmd::Tab(HomeworkTab::All))),
        Class::Read
    );
    let new = HomeworkCmd::New(HomeworkNew::default());
    assert_eq!(class_of(&Command::Homework(new)), Class::Write);
    assert_eq!(
        class_of(&Command::Homework(HomeworkCmd::Advance(Uid(1)))),
        Class::Write
    );
}

#[test]
fn a_new_item_is_filled_in_first_and_saved_in_a_second_phase() {
    let cmd = Command::Homework(HomeworkCmd::New(HomeworkNew {
        title: "Lab".into(),
        due_date: Some("2026-12-02".into()),
        ..HomeworkNew::default()
    }));
    let plan = plan_for(&State::default(), &cmd).unwrap();
    assert_eq!(plan.len(), 2, "fill in, then save");
    assert!(matches!(plan[0][0], Msg::Homework(HomeworkMsg::New)));
    assert_eq!(plan[1], vec![Msg::Homework(HomeworkMsg::Save)]);
}

#[test]
fn an_unknown_stack_is_refused_before_anything_is_done() {
    let cmd = Command::Homework(HomeworkCmd::New(HomeworkNew {
        title: "Lab".into(),
        stack: Some("nowhere".into()),
        ..HomeworkNew::default()
    }));
    assert!(
        plan_for(&State::default(), &cmd)
            .unwrap_err()
            .contains("no stack in folder nowhere")
    );
}

#[test]
fn the_list_answers_that_it_is_not_loaded_before_the_app_reads_it() {
    let line = reply_line(
        &json!(1),
        &State::default(),
        &Command::Homework(HomeworkCmd::List),
        &None,
    );
    assert!(line.contains("\"loaded\":false"), "{line}");
}

#[test]
fn a_refused_change_answers_with_the_reason() {
    let mut state = State::default();
    state.planner.homework_ui.error = Some("the title must be one non-empty line".into());
    let line = reply_line(
        &json!(2),
        &state,
        &Command::Homework(HomeworkCmd::Advance(Uid(7))),
        &None,
    );
    assert!(
        line.contains("\"error\"") && line.contains("non-empty"),
        "{line}"
    );
}
