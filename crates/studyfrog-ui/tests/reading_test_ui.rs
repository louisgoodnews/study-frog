#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The reading-speed test and its use as the baseline of the review.

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::plan::plan_spaced;
use studyfrog_core::run::{Run, RunMode, RunState};
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::reading::word_count;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::reading_test::{ReadingTestMsg, TEXT, TestPhase};
use studyfrog_ui::state::Screen;

fn test(m: &mut Model, now: f64, msg: ReadingTestMsg) -> Vec<Effect> {
    m.state.now = now;
    apply(m, Msg::ReadingTest(msg))
}

#[test]
fn the_test_text_is_long_enough_to_measure_a_speed() {
    let words = word_count(TEXT);
    assert!((100..=160).contains(&words), "{words} words");
}

#[test]
fn reading_the_text_in_a_plausible_time_gives_a_speed_that_can_be_kept() {
    let mut m = open_model();
    let words = word_count(TEXT) as u32;
    test(&mut m, 10.0, ReadingTestMsg::Start);
    assert_eq!(m.state.reading_test.phase, TestPhase::Reading(10.0));
    test(&mut m, 40.0, ReadingTestMsg::Finish);
    assert_eq!(
        m.state.reading_test.phase,
        TestPhase::Result(Some(words * 2))
    );

    let effects = test(&mut m, 41.0, ReadingTestMsg::Save);
    assert_eq!(m.state.app.reading_wpm, Some(words * 2));
    assert!(
        matches!(&effects[..], [Effect::SaveAppSettings(a)] if a.reading_wpm == Some(words * 2))
    );
    assert_eq!(m.state.reading_test.phase, TestPhase::Idle);
    assert!(m.state.notice.as_ref().unwrap().text.contains("saved"));

    let effects = test(&mut m, 50.0, ReadingTestMsg::Reset);
    assert_eq!(m.state.app.reading_wpm, None);
    assert!(matches!(&effects[..], [Effect::SaveAppSettings(a)] if a.reading_wpm.is_none()));
}

#[test]
fn an_implausible_time_is_not_kept_and_stray_messages_do_nothing() {
    let mut m = open_model();
    test(&mut m, 0.0, ReadingTestMsg::Start);
    test(&mut m, 1.0, ReadingTestMsg::Finish);
    assert_eq!(m.state.reading_test.phase, TestPhase::Result(None));
    assert!(
        test(&mut m, 2.0, ReadingTestMsg::Save).is_empty(),
        "nothing to save"
    );
    assert_eq!(m.state.app.reading_wpm, None);

    let mut fresh = open_model();
    assert!(test(&mut fresh, 5.0, ReadingTestMsg::Finish).is_empty());
    assert_eq!(
        fresh.state.reading_test.phase,
        TestPhase::Idle,
        "finishing without starting"
    );
    test(&mut fresh, 6.0, ReadingTestMsg::Start);
    test(&mut fresh, 7.0, ReadingTestMsg::Discard);
    assert_eq!(fresh.state.reading_test.phase, TestPhase::Idle);
}

#[test]
fn the_settings_screen_walks_through_the_test() {
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Settings));
    let mut h = harness(m);
    h.run();
    assert!(
        h.query_by_label("238 words per minute (default, not measured)")
            .is_some()
    );
    h.get_by_label("Measure my reading speed").click();
    h.run();
    assert!(h.query_by_label("I have finished reading").is_some());
    assert_all_named(&h, "reading test");

    let (model, _) = h.state_mut();
    test(model, 100.0, ReadingTestMsg::Start);
    test(model, 130.0, ReadingTestMsg::Finish);
    h.run();
    let wpm = word_count(TEXT) * 2;
    assert!(
        h.query_by_label(&format!("You read {wpm} words per minute."))
            .is_some()
    );
    h.get_by_label("Use this speed").click();
    h.run();
    assert!(
        h.state()
            .1
            .iter()
            .any(|e| matches!(e, Effect::SaveAppSettings(a) if a.reading_wpm == Some(wpm as u32)))
    );
    assert!(
        h.query_by_label(&format!("{wpm} words per minute (measured)"))
            .is_some()
    );
}

#[test]
fn a_run_uses_the_speed_measured_when_it_started() {
    let mut m = open_model();
    m.state.app.reading_wpm = Some(120);
    m.state.screen = Screen::Rehearse;
    let run = Run {
        uid: Uid(100),
        mode: RunMode::Flip,
        plan: plan_spaced(),
        queue: vec![Uid(1)],
        state: RunState::Idle,
        started_at: None,
    };
    apply(&mut m, Msg::RunReady(Ok(run)));
    assert_eq!(
        m.state
            .rehearse
            .run
            .as_ref()
            .unwrap()
            .reading
            .words_per_minute,
        120
    );
}
