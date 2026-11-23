#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Headless egui_kittest suite for mock exam graded review across objective and open formats (STORY-0207).

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_rehearsal::curriculum_build::{AnswerKey, GeneratedQuestion, assemble};
use studyfrog_rehearsal::exam_answer::{ExamAnswer, MULTI_SELECT_RULE};
use studyfrog_rehearsal::exam_format::{ExamItemFormat, default_format_mix};
use studyfrog_service::exam_gen::Generated;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::setup::mock_exam::grading::{DEFAULT_PASS_MARK, breakdown, grade_with};
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
        GeneratedQuestion {
            gap_hints: vec![None],
            gap_answers: vec!["consistency".to_owned()],
            ..q_base(
                ExamItemFormat::FillInGaps,
                "Reliability is [1].",
                "consistency",
            )
        },
        GeneratedQuestion {
            options: vec!["Bias".to_owned(), "Consistency".to_owned()],
            correct_indices: vec![1],
            ..q_base(
                ExamItemFormat::SingleSelect,
                "Which is reliability?",
                "Consistency",
            )
        },
        GeneratedQuestion {
            options: vec![
                "Test-retest".to_owned(),
                "Guessing".to_owned(),
                "Parallel forms".to_owned(),
            ],
            correct_indices: vec![0, 2],
            ..q_base(
                ExamItemFormat::MultipleSelect,
                "Select reliability methods.",
                "Test-retest; Parallel forms",
            )
        },
    ];
    let (exam, keys): (_, Vec<AnswerKey>) = assemble("Review Exam", &qs);
    Generated {
        exam,
        keys,
        from_model: 0,
        from_templates: 4,
        rejected: Vec::new(),
        short: Vec::new(),
    }
}

#[test]
fn graded_review_renders_format_breakdown_engines_and_multi_select_rule() {
    let g = mixed_generated();
    let answers = vec![
        ExamAnswer::Text("Consistency".to_owned()),
        ExamAnswer::Gaps(vec!["  Consistency. ".to_owned()]),
        ExamAnswer::Single(Some(1)),
        ExamAnswer::Multiple(vec![true, true, false]), // 1 hit - 1 wrong = 0 points
    ];
    let mut m = open_model();
    m.state.screen = Screen::Rehearse;
    m.state.rehearse.mock = Some(MockExamUi {
        title: "Review Exam".to_owned(),
        format_mix: default_format_mix(),
        result: Some(Box::new(g)),
        started: true,
        answers,
        ..MockExamUi::default()
    });
    let mut h = harness(m);
    h.run_steps(2);
    h.get_by_label("Finish and grade").click();
    h.run_steps(2);
    assert_all_named(&h, "mock exam graded review");

    assert!(h.query_by_label("By format").is_some());
    assert!(h.query_by_label("By topic").is_some());
    assert!(h.query_by_label(MULTI_SELECT_RULE).is_some());
    assert_eq!(h.query_all_by_label("Objective").count(), 3);
    assert!(h.query_by_label("Word overlap").is_some());
    assert!(h.query_by_label("Passed").is_some());

    let ui = h.state().0.state.rehearse.mock.as_ref().unwrap();
    let graded = ui.graded.as_ref().unwrap();
    assert_eq!(graded[0].earned, 2);
    assert_eq!(graded[1].earned, 2);
    assert_eq!(graded[2].earned, 2);
    assert_eq!(graded[3].earned, 0);

    apply(
        &mut h.state_mut().0,
        Msg::MockExam(MockExamMsg::SetPassMark(90.0)),
    );
    h.run_steps(2);
    assert!(h.query_by_label("Not passed").is_some());
}

#[test]
fn model_grading_and_topic_breakdown_work_for_open_items() {
    let g = mixed_generated();
    let answers = vec![
        ExamAnswer::Text("x".to_owned()),
        ExamAnswer::Gaps(vec!["consistency".to_owned()]),
        ExamAnswer::Single(Some(1)),
        ExamAnswer::Multiple(vec![true, false, true]),
    ];
    let mut ask = |_: &str, _: &str| {
        Ok(r#"{"criteria_results": [{"criterion_text": "Consistency.", "met": true, "feedback": "ok"}]}"#.to_owned())
    };
    let graded = grade_with(&g.exam, &g.keys, &answers, Some(&mut ask));
    assert!(graded[0].from_model);
    assert_eq!(graded[0].earned, 2);
    assert!(!graded[1].from_model);
    let b = breakdown(&graded);
    assert_eq!(b, vec![("Psychometrics".to_owned(), 8, 8)]);
    let _ = DEFAULT_PASS_MARK;
}
