#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Pacing in the run setup and the clock on the rehearsal screen: presets, warnings, the end of a countdown, count up.

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::pacing::Pacing;
use studyfrog_core::plan::plan_spaced;
use studyfrog_core::review::Rating;
use studyfrog_core::run::{Run, RunMode, RunState};
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::input::Action;
use studyfrog_service::content::ItemContent;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::run_clock::{Warning, clock_of, mmss};
use studyfrog_ui::setup::{SetupEdit, SetupStep};
use studyfrog_ui::setup_form::plan_from_form;
use studyfrog_ui::setup_pacing::{PacingEdit, PacingKind};
use studyfrog_ui::state::Screen;

fn pacing(m: &mut Model, e: PacingEdit) {
    apply(m, Msg::Setup(SetupEdit::Pacing(e)));
}

fn running(p: Pacing) -> Model {
    let mut plan = plan_spaced();
    plan.pacing = p;
    let run = Run {
        uid: Uid(100),
        mode: RunMode::Flip,
        plan,
        queue: vec![Uid(1), Uid(2)],
        state: RunState::Idle,
        started_at: None,
    };
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Rehearse));
    m.state.now = 0.0;
    apply(&mut m, Msg::RunReady(Ok(run)));
    for (uid, prompt, answer) in [(1, "Capital of France?", "Paris"), (2, "7 x 8?", "56")] {
        let content = ItemContent {
            title: prompt.into(),
            prompt: prompt.into(),
            answer: answer.into(),
            previous: None,
            source: None,
            color: None,
            expected: None,
            occlusion: None,
        };
        apply(&mut m, Msg::ItemLoaded(Uid(uid), Ok(content)));
    }
    m
}

fn clock(m: &Model) -> Option<(String, Option<Warning>)> {
    clock_of(&m.state, m.state.rehearse.run.as_ref().unwrap()).map(|c| (c.text, c.warning))
}

#[test]
fn the_setup_builds_the_pacing_of_the_plan_and_refuses_a_bad_countdown() {
    let mut m = open_model();
    let plan = |m: &Model| plan_from_form(&m.state.rehearse.form, 1);
    assert_eq!(plan(&m).unwrap().pacing, Pacing::Free);
    pacing(&mut m, PacingEdit::Kind(PacingKind::CountUp));
    assert_eq!(plan(&m).unwrap().pacing, Pacing::CountUp);
    pacing(&mut m, PacingEdit::Kind(PacingKind::Countdown));
    assert_eq!(
        plan(&m).unwrap().pacing,
        Pacing::Countdown(600),
        "ten minutes by default"
    );
    pacing(&mut m, PacingEdit::Minutes("25".into()));
    assert_eq!(plan(&m).unwrap().pacing, Pacing::Countdown(1500));
    for bad in ["0", "181", "x", "", "-5"] {
        pacing(&mut m, PacingEdit::Minutes(bad.into()));
        assert!(plan(&m).is_err(), "{bad:?}");
    }
    pacing(&mut m, PacingEdit::Minutes("180".into()));
    assert_eq!(plan(&m).unwrap().pacing, Pacing::Countdown(10_800));
}

#[test]
fn the_how_step_offers_the_modes_and_the_presets_and_a_custom_time() {
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Rehearse));
    apply(&mut m, Msg::SetupStep(SetupStep::How));
    let mut h = harness(m);
    h.run_steps(4);
    for t in ["No clock", "Countdown", "Count up"] {
        assert!(h.query_by_label(t).is_some(), "{t}");
    }
    h.get_by_label("Countdown").click();
    h.run_steps(3);
    for t in [
        "5 minutes",
        "10 minutes",
        "15 minutes",
        "25 minutes",
        "Countdown minutes (1-180)",
    ] {
        assert!(h.query_by_label(t).is_some(), "{t}");
    }
    h.get_by_label("15 minutes").click();
    h.run_steps(3);
    assert_eq!(h.state().0.state.rehearse.form.pacing.minutes, "15");
    assert_all_named(&h, "setup with a countdown");
}

#[test]
fn a_countdown_shows_the_time_left_warns_in_text_and_ends_by_itself_without_rating_the_item_on_screen()
 {
    let mut m = running(Pacing::Countdown(120));
    assert_eq!(clock(&m), Some(("Time left 02:00".into(), None)));
    m.state.now = 30.0;
    assert_eq!(clock(&m).unwrap().0, "Time left 01:30");
    m.state.now = 70.0;
    assert_eq!(
        clock(&m),
        Some(("Time left 00:50".into(), Some(Warning::OneMinute)))
    );
    m.state.now = 111.0;
    assert_eq!(
        clock(&m),
        Some(("Time left 00:09".into(), Some(Warning::TenSeconds)))
    );
    let mut h = harness(running(Pacing::Countdown(60)));
    h.run_steps(3);
    assert!(h.query_by_label_contains("Time left").is_some());
    assert!(
        h.query_by_label("One minute left").is_some(),
        "a minute or less is announced in words"
    );
    assert_all_named(&h, "run with a warning");

    let mut m = running(Pacing::Countdown(120));
    m.state.now = 10.0;
    apply(&mut m, Msg::RunAction(Action::Flip));
    m.state.now = 119.0;
    assert!(
        apply(&mut m, Msg::Tick)
            .iter()
            .all(|e| !matches!(e, Effect::RecordStep { .. })),
        "time is not up yet"
    );
    m.state.now = 120.5;
    let fx = apply(&mut m, Msg::Tick);
    let run = m.state.rehearse.run.as_ref().unwrap();
    assert_eq!(
        run.session.run.state,
        RunState::Finished,
        "the tick ended the run"
    );
    assert!(fx.iter().any(|e| matches!(e, Effect::RecordStep { .. })));
    assert!(
        run.events
            .iter()
            .any(|e| format!("{:?}", e.kind).contains("TimeUp"))
    );
    assert!(
        !run.events
            .iter()
            .any(|e| format!("{:?}", e.kind).contains("Rated")),
        "the item on screen was not rated"
    );
    let late = apply(&mut m, Msg::RunAction(Action::Rate(Rating::Good)));
    assert!(
        late.iter().all(|e| !matches!(e, Effect::RecordStep { .. })),
        "no rating after the end"
    );
}

#[test]
fn a_count_up_shows_the_time_used_and_a_free_run_shows_no_clock_and_a_pause_stops_the_clock() {
    let mut m = running(Pacing::CountUp);
    m.state.now = 65.0;
    assert_eq!(clock(&m), Some(("Time used 01:05".into(), None)));
    assert_eq!(clock(&running(Pacing::Free)), None);
    apply(&mut m, Msg::RunAction(Action::Pause));
    m.state.now = 500.0;
    assert_eq!(
        clock(&m).unwrap().0,
        "Time used 01:05",
        "a paused clock stands still"
    );
    assert_eq!(mmss(3_599_000), "59:59");
}

#[test]
fn the_review_after_a_countdown_says_the_planned_and_the_used_time() {
    let mut m = running(Pacing::Countdown(120));
    m.state.now = 10.0;
    apply(&mut m, Msg::RunAction(Action::Flip));
    m.state.now = 20.0;
    apply(&mut m, Msg::RunAction(Action::Rate(Rating::Good)));
    m.state.now = 121.0;
    apply(&mut m, Msg::Tick);
    assert_eq!(
        m.state.rehearse.run.as_ref().unwrap().session.run.state,
        RunState::Finished
    );
    let mut h = harness(m);
    h.run_steps(4);
    assert!(
        h.query_all_by_label_contains("Countdown of 02:00, used")
            .count()
            >= 1
    );
    assert!(h.query_all_by_label_contains("answers a minute").count() >= 1);
}
