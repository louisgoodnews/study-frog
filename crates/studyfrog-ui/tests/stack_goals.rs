#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Goals tab: read when it opens, edited in place, saved into the stack's file.

mod common;

use common::flow::{opened, send};
use common::views::{assert_all_named, harness};
use egui_kittest::kittest::Queryable;
use studyfrog_ui::library_ui::LibraryMsg;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::stack_view_ui::StackViewMsg;
use studyfrog_ui::state::{Screen, Selection};

fn stack(m: StackViewMsg) -> Msg {
    Msg::Library(LibraryMsg::Stack(m))
}

#[test]
fn goals_are_read_edited_saved_to_the_stack_file_and_read_again() {
    let (vault, mut model, mut worker) = opened("goals");
    model.state.app.stack_tabs = vec!["french:goals".into()];
    send(&mut model, &mut worker, Msg::Navigate(Screen::Library));
    send(
        &mut model,
        &mut worker,
        Msg::Select(Selection::Stack("french".into())),
    );
    let g = model
        .state
        .library
        .stack_view
        .goals
        .clone()
        .expect("goals read");
    assert_eq!(
        (g.folder.as_str(), g.draft.as_str(), g.saving),
        ("french", "", false)
    );

    send(
        &mut model,
        &mut worker,
        stack(StackViewMsg::GoalsEdited(
            "- pass the exam\n- [ ] 100 verbs".into(),
        )),
    );
    let g = model.state.library.stack_view.goals.clone().unwrap();
    assert_ne!(g.draft, g.saved, "unsaved");
    send(&mut model, &mut worker, stack(StackViewMsg::GoalsSave));
    let g = model.state.library.stack_view.goals.clone().unwrap();
    assert_eq!(g.draft, g.saved);
    let file = std::fs::read_to_string(vault.path.join("french/_stack.md")).unwrap();
    assert!(
        file.contains("## Goals\n\n- pass the exam\n- [ ] 100 verbs"),
        "{file}"
    );
    assert_eq!(model.state.notice.as_ref().unwrap().text, "Goals saved.");

    model.state.library.stack_view.goals = None;
    send(&mut model, &mut worker, Msg::Tick);
    assert_eq!(
        model.state.library.stack_view.goals.unwrap().saved,
        "- pass the exam\n- [ ] 100 verbs"
    );
}

#[test]
fn the_tab_shows_an_editor_and_a_save_button_that_wakes_up_with_changes() {
    let (_vault, mut model, mut worker) = opened("goals-view");
    model.state.app.stack_tabs = vec!["french:goals".into()];
    send(&mut model, &mut worker, Msg::Navigate(Screen::Library));
    send(
        &mut model,
        &mut worker,
        Msg::Select(Selection::Stack("french".into())),
    );
    let mut h = harness(model);
    h.run_steps(5);
    assert!(h.query_all_by_label("Goals").count() >= 1);
    assert!(h.query_by_label("Save goals").is_some());
    assert!(h.query_by_label("Resize Goals").is_some());
    assert_all_named(&h, "goals tab");
    assert!(h.query_by_label_contains("Unsaved changes").is_none());
}
