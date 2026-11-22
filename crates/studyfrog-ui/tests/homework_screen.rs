#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The homework planner screen (STORY-0167): the dialog's rules and the screen's transitions.
//! Pure update, no I/O.

use studyfrog_calendar::homework::{ChecklistItem, HomeworkItem, HomeworkPriority, HomeworkStatus};
use studyfrog_calendar::homework_query::HomeworkTab;
use studyfrog_core::civil::{Date, LocalTime};
use studyfrog_core::uid::Uid;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::homework_form::{HomeworkEdit, form_of, new_form, to_item};
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
        checklist: vec![
            ChecklistItem {
                text: "Read".into(),
                done: true,
            },
            ChecklistItem {
                text: "Draft".into(),
                done: false,
            },
        ],
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
fn a_new_form_starts_blank_and_due_at_the_end_of_today() {
    let f = new_form(day(21));
    assert_eq!(f.existing, None);
    assert_eq!(
        (f.due_date.as_str(), f.due_time.as_str()),
        ("2026-11-21", "23:59")
    );
    assert_eq!(f.estimate_min, 60);
    assert_eq!(f.priority, HomeworkPriority::Medium);
}

#[test]
fn saving_a_new_item_asks_the_worker_for_it_without_an_id() {
    let (s, _) = send(loaded(Vec::new()), HomeworkMsg::New);
    assert!(s.planner.homework_ui.form.is_some());
    let (s, _) = send(
        s,
        HomeworkMsg::Edit(HomeworkEdit::Title("Lab report".into())),
    );
    let (s, fx) = send(s, HomeworkMsg::Save);
    let [Effect::Homework(HomeworkEffect::Save { existing, item })] = fx.as_slice() else {
        panic!("expected one save effect, got {fx:?}");
    };
    assert_eq!(*existing, None);
    assert_eq!(item.title, "Lab report");
    assert!(
        s.planner.homework_ui.form.is_some(),
        "the dialog stays open until the save succeeds"
    );
}

#[test]
fn a_bad_due_date_or_time_is_refused_in_the_dialog() {
    let (s, _) = send(loaded(Vec::new()), HomeworkMsg::New);
    let (s, _) = send(
        s,
        HomeworkMsg::Edit(HomeworkEdit::DueDate("tomorrow".into())),
    );
    let (s, fx) = send(s, HomeworkMsg::Save);
    assert!(fx.is_empty());
    assert!(
        s.planner
            .homework_ui
            .error
            .clone()
            .unwrap()
            .contains("due date")
    );
    let (s, _) = send(
        s,
        HomeworkMsg::Edit(HomeworkEdit::DueDate("2026-11-30".into())),
    );
    let (s, _) = send(s, HomeworkMsg::Edit(HomeworkEdit::DueTime("25:00".into())));
    let (s, fx) = send(s, HomeworkMsg::Save);
    assert!(fx.is_empty());
    assert!(
        s.planner
            .homework_ui
            .error
            .clone()
            .unwrap()
            .contains("due time")
    );
}

#[test]
fn editing_keeps_the_start_status_and_ticked_steps() {
    let existing = item(7, 25);
    let (s, _) = send(
        loaded(vec![existing.clone()]),
        HomeworkMsg::EditItem(Uid(7)),
    );
    let form = s.planner.homework_ui.form.clone().unwrap();
    assert_eq!(form.existing, Some(Uid(7)));
    assert_eq!(form.checklist, "Read\nDraft");
    let (s, _) = send(
        s,
        HomeworkMsg::Edit(HomeworkEdit::Checklist("Read\n\n  Draft\nPolish".into())),
    );
    let (_, fx) = send(s, HomeworkMsg::Save);
    let [Effect::Homework(HomeworkEffect::Save { existing: id, item })] = fx.as_slice() else {
        panic!("expected a save effect");
    };
    assert_eq!(*id, Some(Uid(7)));
    assert_eq!(item.checklist.len(), 3, "blank lines are dropped");
    assert!(
        item.checklist[0].done,
        "an unchanged ticked step stays ticked"
    );
    assert!(!item.checklist[1].done);
    assert_eq!(item.status, HomeworkStatus::NotStarted);
}

#[test]
fn a_saved_item_closes_the_dialog_and_refreshes_the_list() {
    let (s, _) = send(loaded(Vec::new()), HomeworkMsg::New);
    let (s, _) = send(s, HomeworkMsg::Saved(Ok((vec![item(1, 24)], clock()))));
    assert!(s.planner.homework_ui.form.is_none());
    assert_eq!(s.planner.homework.len(), 1);
    assert!(s.planner.homework_ui.error.is_none());
}

#[test]
fn a_refused_save_keeps_the_dialog_open_with_the_reason() {
    let (s, _) = send(loaded(Vec::new()), HomeworkMsg::New);
    let (s, _) = send(
        s,
        HomeworkMsg::Saved(Err("the title must be one non-empty line".into())),
    );
    assert!(s.planner.homework_ui.form.is_some());
    assert!(
        s.planner
            .homework_ui
            .error
            .clone()
            .unwrap()
            .contains("title")
    );
}

#[test]
fn advancing_and_ticking_ask_the_worker_and_keep_the_dialog_as_it_is() {
    let s = loaded(vec![item(3, 25)]);
    let (_, fx) = send(s.clone(), HomeworkMsg::Advance(Uid(3)));
    assert_eq!(fx, vec![Effect::Homework(HomeworkEffect::Advance(Uid(3)))]);
    let (_, fx) = send(s.clone(), HomeworkMsg::ToggleStep(Uid(3), 1));
    assert_eq!(
        fx,
        vec![Effect::Homework(HomeworkEffect::ToggleStep(Uid(3), 1))]
    );
    let (s, _) = send(s, HomeworkMsg::Changed(Ok((vec![item(3, 25)], clock()))));
    assert_eq!(s.planner.homework.len(), 1);
}

#[test]
fn tabs_are_remembered_and_the_dialog_closes_on_request() {
    let (s, _) = send(State::default(), HomeworkMsg::Tab(HomeworkTab::Overdue));
    assert_eq!(s.planner.homework_ui.tab, HomeworkTab::Overdue);
    let (s, _) = send(s, HomeworkMsg::New);
    let (s, _) = send(s, HomeworkMsg::Close);
    assert!(s.planner.homework_ui.form.is_none());
}

#[test]
fn to_item_trims_the_title_and_checks_the_date_and_time() {
    let mut f = form_of(&item(9, 26));
    f.title = "  Problem set  ".into();
    let ok = to_item(&f, Uid(9)).unwrap();
    assert_eq!(ok.title, "Problem set");
    assert_eq!(ok.due.date, day(26));
    f.due_time = "18:30".into();
    assert_eq!(to_item(&f, Uid(9)).unwrap().due.minute, 18 * 60 + 30);
}

#[test]
fn the_subject_filter_is_kept_when_the_tab_changes() {
    let (s, _) = send(State::default(), HomeworkMsg::Subject(Some(Uid(5))));
    assert_eq!(s.planner.homework_ui.subject, Some(Uid(5)));
    let (s, _) = send(s, HomeworkMsg::Tab(HomeworkTab::Overdue));
    assert_eq!(s.planner.homework_ui.tab, HomeworkTab::Overdue);
    assert_eq!(
        s.planner.homework_ui.subject,
        Some(Uid(5)),
        "tab change keeps the subject"
    );
    let (s, _) = send(s, HomeworkMsg::Subject(None));
    assert_eq!(s.planner.homework_ui.subject, None);
}

#[test]
fn the_subject_filter_narrows_the_tab_list_to_that_stack() {
    use studyfrog_calendar::homework_query::{planner_order, tab_filter};
    let mut mine = item(1, 22);
    mine.course = Some(Uid(5));
    let other = item(2, 22);
    let items = [mine, other];
    let filter = tab_filter(HomeworkTab::All, Some(Uid(5)));
    let shown: Vec<Uid> = planner_order(&items, &filter, clock())
        .iter()
        .map(|i| i.id)
        .collect();
    assert_eq!(shown, [Uid(1)]);
}
