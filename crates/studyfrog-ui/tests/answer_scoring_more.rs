#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Typed answers are compared with the reference and the result guides, never forces, the rating.
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
use studyfrog_ui::effect::EffectVault;
use studyfrog_ui::exec::{execute, worker_new};
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::msg::MsgRehearsal;

mod common;

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
fn the_comparison_is_stored_with_its_run_and_word_matching_can_be_forced() {
    use studyfrog_store::answer_scores::load_answer_scores;
    let vault = sample_vault("score-store");
    let mut worker = worker_new(None);
    execute(
        &mut worker,
        Effect::Vault(EffectVault::OpenVault(vault.path.display().to_string())),
        NOW_NS,
    );
    let ask = |words_only| {
        Effect::Answer(studyfrog_ui::run_state::AnswerEffect::Score {
            run: Uid(500),
            item: Uid(10),
            answer: ("a".into(), None),
            words_only,
        })
    };
    execute(&mut worker, ask(false), NOW_NS);
    let msgs = execute(&mut worker, ask(true), NOW_NS);
    let Msg::Rehearsal(MsgRehearsal::AnswerScored(_, Ok(s))) = &msgs[0] else {
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
    apply(
        &mut m,
        Msg::Rehearsal(MsgRehearsal::RunAction(Action::SubmitAnswer(
            "Paris".into(),
        ))),
    );
    apply(
        &mut m,
        Msg::Rehearsal(MsgRehearsal::AnswerScored(
            Uid(1),
            Ok(scored(0.9, Verdict::Correct, Rating::Good)),
        )),
    );
    m.state.now = 4.0;
    apply(
        &mut m,
        Msg::Rehearsal(MsgRehearsal::RunAction(Action::Rate(Rating::Good))),
    );
    apply(
        &mut m,
        Msg::Rehearsal(MsgRehearsal::RunAction(Action::Abort)),
    );
    let rows = studyfrog_ui::review::review_rows(m.state.rehearse.run.as_ref().unwrap());
    assert_eq!(rows[0].answer_match, Some((90, Verdict::Correct)));
    let mut h = harness(m);
    h.run();
    assert!(h.query_by_label("90 % (Correct)").is_some());
}
