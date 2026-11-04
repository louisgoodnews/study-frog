#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Headless UI and update tests for marking folders as stacks or normal folders (TASK-0217).

mod common;

use common::views::{assert_all_named, harness, open_model};
use common::{NOW_NS, sample_vault};
use egui_kittest::kittest::Queryable;
use studyfrog_core::uid::Uid;
use studyfrog_service::bulk::Snapshot;
use studyfrog_service::stack_mark_plan::UnmarkPlan;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{execute, worker_new};
use studyfrog_ui::modal::{Modal, ModalMsg, ToastAction};
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::stack_mark_ui::StackMarkEffect;
use studyfrog_ui::state::Selection;

fn sample_plan(folder: &str, title: &str) -> UnmarkPlan {
    UnmarkPlan {
        folder: folder.into(),
        title: title.into(),
        uid: Uid(42),
        has_settings: true,
        status: Some(studyfrog_core::status::Status::Learning),
        goals_count: 2,
        subjects: vec!["math".into()],
        sub_stacks: vec!["analysis".into()],
        referencing_items: vec![(Uid(100), "notes/test.md".into())],
        items_count: 5,
        target_folder: None,
        target_title: "Inbox".into(),
    }
}

#[test]
fn unmark_modal_renders_and_all_elements_are_named() {
    let mut m = open_model();
    let plan = sample_plan("french", "French");
    m.state.modal = Some(Modal::ConfirmUnmark(Box::new(plan)));

    let mut h = harness(m);
    h.run_steps(8);
    assert_all_named(&h, "confirm unmark dialog");
    assert!(
        h.query_by_label("Confirm making French a normal folder")
            .is_some()
    );
    assert!(h.query_by_label("Cancel").is_some());
}

#[test]
fn unmark_flow_produces_effects_updates_state_and_offers_undo_toast() {
    let mut m = open_model();
    let folder = "french".to_owned();

    // 1. AskUnmark produces PlanUnmark effect
    let effs = apply(&mut m, Msg::Modal(ModalMsg::AskUnmark(folder.clone())));
    assert_eq!(
        effs,
        vec![Effect::StackMark(StackMarkEffect::PlanUnmark(
            folder.clone()
        ))]
    );

    // 2. UnmarkPlanLoaded opens ConfirmUnmark modal
    let plan = sample_plan(&folder, "French");
    let effs = apply(
        &mut m,
        Msg::Modal(ModalMsg::UnmarkPlanLoaded(Ok(Box::new(plan.clone())))),
    );
    assert!(effs.is_empty());
    assert!(matches!(&m.state.modal, Some(Modal::ConfirmUnmark(_))));

    // 3. ConfirmUnmark produces Unmark effect
    let effs = apply(&mut m, Msg::Modal(ModalMsg::ConfirmUnmark(folder.clone())));
    assert_eq!(
        effs,
        vec![Effect::StackMark(StackMarkEffect::Unmark(folder.clone()))]
    );
    assert!(m.state.modal.is_none());

    // 4. Unmarked completes, resets selection, and shows Undo toast
    m.state.browser.selection = Selection::Stack(folder.clone());
    let mut info = m.state.vault.clone().unwrap();
    info.overview.stacks.retain(|s| s.folder != folder);
    let snap = Snapshot(Vec::new());
    apply(
        &mut m,
        Msg::Modal(ModalMsg::Unmarked(Ok((info, snap.clone(), plan)))),
    );

    assert_eq!(m.state.browser.selection, Selection::Nothing);
    assert_eq!(m.state.toasts.len(), 1);
    assert_eq!(m.state.toasts[0].action, ToastAction::UndoMark(snap));

    // 5. Clicking Undo toast requests stack mark undo
    let effs = apply(&mut m, Msg::Modal(ModalMsg::ToastClicked(0)));
    assert!(matches!(
        &effs[0],
        Effect::StackMark(StackMarkEffect::Undo(_))
    ));
}

#[test]
fn create_dialog_allows_creating_normal_folder_choice() {
    let mut m = open_model();
    apply(&mut m, Msg::Modal(ModalMsg::OpenCreateStack));

    // Initially is_stack is true
    assert!(matches!(&m.state.modal, Some(Modal::Create(f)) if f.is_stack));

    // Toggle is_stack to false
    apply(&mut m, Msg::Modal(ModalMsg::SetIsStack(false)));
    assert!(matches!(&m.state.modal, Some(Modal::Create(f)) if !f.is_stack));

    // Clear parent and edit title
    apply(&mut m, Msg::Modal(ModalMsg::SetParent(None)));
    apply(
        &mut m,
        Msg::Modal(ModalMsg::Edit(
            studyfrog_ui::modal::Field::Title,
            "docs".into(),
        )),
    );

    // Submitting creates folder effect
    let effs = apply(&mut m, Msg::Modal(ModalMsg::Submit));
    assert_eq!(
        effs,
        vec![Effect::StackMark(StackMarkEffect::CreateFolder {
            parent: None,
            name: "docs".into(),
        })]
    );
}

#[test]
fn executor_runs_mark_and_unmark_against_real_vault() {
    let vault = sample_vault("stack-mark-ui");
    let mut w = worker_new(None);
    execute(
        &mut w,
        Effect::OpenVault(vault.path.display().to_string()),
        NOW_NS,
    );

    // 1. Create a normal folder on disk
    let msgs = execute(
        &mut w,
        Effect::StackMark(StackMarkEffect::CreateFolder {
            parent: None,
            name: "notes".into(),
        }),
        NOW_NS,
    );
    assert!(matches!(
        &msgs[0],
        Msg::Modal(ModalMsg::CreatedFolder(Ok(_)))
    ));
    assert!(vault.path.join("notes").is_dir());
    assert!(!vault.path.join("notes/_stack.md").exists());

    // 2. Mark the folder as a stack
    let msgs = execute(
        &mut w,
        Effect::StackMark(StackMarkEffect::Mark("notes".into())),
        NOW_NS,
    );
    let Msg::Modal(ModalMsg::Marked(Ok((info, _mark_snap, created)))) = &msgs[0] else {
        panic!("failed to mark: {:?}", msgs[0]);
    };
    assert_eq!(created.title, "notes");
    assert!(vault.path.join("notes/_stack.md").exists());
    assert!(info.overview.stacks.iter().any(|s| s.folder == "notes"));

    // 3. Plan unmark
    let msgs = execute(
        &mut w,
        Effect::StackMark(StackMarkEffect::PlanUnmark("notes".into())),
        NOW_NS,
    );
    assert!(matches!(
        &msgs[0],
        Msg::Modal(ModalMsg::UnmarkPlanLoaded(Ok(_)))
    ));

    // 4. Unmark stack
    let msgs = execute(
        &mut w,
        Effect::StackMark(StackMarkEffect::Unmark("notes".into())),
        NOW_NS,
    );
    let Msg::Modal(ModalMsg::Unmarked(Ok((_info, unmark_snap, _plan)))) = &msgs[0] else {
        panic!("failed to unmark: {:?}", msgs[0]);
    };
    assert!(!vault.path.join("notes/_stack.md").exists());

    // 5. Undo unmark restores stack
    let msgs = execute(
        &mut w,
        Effect::StackMark(StackMarkEffect::Undo(unmark_snap.clone())),
        NOW_NS,
    );
    assert!(matches!(&msgs[0], Msg::Modal(ModalMsg::MarkUndone(Ok(_)))));
    assert!(vault.path.join("notes/_stack.md").exists());
}
