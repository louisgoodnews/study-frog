#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Sub-stacks tab: the tree, folding, opening a sub-stack and the empty state.

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::status::Status;
use studyfrog_service::stack_summary::{StackSummary, SubStack};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Selection;

fn sub(folder: &str, title: &str, depth: usize, due: Option<i64>) -> SubStack {
    SubStack {
        folder: folder.into(),
        title: title.into(),
        status: Status::Learning,
        done: 1,
        total: 4,
        percent: Some(55),
        depth,
        due_day: due,
    }
}

fn model(tree: Vec<SubStack>) -> Model {
    let mut m = open_model();
    m.state.app.stack_tabs = vec!["french:substacks".into()];
    apply(&mut m, Msg::Select(Selection::Stack("french".into())));
    let summary = StackSummary {
        completed: 0,
        updated: 0,
        created: 0,
        due_soon: 0,
        statuses: [0; 5],
        priorities: [0; 5],
        types: [0; 3],
        activity: vec![],
        substacks: vec![],
        tree,
    };
    m.state.library.stack_view.summary = Some((
        "french".into(),
        m.state.library.generation,
        Box::new(summary),
    ));
    m
}

#[test]
fn the_tree_shows_status_progress_due_date_and_familiarity_and_folds_and_opens() {
    let tree = vec![
        sub("french/verbs", "Verbs", 1, Some(20_500)),
        sub("french/verbs/irregular", "Irregular", 2, None),
        sub("french/nouns", "Nouns", 1, None),
    ];
    let mut h = harness(model(tree));
    h.run_steps(4);
    assert_eq!(h.query_all_by_label_contains("1 of 4 done").count(), 3);
    assert!(h.query_all_by_label_contains("familiarity 55 %").count() >= 3);
    assert_eq!(h.query_all_by_label_contains("due 20").count(), 1);
    assert!(h.query_by_label("Irregular").is_some());
    assert_all_named(&h, "sub-stacks tab");

    h.get_by_label("Fold Verbs").click();
    h.run_steps(3);
    assert!(h.query_by_label("Irregular").is_none(), "folded");
    assert!(h.query_by_label("Nouns").is_some(), "its sibling stays");
    h.get_by_label("Unfold Verbs").click();
    h.run_steps(3);
    assert!(h.query_by_label("Irregular").is_some());

    h.get_by_label("Irregular").click();
    h.run_steps(3);
    assert_eq!(
        h.state().0.state.browser.selection,
        Selection::Stack("french/verbs/irregular".into())
    );
}

#[test]
fn a_stack_without_stacks_inside_shows_an_empty_state() {
    let mut h = harness(model(vec![]));
    h.run_steps(4);
    assert!(
        h.query_by_label_contains("No stacks inside this one")
            .is_some()
    );
}
