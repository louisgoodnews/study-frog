//! Choice grading in rehearsal reuses the mock exam's rules (STORY-0221).

use studyfrog_rehearsal::choice_grade::{ChoiceKey, grade_choice};

fn single_key() -> ChoiceKey {
    ChoiceKey {
        options: vec![
            "Ribosome".to_owned(),
            "Mitochondrion".to_owned(),
            "Golgi apparatus".to_owned(),
        ],
        correct_indices: vec![1],
    }
}

fn multiple_key() -> ChoiceKey {
    ChoiceKey {
        options: vec!["Whale".to_owned(), "Shark".to_owned(), "Bat".to_owned()],
        correct_indices: vec![0, 2],
    }
}

#[test]
fn single_correct_choice_is_correct() {
    let grade = grade_choice(&single_key(), &[false, true, false]).unwrap();
    assert!(grade.correct);
    assert!(!grade.multiple);
    assert_eq!(grade.expected, "B. Mitochondrion");
    assert_eq!(grade.given, "B. Mitochondrion");
}

#[test]
fn single_wrong_choice_is_wrong() {
    let grade = grade_choice(&single_key(), &[true, false, false]).unwrap();
    assert!(!grade.correct);
    assert!(!grade.multiple);
    assert_eq!(grade.given, "A. Ribosome");
}

#[test]
fn single_with_nothing_chosen_is_wrong() {
    let grade = grade_choice(&single_key(), &[false, false, false]).unwrap();
    assert!(!grade.correct);
    assert_eq!(grade.given, "(none)");
}

#[test]
fn single_with_two_chosen_is_wrong_even_if_one_is_correct() {
    let grade = grade_choice(&single_key(), &[false, true, true]).unwrap();
    assert!(!grade.correct);
}

#[test]
fn multiple_all_correct_is_correct() {
    let grade = grade_choice(&multiple_key(), &[true, false, true]).unwrap();
    assert!(grade.correct);
    assert!(grade.multiple);
    assert_eq!(grade.expected, "A. Whale; C. Bat");
    assert_eq!(grade.given, "A. Whale; C. Bat");
}

#[test]
fn multiple_with_one_missing_is_wrong() {
    let grade = grade_choice(&multiple_key(), &[true, false, false]).unwrap();
    assert!(!grade.correct);
    assert!(grade.multiple);
}

#[test]
fn multiple_with_one_wrong_is_wrong() {
    let grade = grade_choice(&multiple_key(), &[true, true, true]).unwrap();
    assert!(!grade.correct);
}

#[test]
fn key_without_correct_option_has_no_grade() {
    let key = ChoiceKey {
        options: vec!["a".to_owned(), "b".to_owned()],
        correct_indices: Vec::new(),
    };
    assert!(grade_choice(&key, &[true, false]).is_none());
}
