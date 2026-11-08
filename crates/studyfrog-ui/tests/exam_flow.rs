#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0238 and TASK-0240: Exam in the setup, a whole exam through the real worker, and the Runs report.

mod common;

use common::flow::{opened, send};
use common::run_to_idle;
use common::views::{assert_all_named, harness};
use egui_kittest::kittest::Queryable;
use studyfrog_core::exam::ExamGrade;
use studyfrog_core::plan::{ItemKind, ScheduleEffect};
use studyfrog_core::run::RunMode;
use studyfrog_rehearsal::input::Action;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::explorer_ui::ReportTab;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::setup::{Preset, SetupEdit, SetupForm, SetupStep, apply_edit, plan_from_form};
use studyfrog_ui::state::Screen;

#[test]
fn the_setup_offers_exam_with_its_rules_and_an_exam_plan_never_schedules() {
    let form = [
        SetupEdit::Kind(ItemKind::Note, true),
        SetupEdit::Mode(RunMode::Exam),
    ]
    .into_iter()
    .fold(SetupForm::default(), apply_edit);
    assert!(
        plan_from_form(&form, 0).is_err(),
        "notes alone cannot make an exam"
    );
    let form = apply_edit(form, SetupEdit::Kind(ItemKind::Card, true));
    let plan = plan_from_form(&form, 0).unwrap();
    assert_eq!(
        (plan.filter.kinds, plan.effect, plan.requeue_again),
        (vec![ItemKind::Card], ScheduleEffect::PracticeOnly, false)
    );

    let mut m = common::views::open_model();
    apply(&mut m, Msg::Navigate(Screen::Rehearse));
    apply(&mut m, Msg::SetupStep(SetupStep::How));
    let mut h = harness(m);
    h.run_steps(2);
    h.get_by_label("Exam").click();
    h.run_steps(2);
    assert_eq!(h.state().0.state.rehearse.form.mode, RunMode::Exam);
    assert!(
        h.query_by_label_contains("You cannot skip, go back or see an answer")
            .is_some()
    );
    assert!(
        h.query_by_label("An exam never changes your review schedule.")
            .is_some()
    );
    assert!(
        h.query_by_label("Update my review schedule with these ratings")
            .is_none()
    );
    assert_all_named(&h, "exam setup");
}

#[test]
fn a_whole_exam_runs_through_the_worker_and_lands_in_the_runs_report() {
    let (_vault, mut m, mut w) = opened("exam-flow");
    send(&mut m, &mut w, Msg::Navigate(Screen::Rehearse));
    send(
        &mut m,
        &mut w,
        Msg::Setup(SetupEdit::Preset(Preset::EverythingInOrder)),
    );
    send(&mut m, &mut w, Msg::Setup(SetupEdit::Mode(RunMode::Exam)));
    send(&mut m, &mut w, Msg::StartRunClicked);
    let queue = m.state.rehearse.run.as_ref().unwrap().session.queue.len();
    assert_eq!(queue, 2, "the two cards; the notes are left out");
    for text in ["a", "something else entirely"] {
        send(&mut m, &mut w, Msg::AnswerEdited(text.into()));
        send(
            &mut m,
            &mut w,
            Msg::RunAction(Action::SubmitAnswer(text.into())),
        );
    }
    let run = m.state.rehearse.run.as_ref().unwrap();
    let result = run.exam.result.clone().expect("graded");
    assert_eq!((result.items, result.answered), (2, 2));
    assert_eq!(
        (result.percent, result.grade, result.passed),
        (50.0, ExamGrade::F, false)
    );

    run_to_idle(&mut m, &mut w, vec![Effect::BuildReports]);
    let row = &m.state.reports.as_ref().unwrap().runs[0];
    assert_eq!((row.mode, row.exam.clone()), (RunMode::Exam, Some(result)));
    send(&mut m, &mut w, Msg::LeaveRun);
    m.state.screen = Screen::Reports;
    m.state.explorer.tab = ReportTab::Runs;
    let mut h = harness(m);
    h.run_steps(3);
    h.get_by_label("Exam").click();
    h.run_steps(3);
    assert!(
        h.query_by_label_contains("Exam, 50.0 % (F / 5.0), not passed, 2 of 2 answered")
            .is_some()
    );
}

#[test]
fn the_result_screen_says_grading_until_every_answer_is_compared() {
    let mut m = common::views::running_model(RunMode::Exam);
    for text in ["Paris", "56"] {
        apply(&mut m, Msg::AnswerEdited(text.into()));
        apply(&mut m, Msg::RunAction(Action::SubmitAnswer(text.into())));
    }
    let mut h = harness(m);
    h.run_steps(3);
    assert!(h.query_by_label("Exam finished").is_some());
    assert!(h.query_by_label("Grading…").is_some());
    assert!(h.query_by_label("0 of 2 answers compared").is_some());
    assert_eq!(h.query_all_by_label("comparing…").count(), 2);
}
