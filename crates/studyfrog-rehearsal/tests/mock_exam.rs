#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for the mock exam model, navigation, review overview, and the strict timer.

use studyfrog_rehearsal::mock_exam::{
    ExamQuestion, ExamSection, MockExam, QuestionPoints, total_points, validate_mock_exam,
};
use studyfrog_rehearsal::mock_nav::{
    jump_to, new_progress, next_unanswered, review_overview, set_answered, toggle_flag,
};
use studyfrog_rehearsal::mock_timer::{TimerAlert, TimerStatus, alerts_between, timer_status};

fn q(points: u8) -> ExamQuestion {
    ExamQuestion {
        prompt: "Define reliability".to_owned(),
        topic: "Psychometrics".to_owned(),
        points: QuestionPoints(points),
    }
}

fn exam() -> MockExam {
    MockExam {
        title: "Altklausur 1".to_owned(),
        time_limit_min: 90,
        sections: vec![
            ExamSection {
                name: "Part A".to_owned(),
                questions: vec![q(2), q(2)],
            },
            ExamSection {
                name: "Part C".to_owned(),
                questions: vec![q(8)],
            },
        ],
    }
}

#[test]
fn totals_add_up_across_sections() {
    assert_eq!(total_points(&exam()), 12);
}

#[test]
fn a_valid_exam_passes_validation() {
    assert!(validate_mock_exam(&exam()).is_ok());
}

#[test]
fn invalid_exams_are_rejected() {
    let mut no_sections = exam();
    no_sections.sections.clear();
    assert!(validate_mock_exam(&no_sections).is_err());
    let mut zero_points = exam();
    zero_points.sections[0].questions[0].points = QuestionPoints(0);
    assert!(validate_mock_exam(&zero_points).is_err());
    let mut too_long = exam();
    too_long.time_limit_min = 361;
    assert!(validate_mock_exam(&too_long).is_err());
    let mut empty_section = exam();
    empty_section.sections[1].questions.clear();
    assert!(validate_mock_exam(&empty_section).is_err());
}

#[test]
fn jumping_and_flagging_are_range_checked() {
    let p = new_progress(3);
    assert_eq!(jump_to(&p, 2).unwrap().current, 2);
    assert!(jump_to(&p, 3).is_none());
    let flagged = toggle_flag(&p, 1).unwrap();
    assert!(flagged.flagged[1]);
    assert!(!toggle_flag(&flagged, 1).unwrap().flagged[1]);
    assert!(toggle_flag(&p, 9).is_none());
}

#[test]
fn next_unanswered_wraps_and_stops_when_done() {
    let mut p = new_progress(3);
    p = set_answered(&p, 1, true).unwrap();
    p = jump_to(&p, 1).unwrap();
    assert_eq!(next_unanswered(&p), Some(2));
    p = set_answered(&p, 2, true).unwrap();
    assert_eq!(next_unanswered(&p), Some(0));
    p = set_answered(&p, 0, true).unwrap();
    assert_eq!(next_unanswered(&p), None);
}

#[test]
fn review_overview_lists_unanswered_and_flagged() {
    let mut p = new_progress(4);
    p = set_answered(&p, 0, true).unwrap();
    p = toggle_flag(&p, 2).unwrap();
    let o = review_overview(&p);
    assert_eq!(o.answered, 1);
    assert_eq!(o.unanswered, vec![1, 2, 3]);
    assert_eq!(o.flagged, vec![2]);
}

#[test]
fn timer_status_warns_and_expires() {
    assert_eq!(
        timer_status(3600, 0),
        TimerStatus::Running {
            remaining_sec: 3600
        }
    );
    assert_eq!(timer_status(3600, 3000), TimerStatus::TenMinutesLeft);
    assert_eq!(timer_status(3600, 3480), TimerStatus::TwoMinutesLeft);
    assert_eq!(timer_status(3600, 3600), TimerStatus::Expired);
    assert_eq!(timer_status(3600, 9000), TimerStatus::Expired);
}

#[test]
fn each_alert_is_reported_once_even_across_a_long_gap() {
    assert_eq!(alerts_between(3600, 2000, 2999), Vec::<TimerAlert>::new());
    assert_eq!(
        alerts_between(3600, 2999, 3000),
        vec![TimerAlert::TenMinutes]
    );
    assert_eq!(
        alerts_between(3600, 2000, 3700),
        vec![
            TimerAlert::TenMinutes,
            TimerAlert::TwoMinutes,
            TimerAlert::Expired
        ]
    );
    assert_eq!(alerts_between(3600, 3000, 3100), Vec::<TimerAlert>::new());
}
