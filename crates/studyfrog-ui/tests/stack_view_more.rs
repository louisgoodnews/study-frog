#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The page of a stack: header, tabs remembered per stack, the Summary read when its tab opens.
use common::views::{harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::status::Status;
use studyfrog_core::uid::Uid;
use studyfrog_service::stack_summary::{Activity, Happened, StackSummary, SubStack};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::msg::MsgBrowse;
use studyfrog_ui::state::Selection;

mod common;

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
    apply(
        &mut m,
        Msg::Browse(MsgBrowse::Select(Selection::Stack("french".into()))),
    );
    m
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
        assert!((b.x0 - first.x0).abs() < 0.5 && (b.x1 - first.x1).abs() < 0.5);
    }
}
