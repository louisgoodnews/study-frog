#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The mock exam from notes (STORY-0186): asking for it, showing it, and saving it.

mod common;

use studyfrog_core::uid::Uid;
use studyfrog_service::exam_gen::generate;

use studyfrog_ui::effect::Effect;
use studyfrog_ui::setup::mock_exam::{
    EXAM_QUESTIONS, ExamSource, MockExamEffect, MockExamMsg, update,
};
use studyfrog_ui::state::{Selection, State};

fn notes() -> &'static str {
    "# Reliabilität\nReliabilität: consistency of measurement over repeated testing occasions.\n\nThe reliability coefficient compares true score variance with observed score variance.\n\n# Normierung\nStandard scores place a result against the norm group mean and standard deviation.\n"
}

#[test]
fn opening_a_stack_asks_for_an_exam_of_its_notes() {
    let mut state = State::default();
    state.browser.selection = Selection::Stack("bio".to_owned());
    let (state, effects) = update(state, MockExamMsg::Open);
    assert!(state.rehearse.mock.as_ref().is_some_and(|m| m.loading));
    assert!(
        matches!(
            effects.as_slice(),
            [Effect::MockExam(MockExamEffect::Generate { source: ExamSource::Notes(Some(f)), total, provider: None, .. })]
                if f == "bio" && *total == EXAM_QUESTIONS
        ),
        "{effects:?}"
    );
}

#[test]
fn opening_with_nothing_chosen_says_so() {
    let (state, effects) = update(State::default(), MockExamMsg::Open);
    assert!(effects.is_empty());
    assert!(
        state
            .rehearse
            .mock
            .as_ref()
            .and_then(|m| m.error.clone())
            .is_some()
    );
}

#[test]
fn a_made_exam_is_shown_and_its_answers_toggle() {
    let g = generate(
        &studyfrog_rehearsal::curriculum::chunk_markdown(notes()),
        4,
        "Klausur",
        None,
    );
    let state = State::default();
    let (state, _) = update(state, MockExamMsg::Open);
    let (state, _) = update(state, MockExamMsg::Generated(Ok(Box::new(g))));
    let ui = state.rehearse.mock.clone().unwrap();
    assert!(!ui.loading && ui.result.is_some());
    let (state, _) = update(state, MockExamMsg::ToggleAnswers);
    assert!(state.rehearse.mock.as_ref().unwrap().show_answers);
}

#[test]
fn saving_asks_for_the_note_and_the_outcome_is_reported() {
    let g = generate(
        &studyfrog_rehearsal::curriculum::chunk_markdown(notes()),
        4,
        "Klausur",
        None,
    );
    let mut state = State::default();
    state.browser.selection = Selection::Stack("bio".to_owned());
    let (state, _) = update(state, MockExamMsg::Open);
    let (state, _) = update(state, MockExamMsg::Generated(Ok(Box::new(g))));
    let (state, effects) = update(state, MockExamMsg::Save);
    assert!(
        matches!(
            effects.as_slice(),
            [Effect::MockExam(MockExamEffect::Save { .. })]
        ),
        "{effects:?}"
    );
    let (state, _) = update(state, MockExamMsg::Saved(Ok("bio/klausur.md".to_owned())));
    assert!(
        state
            .notice
            .as_ref()
            .is_some_and(|n| n.text.contains("bio/klausur.md"))
    );
    let (state, _) = update(state, MockExamMsg::Saved(Err("disk full".to_owned())));
    assert!(
        state
            .notice
            .as_ref()
            .is_some_and(|n| n.text.contains("disk full"))
    );
}

#[test]
fn writing_the_answers_and_finishing_grades_the_exam() {
    use studyfrog_ui::setup::mock_exam::grading::{DEFAULT_PASS_MARK, total};
    let g = generate(
        &studyfrog_rehearsal::curriculum::chunk_markdown(notes()),
        4,
        "Klausur",
        None,
    );
    let keys = g.keys.clone();
    let state = State::default();
    let (state, _) = update(state, MockExamMsg::Open);
    let (state, _) = update(state, MockExamMsg::Generated(Ok(Box::new(g))));
    let (state, _) = update(state, MockExamMsg::Start);
    assert!(state.rehearse.mock.as_ref().unwrap().started);
    let mut state = state;
    for (i, k) in keys.iter().enumerate() {
        let text = format!("{} {}", k.sample_answer, k.rubric.join(" "));
        state = update(state, MockExamMsg::Answer(i, text)).0;
    }
    let (state, _) = update(state, MockExamMsg::Finish);
    let graded = state
        .rehearse
        .mock
        .as_ref()
        .unwrap()
        .graded
        .clone()
        .expect("graded");
    assert_eq!(
        total(&graded, DEFAULT_PASS_MARK).earned,
        total(&graded, DEFAULT_PASS_MARK).possible,
        "{graded:?}"
    );
    assert!(!state.rehearse.mock.as_ref().unwrap().started);
}

#[test]
fn an_unwritten_exam_earns_nothing() {
    use studyfrog_ui::setup::mock_exam::grading::{DEFAULT_PASS_MARK, total};
    let g = generate(
        &studyfrog_rehearsal::curriculum::chunk_markdown(notes()),
        4,
        "Klausur",
        None,
    );
    let (state, _) = update(State::default(), MockExamMsg::Open);
    let (state, _) = update(state, MockExamMsg::Generated(Ok(Box::new(g))));
    let (state, _) = update(state, MockExamMsg::Finish);
    let graded = state
        .rehearse
        .mock
        .as_ref()
        .unwrap()
        .graded
        .clone()
        .unwrap();
    assert_eq!(total(&graded, DEFAULT_PASS_MARK).earned, 0);
    assert_eq!(total(&graded, DEFAULT_PASS_MARK).word, "Nicht bestanden");
}

#[test]
fn the_exam_is_made_from_the_stack_ticked_in_the_setup_and_names_it() {
    let mut state = State {
        vault: Some(common::views::vault_with_stacks(&[(1, "bio"), (2, "diag")])),
        ..State::default()
    };
    state.rehearse.form.stacks = vec![Uid(2)];
    state.browser.selection = Selection::Stack("bio".to_owned());
    let (state, effects) = update(state, MockExamMsg::Open);
    assert!(
        matches!(
            effects.as_slice(),
            [Effect::MockExam(MockExamEffect::Generate { source: ExamSource::Notes(Some(f)), title, .. })]
                if f == "diag" && title.contains("diag")
        ),
        "{effects:?}"
    );
    assert!(
        state
            .rehearse
            .mock
            .as_ref()
            .is_some_and(|m| m.title.contains("diag"))
    );
}

#[test]
fn the_pass_mark_is_a_setting_of_the_exam_that_moves_the_verdict_and_is_clamped() {
    use studyfrog_rehearsal::grade::grade_label;
    use studyfrog_ui::setup::mock_exam::MockExamUi;
    use studyfrog_ui::setup::mock_exam::grading::{
        DEFAULT_PASS_MARK, Graded, clamp_pass_mark, pass_mark_of, total,
    };
    // 6 of 10 points is 60 %: grade 3.3 at the default pass mark of 50 %, a fail at 65 %.
    let one = Graded {
        topic: "Test".to_owned(),
        prompt: "Question".to_owned(),
        answer: String::new(),
        sample_answer: String::new(),
        points_made: Vec::new(),
        possible: 10,
        earned: 6,
        counted: true,
        from_model: false,
        caveat: None,
    };
    let mut state = State::default();
    state.rehearse.mock = Some(MockExamUi {
        graded: Some(vec![one.clone()]),
        ..MockExamUi::default()
    });
    let ui = state.rehearse.mock.clone().unwrap();
    assert_eq!(pass_mark_of(&ui), DEFAULT_PASS_MARK);
    assert_eq!(
        grade_label(total(std::slice::from_ref(&one), pass_mark_of(&ui)).grade),
        "3.3"
    );

    let (state, effects) = update(state, MockExamMsg::SetPassMark(65.0));
    assert!(effects.is_empty());
    let ui = state.rehearse.mock.clone().unwrap();
    assert_eq!(pass_mark_of(&ui), 65.0);
    let graded = ui.graded.clone().unwrap();
    assert_eq!(grade_label(total(&graded, pass_mark_of(&ui)).grade), "5.0");
    assert_eq!(grade_label(total(&graded, DEFAULT_PASS_MARK).grade), "3.3");

    let (state, _) = update(state, MockExamMsg::SetPassMark(250.0));
    assert_eq!(pass_mark_of(state.rehearse.mock.as_ref().unwrap()), 100.0);
    let (state, _) = update(state, MockExamMsg::SetPassMark(0.0));
    assert_eq!(pass_mark_of(state.rehearse.mock.as_ref().unwrap()), 1.0);
    assert_eq!(clamp_pass_mark(f32::NAN), DEFAULT_PASS_MARK);
}
