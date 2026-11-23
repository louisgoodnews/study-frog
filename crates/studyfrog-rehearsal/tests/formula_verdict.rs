#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! BUG-0051: the drill praises a result only when the answer and every working row are right.

use studyfrog_rehearsal::formula_trace::{Verdict, diagnose_se, verdict};

const S: f64 = 13.3;
const R: f64 = 0.93;

#[test]
fn a_right_answer_with_a_wrong_first_row_is_not_praised() {
    // The reported case: `1 - r_xx` typed as 0.3, the answer 3.5 is within 1 % of 3.519.
    let d = diagnose_se(S, R, &[0.3, 0.7_f64.sqrt(), 3.5]);
    assert_eq!(d.deviates_at, Some(0));
    assert_eq!(
        verdict(true, d.deviates_at.is_none()),
        Verdict::AnswerRightRowWrong
    );
    assert!(
        !d.message.contains("Go back"),
        "no step precedes the first row: {}",
        d.message
    );
    assert!(!d.message.contains("All steps are correct"));
}

#[test]
fn a_right_answer_with_every_row_right_is_praised() {
    let d = diagnose_se(S, R, &[0.07, 0.07_f64.sqrt(), 3.519]);
    assert_eq!(d.deviates_at, None);
    assert_eq!(verdict(true, d.deviates_at.is_none()), Verdict::WellWorked);
}

#[test]
fn a_wrong_answer_is_outside_the_tolerance_whatever_the_rows() {
    assert_eq!(verdict(false, true), Verdict::AnswerOutside);
    assert_eq!(verdict(false, false), Verdict::AnswerOutside);
}
