#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Marking items while rehearsing and the review after the run.

mod common;

use common::views::{assert_all_named, harness, running_model};
use common::{NOW_NS, sample_vault};
use egui_kittest::kittest::Queryable;
use studyfrog_core::event::{Difficulty, EventKind};
use studyfrog_core::plan::ScheduleEffect;
use studyfrog_core::review::Rating;
use studyfrog_core::run::RunMode;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::input::Action;
use studyfrog_service::content::ItemContent;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{execute, worker_new};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::review::{current_mark, review_rows};

const TWENTY_WORDS: &str = "one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen twenty";

fn act(m: &mut Model, now: f64, action: Action) -> Vec<Effect> {
    m.state.now = now;
    apply(m, Msg::RunAction(action))
}

fn mark_of(m: &Model) -> Option<Difficulty> {
    current_mark(m.state.rehearse.run.as_ref().unwrap())
}

/// Item 1 has a twenty-word back and was reviewed before; it is read for `read_s` seconds.
fn finished_run(read_s: f64, previous: Option<Rating>, rating: Rating) -> Model {
    let mut m = running_model(RunMode::Flip);
    let content = ItemContent {
        title: "Numbers".into(),
        prompt: "Count".into(),
        answer: TWENTY_WORDS.into(),
        previous,
    };
    apply(&mut m, Msg::ItemLoaded(Uid(1), Ok(content)));
    act(&mut m, 0.5, Action::Mark(Difficulty::Easy));
    act(&mut m, 1.0, Action::Flip);
    act(&mut m, 1.0 + read_s, Action::Rate(rating));
    act(&mut m, 20.0, Action::Abort);
    m
}

#[test]
fn marking_is_recorded_replaced_by_the_latest_mark_and_forgotten_for_the_next_item() {
    let mut m = running_model(RunMode::Flip);
    assert_eq!(mark_of(&m), None);
    let effects = act(&mut m, 0.5, Action::Mark(Difficulty::Difficult));
    let saved = effects.iter().any(|e| {
        matches!(e, Effect::RecordStep { out, .. }
        if out.events.iter().any(|ev| ev.kind == EventKind::ItemMarked(Difficulty::Difficult)))
    });
    assert!(saved, "the mark is saved with the run: {effects:?}");
    assert_eq!(mark_of(&m), Some(Difficulty::Difficult));
    act(&mut m, 0.6, Action::Mark(Difficulty::Medium));
    assert_eq!(
        mark_of(&m),
        Some(Difficulty::Medium),
        "the last mark counts"
    );
    act(&mut m, 1.0, Action::Flip);
    act(&mut m, 2.0, Action::Mark(Difficulty::Easy));
    act(&mut m, 3.0, Action::Rate(Rating::Good));
    assert_eq!(mark_of(&m), None, "the next item starts unmarked");
}

#[test]
fn the_mark_buttons_and_keys_send_the_mark() {
    let mut h = harness(running_model(RunMode::Flip));
    h.run();
    h.get_by_label("Difficult (D)").click();
    h.run();
    assert_eq!(mark_of(&h.state().0), Some(Difficulty::Difficult));
    h.get_by_label("Easy (E)").click();
    h.run();
    assert_eq!(mark_of(&h.state().0), Some(Difficulty::Easy));
}

#[test]
fn the_review_shows_previous_own_and_suggested_rating() {
    let m = finished_run(2.0, Some(Rating::Hard), Rating::Good);
    let rows = review_rows(m.state.rehearse.run.as_ref().unwrap());
    let r = &rows[0];
    assert_eq!(
        (r.title.as_str(), r.previous, r.marked, r.rated),
        (
            "Numbers",
            Some(Rating::Hard),
            Some(Difficulty::Easy),
            Some(Rating::Good)
        )
    );
    assert_eq!(r.reading, Some((20, 20)), "twenty words in two seconds");
    assert_eq!(r.suggestion.unwrap().rating, Rating::Easy);

    let mut h = harness(m);
    h.run();
    assert_all_named(&h, "run review");
    for text in [
        "Previous rating",
        "Your mark",
        "Your rating",
        "Suggested rating",
        "Hard",
        "easy",
        "Good",
        "Easy",
        "This item was rather easy for you.",
        "20 words in 2.0 s",
    ] {
        assert!(h.query_by_label(text).is_some(), "missing {text}");
    }
}

#[test]
fn a_new_item_that_went_by_fast_is_suggested_hard_and_a_normal_pace_suggests_nothing() {
    let m = finished_run(2.0, None, Rating::Good);
    let r = &review_rows(m.state.rehearse.run.as_ref().unwrap())[0];
    assert_eq!(r.suggestion.unwrap().rating, Rating::Hard);
    let slow = finished_run(6.0, Some(Rating::Good), Rating::Good);
    assert_eq!(
        review_rows(slow.state.rehearse.run.as_ref().unwrap())[0].suggestion,
        None
    );
}

#[test]
fn applying_asks_to_replace_the_rating_and_the_screen_confirms_it() {
    let mut h = harness(finished_run(2.0, Some(Rating::Good), Rating::Good));
    h.run();
    h.get_by_label("Change the rating of Numbers to Easy")
        .click();
    h.run();
    let want = Effect::Rerate {
        item: Uid(1),
        expected: Rating::Good,
        to: Rating::Easy,
    };
    assert!(h.state().1.contains(&want), "{:?}", h.state().1);

    let (model, _) = h.state_mut();
    apply(model, Msg::Rerated(Uid(1), Ok(())));
    h.run();
    assert!(h.query_by_label("Changed to Easy.").is_some());
    assert!(
        h.query_by_label("Change the rating of Numbers to Easy")
            .is_none()
    );
}

#[test]
fn a_practice_run_shows_the_suggestion_but_offers_no_change() {
    let mut m = finished_run(2.0, Some(Rating::Good), Rating::Good);
    m.state
        .rehearse
        .run
        .as_mut()
        .unwrap()
        .session
        .run
        .plan
        .effect = ScheduleEffect::PracticeOnly;
    let mut h = harness(m);
    h.run();
    assert!(
        h.query_by_label("This item was rather easy for you.")
            .is_some()
    );
    assert!(
        h.query_by_label("Change the rating of Numbers to Easy")
            .is_none()
    );
}

#[test]
fn the_executor_refuses_to_change_a_review_that_does_not_exist() {
    let vault = sample_vault("rerate");
    let mut worker = worker_new(None);
    execute(
        &mut worker,
        Effect::OpenVault(vault.path.display().to_string()),
        NOW_NS,
    );
    let msgs = execute(
        &mut worker,
        Effect::Rerate {
            item: Uid(10),
            expected: Rating::Good,
            to: Rating::Easy,
        },
        NOW_NS,
    );
    assert!(
        matches!(&msgs[0], Msg::Rerated(Uid(10), Err(e)) if e.contains("changed")),
        "{:?}",
        msgs[0]
    );
}

#[test]
fn a_slow_read_is_suggested_one_step_harder_with_its_reason() {
    let m = finished_run(12.0, Some(Rating::Good), Rating::Good);
    let r = &review_rows(m.state.rehearse.run.as_ref().unwrap())[0];
    let s = r.suggestion.unwrap();
    assert_eq!(
        (s.rating, s.reason),
        (Rating::Hard, studyfrog_rehearsal::reading::Reason::Slow)
    );
    let mut h = harness(m);
    h.run();
    assert!(
        h.query_by_label("This took you a long time to read: it may be harder than you rated it.")
            .is_some()
    );
    assert!(
        h.query_by_label("Change the rating of Numbers to Hard")
            .is_some()
    );
}
