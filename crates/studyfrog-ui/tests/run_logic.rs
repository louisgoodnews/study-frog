#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! A rehearsal run: pure update logic, and the whole flow against a real vault and database.

mod common;

use common::{run_to_idle, sample_vault};
use studyfrog_core::plan::ScheduleEffect;
use studyfrog_core::review::Rating::{Again, Good};
use studyfrog_core::run::RunState;
use studyfrog_rehearsal::input::Action;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{Worker, worker_new};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::run_update::{awaiting_rating, awaiting_reveal, is_over};
use studyfrog_ui::setup::{Preset, SetupEdit};
use studyfrog_ui::state::Screen;

fn count(worker: &Worker, sql: &str) -> i64 {
    worker
        .session
        .as_ref()
        .unwrap()
        .conn
        .query_row(sql, [], |r| r.get(0))
        .unwrap()
}

/// Open the sample vault (2 cards, 2 notes) and go to the rehearse screen with a preset.
fn ready(label: &str, preset: Preset) -> (Model, Worker, common::TempDir) {
    let vault = sample_vault(label);
    let (mut model, mut worker) = (Model::default(), worker_new(None));
    let fx = apply(
        &mut model,
        Msg::OpenRecent(vault.path.display().to_string()),
    );
    run_to_idle(&mut model, &mut worker, fx);
    let fx = apply(&mut model, Msg::Navigate(Screen::Rehearse));
    run_to_idle(&mut model, &mut worker, fx);
    let fx = apply(&mut model, Msg::Setup(SetupEdit::Preset(preset)));
    run_to_idle(&mut model, &mut worker, fx);
    (model, worker, vault)
}

/// Press a key at engine time `now` (seconds on the shell clock) and settle the effects.
fn press(model: &mut Model, worker: &mut Worker, now: f64, action: Action) {
    model.state.now = now;
    let fx = apply(model, Msg::RunAction(action));
    run_to_idle(model, worker, fx);
}

fn start(model: &mut Model, worker: &mut Worker) {
    model.state.now = 100.0;
    let fx = apply(model, Msg::StartRunClicked);
    assert!(matches!(&fx[..], [Effect::StartRun { .. }]));
    run_to_idle(model, worker, fx);
}

#[test]
fn a_spaced_run_is_driven_by_keys_and_saved_into_the_schedule() {
    let (mut m, mut w, _vault) = ready("spaced", Preset::Spaced);
    assert_eq!(
        m.state.rehearse.preview,
        studyfrog_ui::setup::Preview::Count(4)
    );
    start(&mut m, &mut w);

    let run = m.state.rehearse.run.as_ref().unwrap();
    assert_eq!(run.session.run.state, RunState::Running);
    assert!(awaiting_reveal(run) && run.session.queue.len() == 4);
    assert!(
        run.content.len() >= 3,
        "the current item and a look-ahead were loaded"
    );

    let mut t = 100.0;
    for rating in [Good, Again, Good, Good, Good] {
        t += 1.5;
        press(&mut m, &mut w, t, Action::Flip);
        assert!(awaiting_rating(m.state.rehearse.run.as_ref().unwrap()));
        t += 0.5;
        press(&mut m, &mut w, t, Action::Rate(rating));
    }
    let run = m.state.rehearse.run.as_ref().unwrap();
    assert!(is_over(run), "4 items plus the one requeued Again");
    assert_eq!(run.session.run.state, RunState::Finished);
    assert_eq!(
        count(&w, "SELECT COUNT(*) FROM review"),
        5,
        "every rating is a review"
    );
    assert_eq!(count(&w, "SELECT COUNT(*) FROM mem_state"), 4);
    assert_eq!(
        count(&w, "SELECT COUNT(*) FROM run WHERE state = 'finished'"),
        1
    );
    assert_eq!(
        count(
            &w,
            "SELECT COUNT(*) FROM run_event WHERE kind = 'item_rated'"
        ),
        5
    );

    let summary = studyfrog_ui::summary::summarize(&run.events, &run.latencies);
    assert_eq!(summary.rated, [1, 0, 4, 0]);
    assert_eq!(
        summary.median_latency_ms,
        Some(1500),
        "latency is engine time between show and reveal"
    );
}

#[test]
fn a_practice_run_records_everything_except_schedule_changes() {
    let (mut m, mut w, _vault) = ready("practice", Preset::EverythingInOrder);
    assert_eq!(m.state.rehearse.form.effect, ScheduleEffect::PracticeOnly);
    start(&mut m, &mut w);
    let mut t = 100.0;
    for _ in 0..4 {
        t += 1.0;
        press(&mut m, &mut w, t, Action::Flip);
        t += 1.0;
        press(&mut m, &mut w, t, Action::Rate(Good));
    }
    assert!(is_over(m.state.rehearse.run.as_ref().unwrap()));
    assert_eq!(count(&w, "SELECT COUNT(*) FROM review"), 0);
    assert_eq!(count(&w, "SELECT COUNT(*) FROM mem_state"), 0);
    assert_eq!(
        count(
            &w,
            "SELECT COUNT(*) FROM run_event WHERE kind = 'item_flipped'"
        ),
        4,
        "events are still kept"
    );
}

#[test]
fn stray_keys_are_ignored_and_pausing_stops_the_clock() {
    let (mut m, mut w, _vault) = ready("stray", Preset::EverythingInOrder);
    start(&mut m, &mut w);
    press(&mut m, &mut w, 101.0, Action::Rate(Good)); // not revealed yet
    press(&mut m, &mut w, 101.5, Action::Resume); // not paused
    assert_eq!(m.state.rehearse.run.as_ref().unwrap().latencies.len(), 0);
    press(&mut m, &mut w, 102.0, Action::Pause);
    press(&mut m, &mut w, 110.0, Action::Resume);
    press(&mut m, &mut w, 111.0, Action::Flip);
    let l = m.state.rehearse.run.as_ref().unwrap().latencies[0];
    assert_eq!(l.ms, 3000, "11 s on screen minus 8 s paused");
}

#[test]
fn stopping_aborts_the_run_and_leaving_returns_to_setup_with_a_fresh_count() {
    let (mut m, mut w, _vault) = ready("stop", Preset::Shuffled);
    start(&mut m, &mut w);
    press(&mut m, &mut w, 101.0, Action::Abort);
    let run = m.state.rehearse.run.as_ref().unwrap();
    assert_eq!(run.session.run.state, RunState::Aborted);
    assert_eq!(
        count(&w, "SELECT COUNT(*) FROM run WHERE state = 'aborted'"),
        1
    );

    let fx = apply(&mut m, Msg::LeaveRun);
    assert!(m.state.rehearse.run.is_none() && matches!(&fx[..], [Effect::PreviewQueue { .. }]));
    assert!(
        apply(&mut m, Msg::LeaveRun).is_empty(),
        "nothing to leave any more"
    );
}

#[test]
fn a_run_cannot_be_started_twice_or_with_an_invalid_form_or_no_items() {
    let (mut m, mut w, _vault) = ready("guard", Preset::Spaced);
    apply(&mut m, Msg::Setup(SetupEdit::Limit("x".into())));
    let fx = apply(&mut m, Msg::StartRunClicked);
    assert!(
        fx.iter().all(|e| matches!(e, Effect::Notifications(_))),
        "an invalid form does not start: {fx:?}"
    );
    assert!(m.state.notice.as_ref().unwrap().text.contains("limit"));
    let fx = apply(&mut m, Msg::Setup(SetupEdit::Limit("".into())));
    run_to_idle(&mut m, &mut w, fx);
    start(&mut m, &mut w);
    assert!(
        apply(&mut m, Msg::StartRunClicked).is_empty(),
        "a run is already on screen"
    );
}

#[test]
fn an_item_that_cannot_be_loaded_is_shown_as_a_problem_and_the_run_continues() {
    let (mut m, mut w, vault) = ready("missing", Preset::EverythingInOrder);
    std::fs::remove_file(vault.path.join("french/a.md")).unwrap();
    start(&mut m, &mut w);
    assert!(
        m.state
            .notice
            .as_ref()
            .unwrap()
            .text
            .contains("Could not load")
    );
    let run = m.state.rehearse.run.as_ref().unwrap();
    let first = studyfrog_rehearsal::session::current_item(&run.session).unwrap();
    assert!(run.content[&first].prompt.contains("could not be loaded"));
    press(&mut m, &mut w, 101.0, Action::Skip);
    assert!(
        awaiting_reveal(m.state.rehearse.run.as_ref().unwrap()),
        "on to the next item"
    );
}
