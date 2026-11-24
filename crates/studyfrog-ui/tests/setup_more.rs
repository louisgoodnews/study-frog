#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Run setup: form validation, presets, preview flow against a real vault, and the real widgets.
use common::views::{harness, open_model};
use common::{run_to_idle, sample_vault};
use egui_kittest::kittest::Queryable;
use studyfrog_core::plan::{ItemKind, Order, ScheduleEffect, StateFilter};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::effect::EffectRehearsal;
use studyfrog_ui::exec::worker_new;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::msg::MsgRehearsal;
use studyfrog_ui::msg::MsgVault;
use studyfrog_ui::setup::{Preset, Preview, SetupEdit};
use studyfrog_ui::state::Screen;

mod common;

#[test]
fn the_count_comes_from_the_real_index_and_follows_the_filters() {
    let vault = sample_vault("setup"); // French: 2 cards + 1 note, plus 1 loose note
    let mut worker = worker_new(None);
    let mut m = Model::default();
    let fx = apply(
        &mut m,
        Msg::Vault(MsgVault::OpenRecent(vault.path.display().to_string())),
    );
    run_to_idle(&mut m, &mut worker, fx);

    let count = |m: &mut Model, worker: &mut _, edit: SetupEdit| {
        let fx = apply(m, Msg::Rehearsal(MsgRehearsal::Setup(edit)));
        run_to_idle(m, worker, fx);
        m.state.rehearse.preview.clone()
    };
    let fx = apply(&mut m, Msg::Vault(MsgVault::Navigate(Screen::Rehearse)));
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
    apply(&mut m, Msg::Vault(MsgVault::Navigate(Screen::Rehearse)));
    apply(&mut m, Msg::Rehearsal(MsgRehearsal::PreviewReady(Ok(7))));
    let mut h = harness(m);
    h.run();
    assert!(h.query_by_label("7 items match").is_some());
    h.get_by_label("Cards").click();
    h.run();
    h.get_by_label("Next: configure").click();
    h.run();
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
            model.state.rehearse.form.shuffle,
            model.state.rehearse.form.effect
        ),
        (Order::Sequential, true, ScheduleEffect::PracticeOnly)
    );
    assert!(
        h.query_by_label_contains("Practice only").is_some(),
        "practice-only is clearly labelled"
    );
    assert!(
        h.state()
            .1
            .iter()
            .any(|e| matches!(e, Effect::Rehearsal(EffectRehearsal::PreviewQueue { .. })))
    );

    h.get_by_label("Maximum number of items").focus();
    h.get_by_label("Maximum number of items").type_text("5");
    h.run();
    let f = &h.state().0.state.rehearse.form;
    assert_eq!(
        (f.kinds.as_slice(), f.limit.as_str()),
        ([ItemKind::Card].as_slice(), "5")
    );
}
