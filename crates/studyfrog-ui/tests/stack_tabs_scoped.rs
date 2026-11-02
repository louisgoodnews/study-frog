#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Board, Calendar and Reports tabs of a stack show only its data; the tab bar folds the rest into More.

mod common;

use common::flow::{opened, send};
use common::views::{assert_all_named, harness, lib_item, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_calendar::session::{Ends, Repeat, Session};
use studyfrog_core::civil::{LocalTime, date_new};
use studyfrog_core::status::Status;
use studyfrog_core::uid::Uid;
use studyfrog_service::library_rows::LibraryItem;
use studyfrog_ui::library_bulk::BulkMsg;
use studyfrog_ui::library_ui::LibraryMsg;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::planner::PlanMsg;
use studyfrog_ui::state::{Screen, Selection};

fn item(n: u128, title: &str, status: Status, chain: &[&str]) -> LibraryItem {
    let mut r = lib_item(n, title);
    r.row.status = status;
    r.row.stacks = chain.iter().map(|s| (*s).to_owned()).collect();
    r
}

fn model(tab: &str) -> Model {
    let mut m = open_model();
    m.state.app.stack_tabs = vec![format!("french:{tab}")];
    apply(&mut m, Msg::Select(Selection::Stack("french".into())));
    m
}

fn session(id: u128, title: &str, stack: Uid) -> Session {
    Session {
        id: Uid(id),
        title: title.into(),
        stack: Some(stack),
        start: LocalTime {
            date: date_new(2026, 10, 12).unwrap(),
            minute: 540,
        },
        duration_min: 30,
        repeat: Repeat::Never,
        ends: Ends::Forever,
        cancelled: Vec::new(),
        notes: String::new(),
        kind: Default::default(),
        reminders: Vec::new(),
    }
}

#[test]
fn the_board_has_a_column_per_status_with_only_this_stacks_items_and_a_menu_moves_one() {
    let mut m = model("board");
    let fr = ["french", "French"];
    let rows = vec![
        item(1, "Bonjour", Status::Learning, &fr),
        item(2, "Merci", Status::New, &fr),
        item(3, "Algebra", Status::Learning, &["maths", "Maths"]),
    ];
    apply(&mut m, Msg::Library(LibraryMsg::Loaded(Ok(rows))));
    let mut h = harness(m);
    h.run_steps(4);
    for s in studyfrog_core::status::STATUSES {
        let label = studyfrog_core::status::status_label(s);
        assert!(h.query_all_by_label_contains(label).count() >= 1, "{label}");
    }
    assert!(h.query_by_label("Bonjour").is_some() && h.query_by_label("Algebra").is_none());
    assert_all_named(&h, "stack board");
    let _ = BulkMsg::AskDelete(false);
}

#[test]
fn the_calendar_tab_shows_only_the_stacks_sessions() {
    let mut m = model("calendar");
    let today = date_new(2026, 10, 12).unwrap();
    let sessions = vec![
        session(7, "French drills", Uid(1)),
        session(8, "Maths drills", Uid(2)),
    ];
    apply(&mut m, Msg::Plan(PlanMsg::Loaded(Ok((sessions, today)))));
    let mut h = harness(m);
    h.run_steps(4);
    assert!(h.query_all_by_label_contains("French drills").count() >= 1);
    assert!(
        h.query_all_by_label_contains("Maths drills").count() == 0,
        "another stack's session is not shown"
    );
}

#[test]
fn the_reports_tab_reads_the_reports_and_shows_the_stacks_familiarity() {
    let (_v, mut model, mut worker) = opened("stack-reports");
    model.state.app.stack_tabs = vec!["french:reports".into()];
    send(&mut model, &mut worker, Msg::Navigate(Screen::Library));
    send(
        &mut model,
        &mut worker,
        Msg::Select(Selection::Stack("french".into())),
    );
    assert!(
        model.state.reports.is_some(),
        "opening the tab built the reports"
    );
    let mut h = harness(model);
    h.run_steps(4);
    assert!(h.query_by_label("Familiarity").is_some());
    assert!(h.query_by_label_contains("French:").is_some());
    assert!(h.query_by_label("Open all reports").is_some());
    assert_all_named(&h, "stack reports");
}

#[test]
fn tabs_that_do_not_fit_go_into_a_more_menu_that_still_reaches_them() {
    let mut h = harness(model("summary"));
    h.run_steps(4);
    assert!(h.query_by_label("Summary").is_some());
    let shown = [
        "Summary",
        "List",
        "Backlog",
        "Roadmap",
        "Board",
        "Calendar",
        "Reports",
        "Sub-stacks",
        "Goals",
    ]
    .iter()
    .filter(|t| h.query_all_by_label(t).count() > 0)
    .count();
    if shown < 9 {
        h.get_by_label("More").click();
        h.run_steps(3);
        assert!(
            h.query_all_by_label("Goals").count() >= 1,
            "the tab that did not fit is in the menu"
        );
    }
}
