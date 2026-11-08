use proptest::prelude::*;

use crate::exam::{
    ExamGrade, GRADE_BANDS, exam_grade, exam_percent, exam_plan, grade_badge, grade_exam,
    grade_from_letter, grade_german, grade_letter, grade_passes,
};
use crate::plan::{ItemKind, RunPlan, ScheduleEffect, plan_spaced};
use crate::uid::Uid;

#[test]
fn every_band_boundary_gets_the_grade_of_the_spec() {
    let table = [
        (100.0, "A+", "1+"),
        (95.0, "A+", "1+"),
        (94.9, "A", "1.0"),
        (90.0, "A", "1.0"),
        (89.9, "A-", "1.3"),
        (85.0, "A-", "1.3"),
        (84.9, "B+", "1.7"),
        (80.0, "B+", "1.7"),
        (79.9, "B", "2.0"),
        (75.0, "B", "2.0"),
        (74.9, "B-", "2.3"),
        (70.0, "B-", "2.3"),
        (69.9, "C", "3.0"),
        (65.0, "C", "3.0"),
        (64.9, "D", "4.0"),
        (60.0, "D", "4.0"),
        (59.9, "F", "5.0"),
        (0.0, "F", "5.0"),
    ];
    for (percent, letter, german) in table {
        let g = exam_grade(percent);
        assert_eq!(
            (grade_letter(g), grade_german(g)),
            (letter, german),
            "{percent}"
        );
    }
    assert!(grade_passes(exam_grade(60.0)));
    assert!(!grade_passes(exam_grade(59.9)));
    assert_eq!(grade_badge(ExamGrade::AMinus), "A- / 1.3");
}

#[test]
fn letters_round_trip_and_bands_are_five_percent_apart() {
    for (g, _) in GRADE_BANDS {
        assert_eq!(grade_from_letter(grade_letter(g)), Some(g));
    }
    assert_eq!(grade_from_letter("E"), None);
    let lows: Vec<u32> = GRADE_BANDS[..8].iter().map(|(_, l)| *l).collect();
    assert!(lows.windows(2).all(|w| w[0] - w[1] == 50));
}

#[test]
fn the_percent_is_the_mean_over_all_items_with_one_decimal() {
    assert_eq!(exam_percent(&[1.0, 0.5], 2), 75.0);
    assert_eq!(exam_percent(&[1.0, 0.5], 4), 37.5, "missing items count 0");
    assert_eq!(exam_percent(&[0.6], 1), 60.0);
    assert_eq!(exam_percent(&[0.123_45], 1), 12.3);
    assert_eq!(exam_percent(&[], 0), 0.0);
    let r = grade_exam(Uid(1), &[Some(0.9), None, Some(0.9)], 2, 5);
    assert_eq!((r.items, r.answered, r.percent), (3, 2, 60.0));
    assert_eq!((r.grade, r.passed), (ExamGrade::D, true));
}

#[test]
fn an_exam_plan_keeps_cards_and_questions_and_never_schedules() {
    let plan = exam_plan(plan_spaced()).unwrap();
    assert_eq!(plan.filter.kinds, [ItemKind::Card, ItemKind::Question]);
    assert_eq!(plan.effect, ScheduleEffect::PracticeOnly);
    assert!(!plan.requeue_again);
    let mut notes_and_cards = plan_spaced();
    notes_and_cards.filter.kinds = vec![ItemKind::Note, ItemKind::Card];
    assert_eq!(
        exam_plan(notes_and_cards).unwrap().filter.kinds,
        [ItemKind::Card]
    );
    let mut notes = RunPlan { ..plan_spaced() };
    notes.filter.kinds = vec![ItemKind::Note];
    assert!(exam_plan(notes).is_err());
}

proptest! {
    #[test]
    fn grades_never_get_better_as_the_percent_falls(a in 0.0_f32..=100.0, b in 0.0_f32..=100.0) {
        let rank = |g| GRADE_BANDS.iter().position(|(x, _)| *x == g).unwrap();
        let (hi, lo) = if a >= b { (a, b) } else { (b, a) };
        prop_assert!(rank(exam_grade(hi)) <= rank(exam_grade(lo)));
        prop_assert_eq!(grade_passes(exam_grade(a)), (a * 10.0).round() >= 600.0);
    }

    #[test]
    fn the_percent_stays_within_0_and_100(scores in prop::collection::vec(-1.0_f32..2.0, 0..20), extra in 0_usize..5) {
        let p = exam_percent(&scores, scores.len() + extra);
        prop_assert!((0.0..=100.0).contains(&p));
    }
}
