#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Run setup: form validation, presets, preview flow against a real vault, and the real widgets.

mod common;

use common::views::{harness, open_model};
use common::{run_to_idle, sample_vault};
use egui_kittest::kittest::Queryable;
use studyfrog_core::plan::{ItemKind, Order, ScheduleEffect, StateFilter};
use studyfrog_core::run::RunMode;
use studyfrog_core::uid::Uid;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::worker_new;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::setup::{
    Preset, Preview, SetupEdit, SetupForm, apply_edit, form_for_preset, plan_from_form,
};
use studyfrog_ui::state::Screen;

fn form(edits: Vec<SetupEdit>) -> SetupForm {
    edits.into_iter().fold(SetupForm::default(), apply_edit)
}

#[test]
fn presets_follow_the_documented_defaults() {
    let spaced = plan_from_form(&form_for_preset(Preset::Spaced, RunMode::Flip), 0).unwrap();
    assert_eq!(
        (spaced.order, spaced.effect, spaced.requeue_again),
        (Order::Due, ScheduleEffect::UpdateSchedule, true)
    );
    for p in [
        Preset::EverythingInOrder,
        Preset::Shuffled,
        Preset::Hardest,
        Preset::Stalest,
    ] {
        let plan = plan_from_form(&form_for_preset(p, RunMode::Answer), 7).unwrap();
        assert_eq!(
            plan.effect,
            ScheduleEffect::PracticeOnly,
            "{p:?} is practice only"
        );
        assert_eq!(plan.seed, 7);
    }
    assert_eq!(
        form_for_preset(Preset::Shuffled, RunMode::Answer).mode,
        RunMode::Answer
    );
    assert_eq!(
        apply_edit(SetupForm::default(), SetupEdit::Preset(Preset::Hardest)).order,
        Order::HardestFirst
    );
}

#[test]
fn edits_build_the_expected_plan() {
    let f = form(vec![
        SetupEdit::Order(Order::Shuffled),
        SetupEdit::Limit(" 25 ".into()),
        SetupEdit::Kind(ItemKind::Card, true),
        SetupEdit::Kind(ItemKind::Note, true),
        SetupEdit::Kind(ItemKind::Note, false),
        SetupEdit::Stack(Uid(5), true),
        SetupEdit::Tags(" verbs, ,Grammar ".into()),
        SetupEdit::State(StateFilter::Learned),
        SetupEdit::DifficultyOn(true),
        SetupEdit::DiffMin("4.5".into()),
        SetupEdit::Effect(ScheduleEffect::PracticeOnly),
        SetupEdit::Requeue(false),
    ]);
    let plan = plan_from_form(&f, 99).unwrap();
    assert_eq!(
        (plan.order, plan.limit, plan.seed, plan.requeue_again),
        (Order::Shuffled, Some(25), 99, false)
    );
    assert_eq!(plan.filter.kinds, [ItemKind::Card]);
    assert_eq!(plan.filter.stacks, [Uid(5)]);
    assert_eq!(plan.filter.tags_any, ["verbs", "Grammar"]);
    assert_eq!(plan.filter.state, StateFilter::Learned);
    let range = plan.filter.difficulty.unwrap();
    assert_eq!((range.min, range.max), (4.5, 10.0));
}

#[test]
fn invalid_input_is_explained_not_ignored() {
    let why = |e: SetupEdit| plan_from_form(&form(vec![e]), 0).unwrap_err();
    assert!(why(SetupEdit::Limit("many".into())).contains("limit"));
    assert!(why(SetupEdit::Limit("0".into())).contains("at least 1"));
    let diff = |min: &str, max: &str| {
        plan_from_form(
            &form(vec![
                SetupEdit::DifficultyOn(true),
                SetupEdit::DiffMin(min.into()),
                SetupEdit::DiffMax(max.into()),
            ]),
            0,
        )
    };
    assert!(diff("x", "5").unwrap_err().contains("number"));
    assert!(diff("0", "5").unwrap_err().contains("1 to 10"));
    assert!(diff("8", "3").unwrap_err().contains("lowest first"));
    assert!(diff("2", "8").is_ok());
    assert!(
        plan_from_form(&form(vec![SetupEdit::Limit("".into())]), 0)
            .unwrap()
            .limit
            .is_none()
    );
}

#[test]
fn entering_the_screen_and_every_edit_ask_for_a_fresh_count() {
    let mut m = open_model();
    let fx = apply(&mut m, Msg::Navigate(Screen::Rehearse));
    assert!(matches!(&fx[..], [Effect::PreviewQueue { .. }]));
    assert_eq!(m.state.rehearse.preview, Preview::Counting);
    let fx = apply(&mut m, Msg::Setup(SetupEdit::Order(Order::Shuffled)));
    assert!(
        matches!(&fx[..], [Effect::PreviewQueue { plan, .. }] if plan.order == Order::Shuffled)
    );

    let fx = apply(&mut m, Msg::Setup(SetupEdit::Limit("abc".into())));
    assert!(fx.is_empty(), "an invalid form asks for nothing");
    assert!(matches!(m.state.rehearse.preview, Preview::Invalid(_)));
    apply(&mut m, Msg::PreviewReady(Ok(12)));
    assert_eq!(m.state.rehearse.preview, Preview::Count(12));
    apply(&mut m, Msg::PreviewReady(Err("db".into())));
    assert!(matches!(m.state.rehearse.preview, Preview::Failed(_)));
}

#[test]
fn the_count_comes_from_the_real_index_and_follows_the_filters() {
    let vault = sample_vault("setup"); // French: 2 cards + 1 note, plus 1 loose note
    let mut worker = worker_new(None);
    let mut m = Model::default();
    let fx = apply(&mut m, Msg::OpenRecent(vault.path.display().to_string()));
    run_to_idle(&mut m, &mut worker, fx);

    let count = |m: &mut Model, worker: &mut _, edit: SetupEdit| {
        let fx = apply(m, Msg::Setup(edit));
        run_to_idle(m, worker, fx);
        m.state.rehearse.preview.clone()
    };
    let fx = apply(&mut m, Msg::Navigate(Screen::Rehearse));
    run_to_idle(&mut m, &mut worker, fx);
    assert_eq!(
        m.state.rehearse.preview,
        Preview::Count(4),
        "spaced: all four are new"
    );

    assert_eq!(
        count(
            &mut m,
            &mut worker,
            SetupEdit::Preset(Preset::EverythingInOrder)
        ),
        Preview::Count(4)
    );
    assert_eq!(
        count(&mut m, &mut worker, SetupEdit::Kind(ItemKind::Card, true)),
        Preview::Count(2)
    );
    assert_eq!(
        count(&mut m, &mut worker, SetupEdit::Limit("1".into())),
        Preview::Count(1)
    );
    assert_eq!(
        count(&mut m, &mut worker, SetupEdit::Limit("".into())),
        Preview::Count(2)
    );
    assert_eq!(
        count(&mut m, &mut worker, SetupEdit::State(StateFilter::Due)),
        Preview::Count(0),
        "nothing is reviewed yet"
    );
}

#[test]
fn the_real_widgets_send_edits_and_show_the_count_and_practice_note() {
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Rehearse));
    apply(&mut m, Msg::PreviewReady(Ok(7)));
    let mut h = harness(m);
    h.run();
    assert!(h.query_by_label("7 items match").is_some());
    assert!(
        h.query_by_label_contains("Practice only").is_none(),
        "spaced updates the schedule"
    );

    h.get_by_label("Everything, shuffled").click();
    h.run();
    let model = &h.state().0;
    assert_eq!(
        (
            model.state.rehearse.form.order,
            model.state.rehearse.form.effect
        ),
        (Order::Shuffled, ScheduleEffect::PracticeOnly)
    );
    assert!(
        h.query_by_label_contains("Practice only").is_some(),
        "practice-only is clearly labelled"
    );
    assert!(
        h.state()
            .1
            .iter()
            .any(|e| matches!(e, Effect::PreviewQueue { .. }))
    );

    h.get_by_label("Cards").click();
    h.get_by_label("Maximum number of items").focus();
    h.get_by_label("Maximum number of items").type_text("5");
    h.run();
    let f = &h.state().0.state.rehearse.form;
    assert_eq!(
        (f.kinds.as_slice(), f.limit.as_str()),
        ([ItemKind::Card].as_slice(), "5")
    );
}
