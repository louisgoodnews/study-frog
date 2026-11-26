#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Headless UI and worker integration tests for the Exam Scheduler (EPIC-076, TASK-0311).

mod common;

use common::flow::{opened, send};
use common::views::{assert_all_named, harness};
use egui_kittest::kittest::Queryable;
use studyfrog_calendar::exam_plan::{ExamPlan, ExamScheduleBlock, ExamTarget, PrepWindow};
use studyfrog_core::civil::{Date, add_days, format_date};
use studyfrog_core::uid::Uid;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::{Msg, MsgVault};
use studyfrog_ui::planner::PlanMsg;
use studyfrog_ui::planner::exam_sched::ExamSchedMsg;
use studyfrog_ui::state::Screen;

fn future_date(today: Date, days: i64) -> String {
    format_date(add_days(today, days))
}

#[test]
fn exam_scheduler_form_toggles_proposed_vs_custom_previews_and_applies() {
    let (_vault, mut model, mut worker) = opened("exam-sched-ui");
    send(
        &mut model,
        &mut worker,
        Msg::Vault(MsgVault::Navigate(Screen::Calendar)),
    );
    let today = model.state.planner.today;
    let stack_folder = model.state.vault.as_ref().unwrap().overview.stacks[0]
        .folder
        .clone();

    let _ = apply(
        &mut model,
        Msg::Plan(PlanMsg::ExamSched(ExamSchedMsg::OpenNew)),
    );
    let _ = apply(
        &mut model,
        Msg::Plan(PlanMsg::ExamSched(ExamSchedMsg::SetTitle(
            "Psychologie Klausur".into(),
        ))),
    );
    let _ = apply(
        &mut model,
        Msg::Plan(PlanMsg::ExamSched(ExamSchedMsg::SetDate(future_date(
            today, 28,
        )))),
    );
    send(
        &mut model,
        &mut worker,
        Msg::Plan(PlanMsg::ExamSched(ExamSchedMsg::ToggleStack(stack_folder))),
    );

    let form = model
        .state
        .planner
        .exam_form
        .as_ref()
        .expect("exam form should be open");
    assert!(
        form.proposed.is_some(),
        "proposed weeks should be estimated"
    );
    assert!(
        form.preview.as_ref().is_some_and(|p| !p.blocks.is_empty()),
        "schedule preview should have generated blocks"
    );

    let mut h = harness(model);
    h.set_size(eframe::egui::vec2(1280.0, 800.0));
    h.run_steps(4);
    assert!(h.query_by_label("StudyFrog proposed").is_some());
    assert!(h.query_by_label("Custom weeks").is_some());
    assert!(h.query_by_label("Apply schedule").is_some());
    assert_all_named(&h, "exam scheduler form");

    // Switch to Custom weeks and apply via worker
    let mut model = h.state().0.clone();
    send(
        &mut model,
        &mut worker,
        Msg::Plan(PlanMsg::ExamSched(ExamSchedMsg::SetUseProposed(false))),
    );
    send(
        &mut model,
        &mut worker,
        Msg::Plan(PlanMsg::ExamSched(ExamSchedMsg::SetCustomWeeks("3".into()))),
    );
    send(
        &mut model,
        &mut worker,
        Msg::Plan(PlanMsg::ExamSched(ExamSchedMsg::Apply)),
    );
    assert!(model.state.planner.exam_form.is_none());
    assert_eq!(model.state.planner.exam_plans.len(), 1);
    assert_eq!(
        model.state.planner.exam_plans[0].title,
        "Psychologie Klausur"
    );
}

#[test]
fn scheduled_exam_blocks_surface_on_calendar_and_home() {
    let (_vault, mut model, mut worker) = opened("exam-sched-home");
    send(
        &mut model,
        &mut worker,
        Msg::Vault(MsgVault::Navigate(Screen::Home)),
    );
    let today = model.state.planner.today;
    let plan = ExamPlan {
        id: Uid(88),
        title: "Diagnostik Final".into(),
        exam_date: add_days(today, 14),
        window: PrepWindow::UserWeeks(2),
        targets: vec![ExamTarget::Stack("french".into())],
        blocks: vec![ExamScheduleBlock {
            date: today,
            item_uids: vec![Uid(1), Uid(2)],
            new_count: 1,
            review_count: 1,
        }],
        created_ms: 0,
        updated_ms: 0,
    };
    let _ = apply(
        &mut model,
        Msg::Plan(PlanMsg::ExamSched(ExamSchedMsg::PlansLoaded(Ok(vec![
            plan,
        ])))),
    );

    let mut h = harness(model);
    h.set_size(eframe::egui::vec2(1280.0, 800.0));
    h.run_steps(4);
    assert!(
        h.query_by_label_contains("Diagnostik Final (2 items)")
            .is_some(),
        "Home Coming up should show today's exam block"
    );
    assert_all_named(&h, "home with exam block");
}
