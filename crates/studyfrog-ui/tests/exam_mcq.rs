//! Unit and integration tests for multiple-choice distractor exam view and automated grading.

use eframe::egui;
use studyfrog_rehearsal::distractor::MultipleChoiceQuestion;
use studyfrog_ui::view::exam_mcq::{
    McqMsg, McqState, mcq_load_question, mcq_select_choice, mcq_submit_or_next, new_mcq_state,
    render_mcq_view, update_mcq,
};

fn sample_question(prompt: &str, correct: &str, distractors: &[&str]) -> MultipleChoiceQuestion {
    let mut all_choices = vec![correct.to_string()];
    for d in distractors {
        all_choices.push(d.to_string());
    }
    // canonical index 0 is correct in our mock pool
    MultipleChoiceQuestion {
        prompt: prompt.to_string(),
        correct_answer: correct.to_string(),
        distractors: distractors.iter().map(|s| s.to_string()).collect(),
        all_choices,
        correct_index: 0,
    }
}

#[test]
fn test_new_mcq_state_defaults() {
    let state: McqState = new_mcq_state();
    assert!(state.current_question.is_none());
    assert!(state.selected_choice.is_none());
    assert!(!state.is_revealed);
    assert_eq!(state.score, 0);
    assert_eq!(state.total_answered, 0);
}

#[test]
fn test_mcq_load_and_select_choice() {
    let mut state = new_mcq_state();
    let q = sample_question("What is 2+2?", "4", &["3", "5", "6"]);
    mcq_load_question(&mut state, q);

    assert!(state.current_question.is_some());
    assert_eq!(state.selected_choice, None);
    assert!(!state.is_revealed);

    mcq_select_choice(&mut state, 2);
    assert_eq!(state.selected_choice, Some(2));

    // Out of bounds selection is ignored
    mcq_select_choice(&mut state, 99);
    assert_eq!(state.selected_choice, Some(2));
}

#[test]
fn test_mcq_submit_correct_and_advance() {
    let mut state = new_mcq_state();
    let q = sample_question(
        "What is the capital of France?",
        "Paris",
        &["Rome", "Berlin", "Madrid"],
    );
    mcq_load_question(&mut state, q);

    // Can't submit without selecting
    assert_eq!(mcq_submit_or_next(&mut state), None);
    assert!(!state.is_revealed);

    // Select correct choice (index 0)
    mcq_select_choice(&mut state, 0);
    let result = mcq_submit_or_next(&mut state);
    assert_eq!(result, Some(true));
    assert!(state.is_revealed);
    assert_eq!(state.score, 1);
    assert_eq!(state.total_answered, 1);

    // Choice selection locked while revealed
    mcq_select_choice(&mut state, 1);
    assert_eq!(state.selected_choice, Some(0));

    // Next toggles reveal and clears question
    let next_result = mcq_submit_or_next(&mut state);
    assert_eq!(next_result, None);
    assert!(!state.is_revealed);
    assert_eq!(state.selected_choice, None);
    assert!(state.current_question.is_none());
    assert_eq!(state.score, 1);
    assert_eq!(state.total_answered, 1);
}

#[test]
fn test_mcq_submit_incorrect() {
    let mut state = new_mcq_state();
    let q = sample_question(
        "What is the speed of light?",
        "299,792 km/s",
        &["150,000 km/s", "3,000 km/s"],
    );
    mcq_load_question(&mut state, q);

    // Select wrong choice (index 1)
    mcq_select_choice(&mut state, 1);
    let result = mcq_submit_or_next(&mut state);
    assert_eq!(result, Some(false));
    assert!(state.is_revealed);
    assert_eq!(state.score, 0);
    assert_eq!(state.total_answered, 1);
}

#[test]
fn test_update_mcq_flow() {
    let mut state = new_mcq_state();
    let q = sample_question(
        "Rust is...",
        "A systems language",
        &["A reptile", "A browser"],
    );
    update_mcq(&mut state, McqMsg::LoadQuestion(q));
    assert!(state.current_question.is_some());

    update_mcq(&mut state, McqMsg::SelectChoice(0));
    assert_eq!(state.selected_choice, Some(0));

    let res = update_mcq(&mut state, McqMsg::SubmitOrNext);
    assert_eq!(res, Some(true));
    assert_eq!(state.score, 1);

    update_mcq(&mut state, McqMsg::Reset);
    assert_eq!(state, new_mcq_state());
}

#[test]
fn test_render_mcq_view_active_question() {
    let mut state = new_mcq_state();
    let q = sample_question(
        "Which planet is closest to the Sun?",
        "Mercury",
        &["Venus", "Earth", "Mars"],
    );
    mcq_load_question(&mut state, q);
    mcq_select_choice(&mut state, 0);

    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        render_mcq_view(ui, &mut state);
    });
    output.textures_delta.clear();
}

#[test]
fn test_render_mcq_view_revealed_and_report() {
    let mut state = new_mcq_state();
    let q = sample_question("What is 5*5?", "25", &["20", "30", "15"]);
    mcq_load_question(&mut state, q);
    mcq_select_choice(&mut state, 1); // incorrect choice
    mcq_submit_or_next(&mut state);
    assert!(state.is_revealed);

    // Render revealed feedback state
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        render_mcq_view(ui, &mut state);
    });
    output.textures_delta.clear();

    // Advance to report state
    mcq_submit_or_next(&mut state);
    assert!(state.current_question.is_none());
    assert_eq!(state.total_answered, 1);

    // Render report state
    let mut output2 = ctx.run_ui(Default::default(), |ui| {
        render_mcq_view(ui, &mut state);
    });
    output2.textures_delta.clear();
}
