#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Headless egui_kittest suite for answering all 6 mock exam item formats (STORY-0207).

mod common;

use common::views::{assert_all_named, harness, open_model};
use eframe::egui::Key;
use eframe::egui::accesskit::Role;
use egui_kittest::kittest::{NodeT, Queryable};
use studyfrog_rehearsal::curriculum_build::{AnswerKey, GeneratedQuestion, assemble};
use studyfrog_rehearsal::exam_answer::ExamAnswer;
use studyfrog_rehearsal::exam_format::{ExamItemFormat, default_format_mix};
use studyfrog_service::exam_gen::Generated;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::setup::mock_exam::answer::init_answers;
use studyfrog_ui::setup::mock_exam::{MockExamMsg, MockExamUi};
use studyfrog_ui::state::Screen;

fn q_base(format: ExamItemFormat, prompt: &str, sample: &str) -> GeneratedQuestion {
    GeneratedQuestion {
        format,
        prompt: prompt.to_owned(),
        sample_answer: sample.to_owned(),
        rubric: vec![sample.to_owned()],
        points: 2,
        topic: "Psychometrics".to_owned(),
        options: Vec::new(),
        correct_indices: Vec::new(),
        gap_hints: Vec::new(),
        gap_answers: Vec::new(),
    }
}

fn mixed_generated() -> Generated {
    let qs = vec![
        q_base(
            ExamItemFormat::OpenQuestion,
            "Define reliability.",
            "Consistency.",
        ),
        q_base(
            ExamItemFormat::CaseScenario,
            "Case 1: evaluate sample.",
            "Representative sample.",
        ),
        q_base(
            ExamItemFormat::Calculation,
            "Give formula for SE.",
            "SE = s / sqrt(n)",
        ),
        GeneratedQuestion {
            gap_hints: vec![Some("noun".to_owned()), None],
            gap_answers: vec!["consistency".to_owned(), "repeated".to_owned()],
            ..q_base(
                ExamItemFormat::FillInGaps,
                "Reliability is [1] across [2] tests.",
                "consistency; repeated",
            )
        },
        GeneratedQuestion {
            options: vec![
                "Bias".to_owned(),
                "Consistency".to_owned(),
                "Noise".to_owned(),
            ],
            correct_indices: vec![1],
            ..q_base(
                ExamItemFormat::SingleSelect,
                "Which best defines reliability?",
                "Consistency",
            )
        },
        GeneratedQuestion {
            options: vec![
                "Test-retest".to_owned(),
                "Random guessing".to_owned(),
                "Parallel forms".to_owned(),
            ],
            correct_indices: vec![0, 2],
            ..q_base(
                ExamItemFormat::MultipleSelect,
                "Select valid reliability methods.",
                "Test-retest; Parallel forms",
            )
        },
    ];
    let (exam, keys): (_, Vec<AnswerKey>) = assemble("Mixed Klausur", &qs);
    Generated {
        exam,
        keys,
        from_model: 0,
        from_templates: 6,
        rejected: Vec::new(),
        short: Vec::new(),
    }
}

fn answering_model(locked: bool) -> studyfrog_ui::model::Model {
    let g = mixed_generated();
    let answers = init_answers(&g.exam);
    let mut m = open_model();
    m.state.screen = Screen::Rehearse;
    m.state.rehearse.mock = Some(MockExamUi {
        title: "Mixed Klausur".to_owned(),
        format_mix: default_format_mix(),
        result: Some(Box::new(g)),
        started: true,
        locked,
        answers,
        ..MockExamUi::default()
    });
    m
}

#[test]
fn preview_shows_format_pills_and_start_switches_to_six_format_inputs() {
    let g = mixed_generated();
    let answers = init_answers(&g.exam);
    let mut m = open_model();
    m.state.screen = Screen::Rehearse;
    m.state.rehearse.mock = Some(MockExamUi {
        title: "Mixed Klausur".to_owned(),
        format_mix: default_format_mix(),
        result: Some(Box::new(g)),
        answers,
        ..MockExamUi::default()
    });
    let mut h = harness(m);
    h.run_steps(2);
    assert!(h.query_by_label("Fill in the gaps").is_some());
    assert!(h.query_by_label("Single select").is_some());
    assert!(h.query_by_label("Multiple select").is_some());
    h.get_by_label("Start the exam").click();
    h.run_steps(2);
    assert!(h.state().0.state.rehearse.mock.as_ref().unwrap().started);
}

#[test]
fn answering_widgets_accept_text_gaps_radio_and_checkbox_input() {
    let mut h = egui_kittest::Harness::builder()
        .with_size(eframe::egui::vec2(1000.0, 1200.0))
        .build_ui_state(
            |ui, (model, effects): &mut common::views::App| {
                effects.extend(studyfrog_ui::model::frame(model, ui));
            },
            (answering_model(false), Vec::new()),
        );
    h.run_steps(2);
    assert_all_named(&h, "mock exam answering");
    for role in [Role::CheckBox, Role::RadioButton] {
        for node in h.query_all_by_role(role) {
            let lbl = node.accesskit_node().label().unwrap_or_default();
            assert!(!lbl.trim().is_empty(), "{role:?} missing label");
        }
    }

    h.get_by_label("Answer 1: Open question").focus();
    h.get_by_label("Answer 1: Open question")
        .type_text("Consistency");
    h.run_steps(2);

    h.get_by_label("Question 4 gap 1").focus();
    h.get_by_label("Question 4 gap 1").type_text("consistency");
    h.run_steps(1);
    h.key_press(Key::Tab);
    h.run_steps(1);
    h.get_by_label("Question 4 gap 2").focus();
    h.get_by_label("Question 4 gap 2").type_text("repeated");
    h.run_steps(2);

    h.get_by_label("Question 5: B. Consistency").click();
    h.run_steps(2);
    h.get_by_label("Question 6: A. Test-retest").click();
    h.run_steps(2);
    h.get_by_label("Question 6: C. Parallel forms").click();
    h.run_steps(2);

    let answers = &h.state().0.state.rehearse.mock.as_ref().unwrap().answers;
    assert_eq!(answers[0], ExamAnswer::Text("Consistency".to_owned()));
    assert_eq!(
        answers[3],
        ExamAnswer::Gaps(vec!["consistency".to_owned(), "repeated".to_owned()])
    );
    assert_eq!(answers[4], ExamAnswer::Single(Some(1)));
    assert_eq!(answers[5], ExamAnswer::Multiple(vec![true, false, true]));
}

#[test]
fn locked_exam_disables_edits_and_hides_finish_button() {
    let mut m = answering_model(true);
    apply(&mut m, Msg::MockExam(MockExamMsg::AnswerSingle(4, 0)));
    let mut h = harness(m);
    h.run_steps(2);
    assert!(h.query_by_label("Finish and grade").is_none());
    let answers = &h.state().0.state.rehearse.mock.as_ref().unwrap().answers;
    assert_eq!(answers[4], ExamAnswer::Single(None));
}
