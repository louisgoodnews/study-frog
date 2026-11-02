#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Backlog tab: grouping, the count per status, and moving items between the lists.

mod common;

use common::flow::{opened, send};
use common::views::{assert_all_named, harness, lib_item, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::status::Status;
use studyfrog_core::uid::Uid;
use studyfrog_service::library_rows::LibraryItem;
use studyfrog_ui::library_ui::LibraryMsg;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::stack_backlog::{backlog, counts};
use studyfrog_ui::state::{Screen, Selection};

fn row(n: u128, title: &str, status: Status, due: Option<i64>, stacks: &[&str]) -> LibraryItem {
    let mut r = lib_item(n, title);
    r.row.status = status;
    r.row.due_day = due;
    r.row.stacks = stacks.iter().map(|s| (*s).to_owned()).collect();
    r
}

const TODAY: i64 = 20_000;

fn rows() -> Vec<LibraryItem> {
    let fr = ["french", "French"];
    let verbs = ["french", "French", "french/verbs", "Verbs"];
    vec![
        row(1, "Active", Status::Learning, None, &fr),
        row(2, "Soon", Status::New, Some(TODAY + 3), &fr),
        row(3, "Far", Status::New, Some(TODAY + 60), &fr),
        row(4, "Fresh", Status::New, None, &verbs),
        row(5, "Held", Status::Paused, Some(TODAY + 1), &fr),
        row(6, "Finished", Status::Done, None, &fr),
        row(7, "Elsewhere", Status::Learning, None, &["maths", "Maths"]),
        row(8, "Later active", Status::Reviewing, Some(TODAY + 90), &fr),
    ]
}

fn model() -> Model {
    let mut m = open_model();
    m.state.app.stack_tabs = vec!["french:backlog".into()];
    m.state.wall_ms = TODAY * 86_400_000;
    apply(&mut m, Msg::Library(LibraryMsg::Loaded(Ok(rows()))));
    apply(&mut m, Msg::Select(Selection::Stack("french".into())));
    m
}

#[test]
fn items_group_into_up_next_and_not_yet_planned_with_their_status_counts() {
    let m = model();
    let b = backlog(&m.state.library, "french", TODAY);
    let titles = |v: &[usize]| {
        v.iter()
            .map(|i| m.state.library.items[*i].row.title.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        titles(&b.up_next),
        ["Soon", "Later active", "Active"].map(String::from),
        "soonest due first, undated last"
    );
    assert_eq!(
        titles(&b.not_planned),
        ["Far", "Fresh", "Held"].map(String::from),
        "the sub-stack's item is in, Done and other stacks are out"
    );
    assert_eq!(counts(&m.state.library, &b.not_planned), [2, 0, 0, 0, 1]);
}

#[test]
fn the_tab_shows_both_lists_and_a_button_moves_an_item_through_the_worker() {
    let mut h = harness(model());
    h.run_steps(4);
    for text in ["Up next (", "Not yet planned ("] {
        assert!(h.query_by_label_contains(text).is_some(), "missing {text}");
    }
    for text in [
        "Move Active to Not yet planned",
        "Move Fresh to Up next",
        "Move Held to Up next",
    ] {
        assert!(h.query_by_label(text).is_some(), "missing {text}");
    }
    assert_all_named(&h, "backlog tab");

    let (vault, mut model, mut worker) = opened("backlog-move");
    model.state.app.stack_tabs = vec!["french:backlog".into()];
    send(&mut model, &mut worker, Msg::Navigate(Screen::Library));
    send(
        &mut model,
        &mut worker,
        Msg::Select(Selection::Stack("french".into())),
    );
    let before = backlog(&model.state.library, "french", 0);
    assert_eq!(
        before.up_next.len() + before.not_planned.len(),
        3,
        "the three items of the sample stack are all new"
    );
    let uid = Uid(10);
    send(
        &mut model,
        &mut worker,
        Msg::Library(LibraryMsg::Bulk(
            studyfrog_ui::library_bulk::BulkMsg::MoveOne(
                uid,
                studyfrog_service::bulk::BulkEdit::Status(Status::Learning),
            ),
        )),
    );
    assert!(
        std::fs::read_to_string(vault.path.join("french/a.md"))
            .unwrap()
            .contains("status: learning")
    );
    let row = model
        .state
        .library
        .items
        .iter()
        .find(|i| i.uid == uid)
        .unwrap();
    assert_eq!(row.row.status, Status::Learning);
}
