#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Typed answers are compared with the reference and the result guides, never forces, the rating.

mod common;

use common::views::{harness, running_model};
use common::{NOW_NS, sample_vault};
use egui_kittest::kittest::Queryable;
use studyfrog_core::review::Rating;
use studyfrog_core::run::RunMode;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::input::Action;
use studyfrog_service::compare::Scored;
use studyfrog_slm::score::{Score, Verdict};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{execute, worker_new};
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;

fn scored(value: f32, verdict: Verdict, rating: Rating) -> Scored {
    Scored {
        score: Score {
            value,
            verdict,
            rubric_hits: vec![true, false],
        },
        rating,
        engine: "test".into(),
    }
}

#[test]
fn submitting_an_answer_asks_for_a_comparison_and_the_result_is_kept_until_rating() {
    let mut m = running_model(RunMode::Answer);
    let effects = apply(&mut m, Msg::RunAction(Action::SubmitAnswer("Paris".into())));
    assert!(
        effects.contains(&Effect::ScoreAnswer {
            run: Uid(100),
            item: Uid(1),
            answer: "Paris".into(),
            words_only: false,
        }),
        "{effects:?}"
    );
    apply(
        &mut m,
        Msg::AnswerScored(Uid(2), Ok(scored(0.1, Verdict::Incorrect, Rating::Again))),
    );
    assert!(
        m.state.rehearse.run.as_ref().unwrap().scored.is_none(),
        "a score for another item is dropped"
    );

    apply(
        &mut m,
        Msg::AnswerScored(Uid(1), Ok(scored(0.9, Verdict::Correct, Rating::Good))),
    );
    assert_eq!(
        m.state
            .rehearse
            .run
            .as_ref()
            .unwrap()
            .scored
            .as_ref()
            .unwrap()
            .rating,
        Rating::Good
    );
    apply(&mut m, Msg::RunAction(Action::Rate(Rating::Hard)));
    assert!(
        m.state.rehearse.run.as_ref().unwrap().scored.is_none(),
        "rating clears it"
    );
}

#[test]
fn a_failed_comparison_is_reported_and_rating_still_works() {
    let mut m = running_model(RunMode::Answer);
    apply(&mut m, Msg::RunAction(Action::SubmitAnswer("x".into())));
    apply(&mut m, Msg::AnswerScored(Uid(1), Err("no model".into())));
    assert!(m.state.notice.as_ref().unwrap().text.contains("no model"));
    let effects = apply(&mut m, Msg::RunAction(Action::Rate(Rating::Good)));
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::RecordStep { .. }))
    );
}

#[test]
fn the_revealed_answer_screen_shows_the_match_and_the_suggested_rating() {
    let mut m = running_model(RunMode::Answer);
    apply(&mut m, Msg::RunAction(Action::SubmitAnswer("Paris".into())));
    apply(
        &mut m,
        Msg::AnswerScored(Uid(1), Ok(scored(0.9, Verdict::Correct, Rating::Good))),
    );
    let mut h = harness(m);
    h.run();
    assert!(
        h.query_by_label("Your answer matched 90 %: Correct. Suggested rating: Good.")
            .is_some()
    );
    assert!(
        h.query_by_label("1 key point(s) were not covered.")
            .is_some()
    );
    assert!(
        h.query_by_label("Good (3)").is_some(),
        "the user still chooses"
    );
}

#[test]
fn the_executor_scores_against_a_real_card_offline() {
    let vault = sample_vault("score");
    let mut worker = worker_new(None);
    let open = execute(
        &mut worker,
        Effect::OpenVault(vault.path.display().to_string()),
        NOW_NS,
    );
    assert!(matches!(open[0], Msg::VaultOpened(Ok(_))));
    let msgs = execute(
        &mut worker,
        Effect::ScoreAnswer {
            run: Uid(500),
            item: Uid(10),
            answer: "a".into(),
            words_only: false,
        },
        NOW_NS,
    );
    let Msg::AnswerScored(uid, Ok(s)) = &msgs[0] else {
        panic!("{:?}", msgs[0])
    };
    assert_eq!(
        (*uid, s.score.verdict, s.rating),
        (Uid(10), Verdict::Correct, Rating::Good)
    );
    let none = execute(
        &mut worker,
        Effect::ScoreAnswer {
            run: Uid(500),
            item: Uid(12),
            answer: "a".into(),
            words_only: false,
        },
        NOW_NS,
    );
    assert!(matches!(&none[0], Msg::AnswerScored(_, Err(e)) if e.contains("cards and questions")));
}

#[test]
fn the_comparison_is_stored_with_its_run_and_word_matching_can_be_forced() {
    use studyfrog_store::answer_scores::load_answer_scores;
    let vault = sample_vault("score-store");
    let mut worker = worker_new(None);
    execute(
        &mut worker,
        Effect::OpenVault(vault.path.display().to_string()),
        NOW_NS,
    );
    let ask = |words_only| Effect::ScoreAnswer {
        run: Uid(500),
        item: Uid(10),
        answer: "a".into(),
        words_only,
    };
    execute(&mut worker, ask(false), NOW_NS);
    let msgs = execute(&mut worker, ask(true), NOW_NS);
    let Msg::AnswerScored(_, Ok(s)) = &msgs[0] else {
        panic!("{:?}", msgs[0])
    };
    assert!(s.engine.contains("chosen in the settings"), "{}", s.engine);
    let stored = load_answer_scores(&worker.session.as_ref().unwrap().conn, Uid(500)).unwrap();
    assert_eq!(stored.len(), 2);
    assert_eq!(
        (stored[0].item, stored[0].verdict.as_str()),
        (Uid(10), "correct")
    );
    assert!(stored[0].value > 0.99 && stored[0].engine.contains("word matching"));
}

#[test]
fn the_review_shows_how_each_answer_matched() {
    let mut m = running_model(RunMode::Answer);
    apply(&mut m, Msg::RunAction(Action::SubmitAnswer("Paris".into())));
    apply(
        &mut m,
        Msg::AnswerScored(Uid(1), Ok(scored(0.9, Verdict::Correct, Rating::Good))),
    );
    m.state.now = 4.0;
    apply(&mut m, Msg::RunAction(Action::Rate(Rating::Good)));
    apply(&mut m, Msg::RunAction(Action::Abort));
    let rows = studyfrog_ui::review::review_rows(m.state.rehearse.run.as_ref().unwrap());
    assert_eq!(rows[0].answer_match, Some((90, Verdict::Correct)));
    let mut h = harness(m);
    h.run();
    assert!(h.query_by_label("90 % (Correct)").is_some());
}
