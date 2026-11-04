use std::collections::BTreeMap;

use crate::civil::Date;
use crate::diary::{
    DiaryEntry, RatingValue, diary_title, is_standard_scale, rating_int, rating_standard_score,
};
use crate::uid::Uid;

#[test]
fn standard_scales_are_recognized() {
    assert!(is_standard_scale("performance"));
    assert!(is_standard_scale("focus"));
    assert!(is_standard_scale("sleep"));
    assert!(!is_standard_scale("caffeine"));
    assert!(!is_standard_scale("unknown"));
}

#[test]
fn diary_title_falls_back_to_civil_date() {
    let entry = DiaryEntry {
        uid: Uid(1),
        date: Date {
            year: 2026,
            month: 11,
            day: 4,
        },
        title: None,
        created: None,
        updated: None,
        study_minutes: Some(25),
        tags: vec!["math".into()],
        ratings: BTreeMap::new(),
        runs: Vec::new(),
        stacks: vec!["stack1".into()],
        items: vec![Uid(2)],
        exams: vec!["midterm".into()],
        content: "Good session.".into(),
    };
    assert_eq!(diary_title(&entry), "Diary: 2026-11-04");

    let mut with_title = entry;
    with_title.title = Some("Calculus prep".into());
    assert_eq!(diary_title(&with_title), "Calculus prep");
}

#[test]
fn rating_conversions() {
    let int_val = RatingValue::Int(4);
    assert_eq!(rating_int(&int_val), Some(4));
    assert_eq!(rating_standard_score(&int_val), Some(4));

    let out_of_range = RatingValue::Int(6);
    assert_eq!(rating_int(&out_of_range), Some(6));
    assert_eq!(rating_standard_score(&out_of_range), None);

    let text_val = RatingValue::Text("high".into());
    assert_eq!(rating_int(&text_val), None);
    assert_eq!(rating_standard_score(&text_val), None);
}
