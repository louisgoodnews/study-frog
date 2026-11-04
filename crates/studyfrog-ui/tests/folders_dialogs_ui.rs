#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Headless UI tests for Folders dialogs: new folder popover, move confirmation, and delete confirmation (TASK-0204).

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_service::folder_ops::{DeletePlan, MovePlan};
use studyfrog_ui::folders_types::NewFolderForm;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

#[test]
fn folders_dialogs_new_folder_and_cross_stack_confirmation() {
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Folders));

    // 1. New Folder Popover
    m.state.folders.new_folder = Some(NewFolderForm {
        parent: "french".into(),
        name: "grammar".into(),
        is_stack: true,
    });
    let mut h = harness(m);
    h.run();
    assert_all_named(&h, "new folder dialog");
    assert!(h.query_by_label("Create folder").is_some());
    assert!(h.query_by_label("Mark as stack").is_some());

    // 2. Cross-stack move confirmation dialog
    let plan = MovePlan {
        src: "french/verbs.md".into(),
        dst: "german/verbs.md".into(),
        is_folder: false,
        files: vec![("french/verbs.md".into(), "german/verbs.md".into())],
        is_cross_stack: true,
        source_stack: Some("french".into()),
        destination_stack: Some("german".into()),
        source_stack_title: "French".into(),
        destination_stack_title: "German".into(),
        items_changing_primary_stack: 1,
        schedules_dropped: 0,
        dropped_memory: Vec::new(),
        doc_updates: Vec::new(),
        link_rewrites: Vec::new(),
    };
    h.state_mut().0.state.folders.new_folder = None;
    h.state_mut().0.state.folders.confirm_move = Some(Box::new(plan));
    h.run();
    assert_all_named(&h, "cross stack move dialog");
    assert!(h.query_by_label("Confirm move").is_some());

    // 3. Delete confirmation dialog
    let del_plan = DeletePlan {
        target: "french".into(),
        is_folder: true,
        files_count: 2,
        items_count: 1,
        stacks_count: 1,
        files_to_delete: vec!["french/verbs.md".into(), "french/guide.pdf".into()],
    };
    h.state_mut().0.state.folders.confirm_move = None;
    h.state_mut().0.state.folders.confirm_delete = Some(Box::new(del_plan));
    h.run();
    assert_all_named(&h, "confirm delete dialog");
    assert!(h.query_by_label("Confirm delete").is_some());
}
