#![allow(clippy::unwrap_used)]
//! Tests for exam item format mix validation, proportional count resolution, and wire names (STORY-0207).

use studyfrog_rehearsal::exam_format::{
    ALL_EXAM_FORMATS, ExamItemFormat, MAX_EXAM_QUESTIONS, MixMode, default_format_mix,
    format_index, format_wire_name, parse_format_wire_name, plan_count, plan_total,
    resolve_format_plan, validate_format_mix,
};

#[test]
fn default_format_mix_validates_and_resolves_to_ten() {
    let mix = default_format_mix();
    assert_eq!(validate_format_mix(&mix), Ok(()));
    let plan = resolve_format_plan(&mix).unwrap();
    assert_eq!(plan_total(&plan), 10);
    for fmt in ALL_EXAM_FORMATS {
        assert!(plan_count(&plan, fmt) >= 1, "missing {fmt:?}");
    }
}

#[test]
fn empty_selection_and_zero_totals_are_rejected() {
    let mut none_enabled = default_format_mix();
    for a in &mut none_enabled.formats {
        a.enabled = false;
    }
    assert!(validate_format_mix(&none_enabled).is_err());

    let mut zero_total = default_format_mix();
    zero_total.total_questions = 0;
    assert!(validate_format_mix(&zero_total).is_err());

    let mut zero_weight = default_format_mix();
    zero_weight.formats[0].value = 0;
    assert!(validate_format_mix(&zero_weight).is_err());

    let mut over_max = default_format_mix();
    over_max.total_questions = MAX_EXAM_QUESTIONS + 1;
    assert!(validate_format_mix(&over_max).is_err());

    let mut counts_zero = default_format_mix();
    counts_zero.mode = MixMode::Counts;
    for a in &mut counts_zero.formats {
        a.value = 0;
    }
    assert!(validate_format_mix(&counts_zero).is_err());
}

#[test]
fn weights_resolve_proportionally_and_preserve_total() {
    for total in 1..=30 {
        let mix = default_format_mix();
        let mut custom = mix;
        custom.total_questions = total;
        let plan = resolve_format_plan(&custom).unwrap();
        assert_eq!(plan_total(&plan), total, "total={total}");
    }
    let mut skewed = default_format_mix();
    skewed.total_questions = 10;
    for a in &mut skewed.formats {
        a.enabled = false;
    }
    skewed.formats[0].enabled = true;
    skewed.formats[0].value = 99;
    skewed.formats[1].enabled = true;
    skewed.formats[1].value = 1;
    let plan = resolve_format_plan(&skewed).unwrap();
    assert_eq!(plan_total(&plan), 10);
    assert_eq!(plan_count(&plan, ExamItemFormat::OpenQuestion), 9);
    assert_eq!(plan_count(&plan, ExamItemFormat::FillInGaps), 1);
}

#[test]
fn counts_mode_preserves_exact_per_format_counts() {
    let mut mix = default_format_mix();
    mix.mode = MixMode::Counts;
    for a in &mut mix.formats {
        a.enabled = false;
        a.value = 5;
    }
    mix.formats[format_index(ExamItemFormat::FillInGaps)].enabled = true;
    mix.formats[format_index(ExamItemFormat::FillInGaps)].value = 3;
    mix.formats[format_index(ExamItemFormat::SingleSelect)].enabled = true;
    mix.formats[format_index(ExamItemFormat::SingleSelect)].value = 4;

    let plan = resolve_format_plan(&mix).unwrap();
    assert_eq!(plan_total(&plan), 7);
    assert_eq!(plan_count(&plan, ExamItemFormat::OpenQuestion), 0);
    assert_eq!(plan_count(&plan, ExamItemFormat::FillInGaps), 3);
    assert_eq!(plan_count(&plan, ExamItemFormat::SingleSelect), 4);
}

#[test]
fn wire_name_round_trip_and_legacy_aliases() {
    for fmt in ALL_EXAM_FORMATS {
        assert_eq!(parse_format_wire_name(format_wire_name(fmt)), Some(fmt));
    }
    assert_eq!(
        parse_format_wire_name("recall"),
        Some(ExamItemFormat::OpenQuestion)
    );
    assert_eq!(
        parse_format_wire_name("cloze"),
        Some(ExamItemFormat::FillInGaps)
    );
    assert_eq!(
        parse_format_wire_name("single-choice"),
        Some(ExamItemFormat::SingleSelect)
    );
    assert_eq!(
        parse_format_wire_name("multi select"),
        Some(ExamItemFormat::MultipleSelect)
    );
    assert_eq!(
        parse_format_wire_name("calc"),
        Some(ExamItemFormat::Calculation)
    );
    assert_eq!(
        parse_format_wire_name("case"),
        Some(ExamItemFormat::CaseScenario)
    );
}

#[test]
fn extreme_weights_resolve_without_overflow_and_large_counts_fail_cleanly() {
    let mut mix = default_format_mix();
    mix.total_questions = MAX_EXAM_QUESTIONS;
    for a in &mut mix.formats {
        a.enabled = true;
        a.value = 1;
    }
    mix.formats[0].value = usize::MAX / 2;

    assert_eq!(validate_format_mix(&mix), Ok(()));
    let plan = resolve_format_plan(&mix).unwrap();
    assert_eq!(plan_total(&plan), MAX_EXAM_QUESTIONS);
    for fmt in ALL_EXAM_FORMATS {
        assert!(plan_count(&plan, fmt) >= 1, "missing {fmt:?}");
    }

    mix.formats[1].value = usize::MAX / 2;
    mix.formats[2].value = usize::MAX / 2;
    assert_eq!(validate_format_mix(&mix), Ok(()));
    let multi = resolve_format_plan(&mix).unwrap();
    assert_eq!(plan_total(&multi), MAX_EXAM_QUESTIONS);

    let mut counts = default_format_mix();
    counts.mode = MixMode::Counts;
    counts.formats[0].value = usize::MAX;
    counts.formats[1].value = 1;
    assert!(validate_format_mix(&counts).is_err());
}
