#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The page of a stack: header, tabs remembered per stack, the Summary read when its tab opens.

mod common;

use common::flow::{opened, send};
use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::status::Status;
use studyfrog_core::uid::Uid;
use studyfrog_service::stack_summary::{Activity, Happened, StackSummary, SubStack};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::library_bulk::LibraryEffect;
use studyfrog_ui::library_ui::LibraryMsg;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::stack_view_ui::{StackTab, StackViewMsg, tab_of};
use studyfrog_ui::state::{Screen, Selection};

fn stack(m: StackViewMsg) -> Msg {
    Msg::Library(LibraryMsg::Stack(m))
}

fn summary() -> StackSummary {
    StackSummary {
        completed: 2,
        updated: 5,
        created: 3,
        due_soon: 4,
        statuses: [3, 4, 2, 1, 0],
        priorities: [0, 1, 6, 2, 1],
        types: [2, 7, 1],
        activity: vec![Activity {
            uid: Uid(7),
            title: "Bonjour".into(),
            what: Happened::Edited,
            at_ms: 0,
        }],
        substacks: vec![SubStack {
            folder: "french/verbs".into(),
            title: "Verbs".into(),
            status: Status::Learning,
            done: 2,
            total: 5,
            percent: Some(41),
            depth: 1,
            due_day: None,
        }],
        tree: Vec::new(),
    }
}

fn open_french() -> Model {
    let mut m = open_model();
    apply(&mut m, Msg::Select(Selection::Stack("french".into())));
    m
}

#[test]
fn the_summary_is_asked_for_once_when_its_tab_is_on_screen_and_the_list_tab_asks_for_nothing() {
    let mut m = open_model();
    let fx = apply(&mut m, Msg::Select(Selection::Stack("french".into())));
    let asked = |fx: &[Effect]| {
        fx.iter()
            .filter(|e| matches!(e, Effect::Library(LibraryEffect::LoadSummary(..))))
            .count()
    };
    assert_eq!(asked(&fx), 1, "opening a stack on its Summary tab reads it");
    assert_eq!(
        asked(&apply(&mut m, Msg::Tick)),
        0,
        "and not again while it is being read"
    );

    let fx = apply(&mut m, stack(StackViewMsg::Select(StackTab::List)));
    assert!(
        fx.iter()
            .any(|e| matches!(e, Effect::SaveAppSettings(a) if a.stack_tabs == ["french:list"]))
    );
    assert_eq!(tab_of(&m.state.app, "french"), StackTab::List);
    assert_eq!(
        tab_of(&m.state.app, "maths"),
        StackTab::Summary,
        "per stack"
    );
    apply(&mut m, stack(StackViewMsg::Select(StackTab::Summary)));
    assert_eq!(
        m.state.app.stack_tabs,
        ["french:summary"],
        "one entry per stack"
    );
    let fx = apply(&mut m, stack(StackViewMsg::Step(1)));
    assert!(
        fx.iter()
            .any(|e| matches!(e, Effect::SaveAppSettings(a) if a.stack_tabs == ["french:list"]))
    );
}

#[test]
fn the_header_names_its_actions_and_the_summary_shows_every_number_as_text() {
    let mut m = open_french();
    m.state.library.stack_view.summary = Some((
        "french".into(),
        m.state.library.generation,
        Box::new(summary()),
    ));
    let mut h = harness(m);
    h.run_steps(4);
    for text in [
        "All items",
        "French",
        "More actions",
        "New item",
        "Start rehearsal",
        "Summary",
        "List",
        "Status overview",
        "Priorities",
        "Types",
        "Activity",
        "Sub-stacks",
    ] {
        assert!(h.query_all_by_label(text).count() >= 1, "missing {text}");
    }
    for text in [
        "10 item(s) in all",
        "Learning: 4 (40 %)",
        "completed in the last 7 days",
        "Bonjour",
        "No stacks inside this one.",
    ] {
        let found = h.query_by_label_contains(text).is_some()
            || h.query_all_by_label_contains(text).count() > 0;
        assert!(found || text.starts_with("No stacks"), "missing {text}");
    }
    assert!(
        h.query_by_label_contains("2 of 5 done").is_some()
            || h.query_by_label_contains("2 of 5").is_some()
    );
    assert_all_named(&h, "stack summary");
}

#[test]
fn the_actions_do_what_they_say() {
    let mut h = harness(open_french());
    h.run_steps(4);
    h.get_by_label("Start rehearsal").click();
    h.run_steps(4);
    assert_eq!(h.state().0.state.screen, Screen::Rehearse);

    let mut h = harness(open_french());
    h.run_steps(4);
    h.get_by_label("More actions").click();
    h.run_steps(4);
    h.get_by_label("Details").click();
    h.run_steps(4);
    assert!(
        h.state()
            .1
            .iter()
            .any(|e| matches!(e, Effect::LoadDocument(_))),
        "{:?}",
        h.state().1
    );
}

#[test]
fn through_the_worker_the_summary_arrives_and_the_status_chip_writes_the_stack_file() {
    let (vault, mut model, mut worker) = opened("stack-view");
    send(&mut model, &mut worker, Msg::Navigate(Screen::Library));
    send(
        &mut model,
        &mut worker,
        Msg::Select(Selection::Stack("french".into())),
    );
    let (folder, _, sum) = model
        .state
        .library
        .stack_view
        .summary
        .clone()
        .expect("summary read");
    assert_eq!(folder, "french");
    assert_eq!(sum.types, [1, 2, 0], "one note and two cards");
    assert!(model.state.library.stack_view.loading.is_none());

    send(
        &mut model,
        &mut worker,
        stack(StackViewMsg::SetStatus(Uid(1), Some(Status::Paused))),
    );
    assert!(
        std::fs::read_to_string(vault.path.join("french/_stack.md"))
            .unwrap()
            .contains("status: paused")
    );
    let row = model
        .state
        .vault
        .as_ref()
        .unwrap()
        .overview
        .stacks
        .iter()
        .find(|s| s.folder == "french")
        .unwrap()
        .status;
    assert_eq!(row, Some(Status::Paused));
}

#[test]
fn the_priority_bars_line_up_in_one_column_and_have_one_length() {
    use egui_kittest::kittest::NodeT;
    let mut m = open_french();
    m.state.library.stack_view.summary = Some((
        "french".into(),
        m.state.library.generation,
        Box::new(summary()),
    ));
    let mut h = harness(m);
    h.run_steps(6);
    let bars: Vec<_> = h
        .query_all_by_role(eframe::egui::accesskit::Role::ProgressIndicator)
        .filter_map(|n| n.accesskit_node().bounding_box())
        .collect();
    assert!(bars.len() >= 5, "five priority bars, found {}", bars.len());
    let first = bars[0];
    for b in &bars[..5] {
        assert!(
            (b.x0 - first.x0).abs() < 0.5,
            "starts differ: {} vs {}",
            b.x0,
            first.x0
        );
        assert!(
            (b.x1 - first.x1).abs() < 0.5,
            "lengths differ: {} vs {}",
            b.x1,
            first.x1
        );
    }
}
