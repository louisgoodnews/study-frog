#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Homework planner UI: due-date order, urgency labels, completed items set apart below the open ones.

mod common;

use std::time::{SystemTime, UNIX_EPOCH};

use common::views::{harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_calendar::homework::{HomeworkItem, HomeworkPriority, HomeworkStatus};
use studyfrog_core::civil::{LocalTime, add_days, utc_to_local};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::timezone::local_offset_minutes;
use studyfrog_ui::model::Model;
use studyfrog_ui::state::Screen;

/// The local time right now, as the planner sees it.
fn now() -> LocalTime {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX));
    let at = UnixMs(ms);
    utc_to_local(at, local_offset_minutes(at))
}

/// An assignment due `days` after today at `minute`.
fn item(
    n: u128,
    title: &str,
    subject: &str,
    days: i64,
    minute: u16,
    status: HomeworkStatus,
) -> HomeworkItem {
    let today = now().date;
    HomeworkItem {
        id: Uid(n),
        title: title.into(),
        subject: Some(subject.into()),
        stack_uid: None,
        timetable_id: None,
        due: LocalTime {
            date: add_days(today, days),
            minute,
        },
        start_date: None,
        estimated_minutes: 30,
        priority: HomeworkPriority::Medium,
        status,
        checklist: Vec::new(),
        notes: String::new(),
        reminders: Vec::new(),
    }
}

/// One item per urgency, deliberately added out of due-date order.
fn sample() -> Vec<HomeworkItem> {
    use HomeworkStatus::{Completed, NotStarted};
    vec![
        item(4, "Problem set", "Maths", 1, 900, NotStarted),
        item(6, "Project", "Physics", 3, 600, Completed),
        item(2, "Essay draft", "English", 0, 1439, NotStarted),
        item(7, "Old quiz", "Chemistry", -5, 480, Completed),
        item(5, "Reading", "History", 9, 480, NotStarted),
        item(1, "Lab report", "Biology", -2, 600, NotStarted),
    ]
}

fn with_planner(items: Vec<HomeworkItem>) -> Model {
    let mut m = open_model();
    m.state.homework = items;
    m.state.screen = Screen::Homework;
    m
}

/// The vertical position of the first label named `name`.
fn top(h: &egui_kittest::Harness<'_, common::views::App>, name: &str) -> f32 {
    h.get_by_label(name).rect().top()
}

#[test]
fn the_sidebar_opens_the_planner() {
    let mut h = harness(open_model());
    h.run_steps(4);
    h.get_by_label("Homework").click();
    h.run_steps(4);
    assert_eq!(h.state().0.state.screen, Screen::Homework);
}

#[test]
fn open_items_are_listed_by_due_date_with_their_urgency() {
    let mut h = harness(with_planner(sample()));
    h.run_steps(8);
    let order = ["Lab report", "Essay draft", "Problem set", "Reading"];
    for pair in order.windows(2) {
        assert!(top(&h, pair[0]) < top(&h, pair[1]), "{pair:?} out of order");
    }
    for (title, label) in [
        ("Lab report", "Overdue"),
        ("Essay draft", "Due today"),
        ("Problem set", "Due tomorrow"),
        ("Reading", "Upcoming"),
    ] {
        let row = h.get_by_label(title).rect();
        let badge = h.get_by_label(label).rect();
        assert!(
            (row.top() - badge.top()).abs() < row.height() + 4.0,
            "{label} is not on the row of {title}"
        );
    }
    assert!(h.query_all_by_label("Maths").count() >= 1);
}

#[test]
fn completed_items_sit_below_the_open_ones_in_their_own_section() {
    let mut h = harness(with_planner(sample()));
    h.run_steps(8);
    let last_open = top(&h, "Reading");
    let heading = top(&h, "Completed");
    assert!(last_open < heading, "completed heading above open items");
    assert!(heading < top(&h, "Old quiz"), "Old quiz above its heading");
    assert!(
        top(&h, "Old quiz") < top(&h, "Project"),
        "completed not in due-date order"
    );
    assert_eq!(h.query_all_by_label("Done").count(), 2);
}

#[test]
fn an_empty_planner_says_so_and_has_no_completed_section() {
    let mut h = harness(with_planner(Vec::new()));
    h.run_steps(4);
    assert!(h.query_all_by_label("Completed").next().is_none());
    assert!(h.query_all_by_label_contains("No homework yet").count() >= 1);
}

#[test]
fn items_saved_to_the_vault_load_into_the_planner_in_order() {
    use common::e2e_helpers::{boot_empty_harness, run_harness_until_idle};
    use studyfrog_service::homework::save_homework;
    use studyfrog_ui::model::apply;
    use studyfrog_ui::msg::Msg;

    let (_dir, mut worker, mut h) = boot_empty_harness("homework-planner-load");
    let session = worker.session.as_mut().expect("vault is open");
    save_homework(
        session,
        item(11, "Reading", "History", 2, 480, HomeworkStatus::NotStarted),
    )
    .expect("open item saved");
    save_homework(
        session,
        item(
            12,
            "Old quiz",
            "Chemistry",
            -3,
            600,
            HomeworkStatus::Completed,
        ),
    )
    .expect("completed item saved");

    let effects = apply(&mut h.state_mut().0, Msg::Navigate(Screen::Homework));
    h.state_mut().1.extend(effects);
    run_harness_until_idle(&mut h, &mut worker);
    h.run_steps(8);

    assert_eq!(h.state().0.state.homework.len(), 2);
    assert!(
        top(&h, "Reading") < top(&h, "Completed"),
        "open item below heading"
    );
    assert!(
        top(&h, "Completed") < top(&h, "Old quiz"),
        "completed item above heading"
    );
    assert_eq!(h.query_all_by_label("Done").count(), 1);
}
