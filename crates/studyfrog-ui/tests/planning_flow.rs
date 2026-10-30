#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The calendar and the board through update and the real executor, against a real vault.

mod common;

use common::flow::{opened, send};
use common::run_to_idle;
use studyfrog_core::civil::format_date;
use studyfrog_core::uid::Uid;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{Worker, worker_new};
use studyfrog_ui::model::Model;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::planner::PlanMsg;
use studyfrog_ui::session_form::{EndsKind, FormEdit, RepeatKind};
use studyfrog_ui::state::Screen;

fn plan(model: &mut Model, worker: &mut Worker, m: PlanMsg) {
    send(model, worker, Msg::Plan(m));
}

#[test]
fn a_session_is_planned_repeats_is_skipped_edited_and_deleted() {
    let (_vault, mut model, mut worker) = opened("cal");
    send(&mut model, &mut worker, Msg::Navigate(Screen::Calendar));
    let today = model.state.planner.today;
    assert!(model.state.planner.loaded && model.state.planner.sessions.is_empty());
    assert_eq!(model.state.planner.selected, today);

    plan(&mut model, &mut worker, PlanMsg::New);
    let edits = [
        FormEdit::Title("French drills".into()),
        FormEdit::Stack(Some(Uid(1))),
        FormEdit::Repeat(RepeatKind::Daily),
        FormEdit::Ends(EndsKind::After),
        FormEdit::EndsValue("3".into()),
    ];
    for e in edits {
        plan(&mut model, &mut worker, PlanMsg::Form(e));
    }
    plan(&mut model, &mut worker, PlanMsg::Save);
    let p = &model.state.planner;
    assert!(
        p.form.is_none() && p.form_error.is_none(),
        "{:?}",
        p.form_error
    );
    assert_eq!(p.sessions.len(), 1);
    assert_eq!(p.shown.len(), 3, "three daily occurrences are in the grid");
    let id = p.sessions[0].id;

    plan(&mut model, &mut worker, PlanMsg::Skip(id, today));
    assert_eq!(model.state.planner.shown.len(), 2);

    plan(&mut model, &mut worker, PlanMsg::Edit(id));
    plan(
        &mut model,
        &mut worker,
        PlanMsg::Form(FormEdit::Title("Verbs".into())),
    );
    plan(&mut model, &mut worker, PlanMsg::Save);
    let s = &model.state.planner.sessions[0];
    assert_eq!(
        (s.title.as_str(), s.cancelled.len()),
        ("Verbs", 1),
        "editing keeps the skipped day"
    );

    // A new worker over the same vault still sees it.
    let path = model.state.vault.as_ref().unwrap().root.clone();
    let mut again = worker_new(None);
    let mut fresh = Model::default();
    run_to_idle(&mut fresh, &mut again, vec![Effect::OpenVault(path)]);
    send(&mut fresh, &mut again, Msg::Navigate(Screen::Calendar));
    assert_eq!(fresh.state.planner.sessions.len(), 1);

    plan(&mut model, &mut worker, PlanMsg::Delete(id));
    assert!(model.state.planner.sessions.is_empty() && model.state.planner.shown.is_empty());
}

#[test]
fn a_bad_form_is_explained_and_nothing_is_stored() {
    let (_vault, mut model, mut worker) = opened("cal-bad");
    send(&mut model, &mut worker, Msg::Navigate(Screen::Calendar));
    plan(&mut model, &mut worker, PlanMsg::New);
    plan(
        &mut model,
        &mut worker,
        PlanMsg::Form(FormEdit::Date("soon".into())),
    );
    plan(&mut model, &mut worker, PlanMsg::Save);
    let p = &model.state.planner;
    assert!(p.form_error.as_deref().unwrap().contains("date"));
    assert!(
        p.form.is_some(),
        "the form stays open so nothing typed is lost"
    );
    let today = p.today;

    plan(
        &mut model,
        &mut worker,
        PlanMsg::Form(FormEdit::Date(format_date(today))),
    );
    plan(&mut model, &mut worker, PlanMsg::Save);
    let e = model.state.planner.form_error.clone().unwrap();
    assert!(
        e.contains("title"),
        "the service's validation reaches the form: {e}"
    );
    assert!(model.state.planner.sessions.is_empty());
}

#[test]
fn starting_a_rehearsal_from_a_session_filters_to_its_stack() {
    let (_vault, mut model, mut worker) = opened("cal-run");
    send(&mut model, &mut worker, Msg::Navigate(Screen::Calendar));
    plan(&mut model, &mut worker, PlanMsg::New);
    plan(
        &mut model,
        &mut worker,
        PlanMsg::Form(FormEdit::Title("Drill".into())),
    );
    plan(
        &mut model,
        &mut worker,
        PlanMsg::Form(FormEdit::Stack(Some(Uid(1)))),
    );
    plan(&mut model, &mut worker, PlanMsg::Save);
    let id = model.state.planner.sessions[0].id;
    plan(&mut model, &mut worker, PlanMsg::Rehearse(id));
    assert_eq!(model.state.screen, Screen::Rehearse);
    assert_eq!(model.state.rehearse.form.stacks, [Uid(1)]);
}
