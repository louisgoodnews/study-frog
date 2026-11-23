//! The 40 / 30 / 30 plan against the material the notes hold (BUG-0055): full, partial and empty notes.
#![allow(clippy::unwrap_used)]

use studyfrog_rehearsal::curriculum::chunk_markdown;
use studyfrog_rehearsal::curriculum_build::{
    ExamItemFormat, FormatPlan, FormatShort, GeneratedQuestion, material, plan, short_note,
    shortfalls, templates,
};
use studyfrog_rehearsal::exam_format::{plan_count, plan_total, set_plan_count};

fn legacy_made(recall: usize, case: usize, calc: usize) -> FormatPlan {
    let mut p = FormatPlan::default();
    set_plan_count(&mut p, ExamItemFormat::OpenQuestion, recall);
    set_plan_count(&mut p, ExamItemFormat::CaseScenario, case);
    set_plan_count(&mut p, ExamItemFormat::Calculation, calc);
    p
}

/// BUG-0055: full notes give the plan's counts exactly.
#[test]
fn full_material_gives_the_plan_counts_and_no_shortfall() {
    let chunks = chunk_markdown(FULL);
    let want = plan(10);
    assert_eq!(want, legacy_made(4, 3, 3));
    let mat = material(&chunks);
    assert_eq!(plan_count(&mat, ExamItemFormat::OpenQuestion), 4);
    assert_eq!(plan_count(&mat, ExamItemFormat::CaseScenario), 5);
    assert_eq!(plan_count(&mat, ExamItemFormat::Calculation), 3);
    let qs = templates(&chunks, want);
    assert_eq!(count_fmt(&qs, ExamItemFormat::OpenQuestion), 4);
    assert_eq!(count_fmt(&qs, ExamItemFormat::CaseScenario), 3);
    assert_eq!(count_fmt(&qs, ExamItemFormat::Calculation), 3);
    assert!(shortfalls(want, made_of(&qs)).is_empty());
}

/// BUG-0055: partial notes give what they hold, and each short part is named with its counts.
#[test]
fn partial_material_gives_what_it_holds_and_names_each_short_part() {
    let chunks = chunk_markdown(PARTIAL);
    let mat = material(&chunks);
    assert_eq!(plan_count(&mat, ExamItemFormat::OpenQuestion), 2);
    assert_eq!(plan_count(&mat, ExamItemFormat::CaseScenario), 1);
    assert_eq!(plan_count(&mat, ExamItemFormat::Calculation), 1);
    let want = plan(10);
    let qs = templates(&chunks, want);
    let made = made_of(&qs);
    assert_eq!(made, legacy_made(2, 1, 1));
    let short = shortfalls(want, made);
    assert_eq!(short.len(), 3);
    assert_eq!(
        short[0],
        FormatShort {
            format: ExamItemFormat::OpenQuestion,
            wanted: 4,
            made: 2
        }
    );
    let note = short_note(&short[2]);
    assert!(
        note.contains("Part C") && note.contains("1 of the 3"),
        "{note}"
    );
}

/// BUG-0055: with no material at all, every part is short and none is silently padded.
#[test]
fn empty_material_makes_no_questions_and_every_part_is_short() {
    let chunks = chunk_markdown("");
    assert_eq!(material(&chunks), FormatPlan::default());
    let want = plan(10);
    let qs = templates(&chunks, want);
    assert!(qs.is_empty());
    assert_eq!(shortfalls(want, made_of(&qs)).len(), 3);
}

/// The plan's shares hold for any total: the parts add up to the total, and a part is never negative.
#[test]
fn the_plan_adds_up_to_the_total() {
    for total in 3..=20 {
        let p = plan(total);
        assert_eq!(plan_total(&p), total, "total {total}");
    }
}

const FULL: &str = "# Reliabilität\nReliabilität: consistency of measurement over repeated testing occasions.\nValidität: the degree to which a test measures what it claims to measure.\nObjektivität: independence of results from the person who scores the test.\nNormierung: placing a raw score against the norm group of the test.\n\nThe reliability coefficient compares true score variance with observed variance.\n\nValidity needs evidence from content, criteria and construct.\n\n# Formeln\ns_e = s_x * sqrt(1 - r_xx)\nKR = (k / (k - 1)) * (1 - sum / 2)\nt = 50 + 10 * z\n\nTesting conditions shape the results of every candidate.\n";

const PARTIAL: &str = "# Reliabilität\nReliabilität: consistency of measurement over repeated testing occasions.\nValidität: the degree to which a test measures what it claims to measure.\ns_e = s_x * sqrt(1 - r_xx)\n";

fn count_fmt(qs: &[GeneratedQuestion], format: ExamItemFormat) -> usize {
    qs.iter().filter(|q| q.format == format).count()
}

fn made_of(qs: &[GeneratedQuestion]) -> FormatPlan {
    legacy_made(
        count_fmt(qs, ExamItemFormat::OpenQuestion),
        count_fmt(qs, ExamItemFormat::CaseScenario),
        count_fmt(qs, ExamItemFormat::Calculation),
    )
}
