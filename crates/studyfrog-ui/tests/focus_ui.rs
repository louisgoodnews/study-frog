#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The focus timer in the top bar, across screens, its completion card and its record (STORY-0112).

mod common;

use common::flow::{opened, send};
use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::focus_ui::{FocusMsg, msg};
use studyfrog_ui::library_bulk::LibraryEffect;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::recall_types::RecallEffect;
use studyfrog_ui::state::Screen;

fn running() -> Model {
    let mut m = open_model();
    m.state.now = 100.0;
    m.state.wall_ms = 1_794_096_000_000;
    apply(&mut m, msg(FocusMsg::Stack(Some("french".into()))));
    apply(&mut m, msg(FocusMsg::Start));
    m
}

#[test]
fn the_top_bar_starts_the_timer_from_its_menu() {
    let mut h = harness(open_model());
    h.run_steps(2);
    h.get_by_label("Focus timer").click();
    h.run_steps(2);
    for label in [
        "Work and breaks",
        "25 / 5",
        "50 / 10",
        "90 / 15",
        "Start focusing",
    ] {
        assert!(h.query_by_label(label).is_some(), "missing {label}");
    }
    assert_all_named(&h, "focus menu");
    h.get_by_label("Start focusing").click();
    h.run_steps(2);
    assert!(h.state().0.state.recall.focus.since.is_some());
}

#[test]
fn the_timer_keeps_running_on_every_screen() {
    let screens = [
        Screen::Pdf,
        Screen::Editor,
        Screen::Calendar,
        Screen::Rehearse,
        Screen::Library,
    ];
    let mut m = running();
    for screen in screens {
        apply(&mut m, Msg::Navigate(screen));
        let mut h = harness(m);
        h.run_steps(1);
        let shown = h.query_all_by_label_contains("Focus 2").count();
        assert!(shown >= 1, "the pill shows on {screen:?}");
        m = std::mem::take(&mut h.state_mut().0);
    }
    let mut m = running();
    for screen in screens {
        apply(&mut m, Msg::Navigate(screen));
        m.state.now += 60.0;
        apply(&mut m, Msg::Tick);
    }
    assert!(m.state.recall.focus.since.is_some(), "still running");
    apply(&mut m, msg(FocusMsg::Pause));
    let left = studyfrog_ui::focus_timer::remaining(&m.state.recall.focus, m.state.now + 500.0);
    assert_eq!(left, 25 * 60 - 300, "paused: the time stands still");
}

#[test]
fn a_finished_block_is_recorded_and_offers_the_break_and_the_diary() {
    let mut m = running();
    m.state.now += 1500.0;
    m.state.wall_ms += 1_500_000;
    let fx = apply(&mut m, Msg::Tick);
    let logged = fx.iter().any(|e| {
        matches!(e,
        Effect::Library(LibraryEffect::Recall(RecallEffect::LogFocus(d)))
            if d.minutes == 25 && d.stack.as_deref() == Some("french"))
    });
    assert!(logged, "{fx:?}");
    let mut h = harness(m);
    h.run_steps(2);
    for label in [
        "Focus session complete (25 min)",
        "Start break (5 min)",
        "Log in Diary…",
    ] {
        assert!(h.query_by_label(label).is_some(), "missing {label}");
    }
    h.get_by_label("Log in Diary…").click();
    h.run_steps(2);
    let s = &h.state().0.state;
    assert_eq!(s.screen, Screen::Diary);
    let draft = s.diary.active_entry.as_ref().unwrap();
    assert_eq!(
        (draft.study_minutes, draft.stacks.as_str()),
        (Some(25), "french")
    );
    let mut m = running();
    m.state.now += 1500.0;
    apply(&mut m, Msg::Tick);
    apply(&mut m, msg(FocusMsg::StartBreak));
    assert!(m.state.recall.focus.done.is_none());
    assert!(m.state.recall.focus.since.is_some(), "the break runs");
    apply(&mut m, msg(FocusMsg::SkipBreak));
    assert_eq!(
        m.state.recall.focus.phase,
        studyfrog_ui::focus_timer::Phase::Work
    );
}

#[test]
fn the_worker_records_the_block_and_says_so() {
    let (_dir, mut m, mut w) = opened("focus-record");
    m.state.now = 0.0;
    m.state.wall_ms = 1_794_096_000_000;
    send(&mut m, &mut w, msg(FocusMsg::Start));
    m.state.now = 1500.0;
    m.state.wall_ms += 1_500_000;
    send(&mut m, &mut w, Msg::Tick);
    let text = m.state.recall.focus.recorded.clone().unwrap();
    assert!(text.starts_with("Focus block of 25 min recorded"), "{text}");
}

#[test]
fn the_menu_pauses_resumes_resets_and_picks_a_preset() {
    let mut m = running();
    m.state.recall.focus.banked = 30.0;
    let mut h = harness(m);
    h.run_steps(2);
    h.get_by_label_contains("Focus 2").click();
    h.run_steps(2);
    h.get_by_label("Pause the timer").click();
    h.run_steps(2);
    assert!(h.state().0.state.recall.focus.since.is_none(), "paused");
    h.get_by_label("Resume the timer").click();
    h.run_steps(2);
    assert!(h.state().0.state.recall.focus.since.is_some(), "resumed");
    h.get_by_label("50 / 10").click();
    h.run_steps(2);
    assert_eq!(h.state().0.state.recall.focus.preset.work, 50);
    h.get_by_label("Reset the timer").click();
    h.run_steps(2);
    assert!(studyfrog_ui::focus_timer::is_idle(
        &h.state().0.state.recall.focus
    ));
}
