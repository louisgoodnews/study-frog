#![allow(clippy::unwrap_used, clippy::expect_used, clippy::cognitive_complexity)] // tests may panic on failure
//! Empirical persistence test for Milestone 3: Stack Due Date through real Worker & SQLite vault.

mod common;

use common::flow::{opened, send};
use common::run_to_idle;
use studyfrog_calendar::session::EventKind;
use studyfrog_core::civil::Date;
use studyfrog_core::uid::Uid;
use studyfrog_ui::deadline_ui::deadline_of;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::worker_new;
use studyfrog_ui::model::Model;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::plan_msg::PlanMsg;
use studyfrog_ui::state::Screen;

#[test]
fn test_stack_due_date_real_vault_persistence_lifecycle() {
    let (_vault, mut model, mut worker) = opened("m3-due-vault");
    // Ensure planner sessions are loaded
    send(&mut model, &mut worker, Msg::Navigate(Screen::Calendar));
    assert!(model.state.planner.loaded);
    assert!(model.state.planner.sessions.is_empty());

    // Switch to editor
    send(&mut model, &mut worker, Msg::Navigate(Screen::Editor));
    assert_eq!(model.state.screen, Screen::Editor);

    let stack_uid = Uid(1);
    let due_1 = Date {
        year: 2026,
        month: 11,
        day: 25,
    };

    // 1. Dispatch SetStackDueDate
    send(
        &mut model,
        &mut worker,
        Msg::Plan(PlanMsg::SetStackDueDate {
            stack: stack_uid,
            date: Some(due_1),
            reminders: vec![1440],
        }),
    );

    // Screen must remain Screen::Editor (no navigation)
    assert_eq!(model.state.screen, Screen::Editor);

    // Planner sessions in model state must reflect the new session
    assert_eq!(model.state.planner.sessions.len(), 1);
    let session = deadline_of(&model.state.planner.sessions, stack_uid).unwrap();
    assert_eq!(session.kind, EventKind::Deadline);
    assert_eq!(session.stack, Some(stack_uid));
    assert_eq!(session.start.date, due_1);
    assert_eq!(session.reminders, vec![1440]);
    let persisted_id = session.id;
    assert_ne!(persisted_id, Uid(0));

    // Verify true disk persistence by opening a fresh worker on the same vault
    let vault_root = model.state.vault.as_ref().unwrap().root.clone();
    let mut worker_2 = worker_new(None);
    let mut model_2 = Model::default();
    run_to_idle(
        &mut model_2,
        &mut worker_2,
        vec![Effect::OpenVault(vault_root.clone())],
    );
    send(&mut model_2, &mut worker_2, Msg::Navigate(Screen::Calendar));
    assert_eq!(model_2.state.planner.sessions.len(), 1);
    let reloaded = &model_2.state.planner.sessions[0];
    assert_eq!(reloaded.id, persisted_id);
    assert_eq!(reloaded.kind, EventKind::Deadline);
    assert_eq!(reloaded.start.date, due_1);

    // 2. Modify due date on existing session
    let due_2 = Date {
        year: 2026,
        month: 12,
        day: 31,
    };
    send(
        &mut model,
        &mut worker,
        Msg::Plan(PlanMsg::SetStackDueDate {
            stack: stack_uid,
            date: Some(due_2),
            reminders: vec![60, 1440],
        }),
    );
    assert_eq!(model.state.screen, Screen::Editor);
    assert_eq!(model.state.planner.sessions.len(), 1);
    let updated = deadline_of(&model.state.planner.sessions, stack_uid).unwrap();
    assert_eq!(updated.id, persisted_id);
    assert_eq!(updated.start.date, due_2);
    assert_eq!(updated.reminders, vec![60, 1440]);

    // Check persistence of modification in another worker
    let mut worker_3 = worker_new(None);
    let mut model_3 = Model::default();
    run_to_idle(
        &mut model_3,
        &mut worker_3,
        vec![Effect::OpenVault(vault_root.clone())],
    );
    send(&mut model_3, &mut worker_3, Msg::Navigate(Screen::Calendar));
    let reloaded_3 = &model_3.state.planner.sessions[0];
    assert_eq!(reloaded_3.id, persisted_id);
    assert_eq!(reloaded_3.start.date, due_2);
    assert_eq!(reloaded_3.reminders, vec![60, 1440]);

    // 3. Clear due date
    send(
        &mut model,
        &mut worker,
        Msg::Plan(PlanMsg::SetStackDueDate {
            stack: stack_uid,
            date: None,
            reminders: Vec::new(),
        }),
    );
    assert_eq!(model.state.screen, Screen::Editor);
    assert!(model.state.planner.sessions.is_empty());

    // Check persistence of deletion in another worker
    let mut worker_4 = worker_new(None);
    let mut model_4 = Model::default();
    run_to_idle(
        &mut model_4,
        &mut worker_4,
        vec![Effect::OpenVault(vault_root)],
    );
    send(&mut model_4, &mut worker_4, Msg::Navigate(Screen::Calendar));
    assert!(model_4.state.planner.sessions.is_empty());
}
