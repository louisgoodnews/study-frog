#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The case study inside a run (STORY-0199): the suggested rating follows the check, and a wrong choice
//! never suggests more than Hard.

mod common;

use studyfrog_agent::rubric_eval::fallback_evaluate;
use studyfrog_core::review::Rating;
use studyfrog_ui::model::apply;
use studyfrog_ui::setup::case::run::suggested_rating;

fn criteria() -> Vec<String> {
    vec![
        "addresses scoring speed".to_owned(),
        "cites criterion validity".to_owned(),
    ]
}

#[test]
fn a_full_answer_with_the_right_choice_suggests_good_or_better() {
    let r = fallback_evaluate(
        "The short screen addresses scoring speed for busy staff and cites criterion validity for this group.",
        true,
        &criteria(),
    );
    assert!(suggested_rating(&r) >= Rating::Good, "{r:?}");
}

#[test]
fn a_wrong_choice_suggests_at_most_hard_however_well_it_is_argued() {
    let r = fallback_evaluate(
        "The short screen addresses scoring speed and cites criterion validity.",
        false,
        &criteria(),
    );
    assert!(suggested_rating(&r) <= Rating::Hard);
}

#[test]
fn an_empty_answer_suggests_again() {
    let r = fallback_evaluate("", true, &criteria());
    assert_eq!(suggested_rating(&r), Rating::Again);
}

#[test]
fn checking_a_case_in_a_run_asks_for_its_score_to_be_recorded() {
    use studyfrog_core::run::RunMode;
    use studyfrog_core::uid::Uid;
    use studyfrog_rehearsal::case_study::CaseStudyItem;
    use studyfrog_rehearsal::input::Action;
    use studyfrog_service::content::ItemContent;
    use studyfrog_ui::effect::Effect;
    use studyfrog_ui::msg::Msg;
    use studyfrog_ui::run_state::AnswerEffect;
    use studyfrog_ui::setup::case::CaseMsg;

    let mut m = common::views::running_model(RunMode::Flip);
    let case = CaseStudyItem {
        vignette: "A clinic wants a short screen for anxiety. Staff are rushed and the test must be quick to score.".to_owned(),
        decision_prompt: "Which instrument?".to_owned(),
        options: vec!["Short screen".to_owned(), "Full interview".to_owned()],
        expected_choices: vec!["Short screen".to_owned()],
        rubric_criteria: vec!["addresses scoring speed".to_owned()],
    };
    let content = ItemContent {
        case: Some(Box::new(case)),
        title: "Screen".into(),
        prompt: "Vignette".into(),
        answer: "Expected: Short screen".into(),
        ..ItemContent::default()
    };
    apply(&mut m, Msg::ItemLoaded(Uid(1), Ok(content)));
    apply(&mut m, Msg::Case(CaseMsg::RunPick(0, true)));
    apply(
        &mut m,
        Msg::AnswerEdited("The short screen addresses scoring speed for busy staff.".to_owned()),
    );
    let effects = apply(&mut m, Msg::Case(CaseMsg::RunCheck));
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::Answer(AnswerEffect::Record { item: Uid(1), .. }))),
        "{effects:?}"
    );
    let _ = Action::Flip;
}
