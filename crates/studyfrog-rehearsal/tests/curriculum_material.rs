//! The 40 / 30 / 30 plan against the material the notes hold (BUG-0055): full, partial and empty notes.
#![allow(clippy::unwrap_used)]

use studyfrog_rehearsal::curriculum::chunk_markdown;
use studyfrog_rehearsal::curriculum_build::{
    GeneratedQuestion, Kind, Plan, Short, material, plan, short_note, shortfalls, templates,
};

/// BUG-0055: full notes (four definitions, three formulas, five paragraphs) give the plan's counts exactly.
#[test]
fn full_material_gives_the_plan_counts_and_no_shortfall() {
    let chunks = chunk_markdown(FULL);
    let want = plan(10);
    assert_eq!(
        want,
        Plan {
            recall: 4,
            case: 3,
            calc: 3
        }
    );
    assert_eq!(
        material(&chunks),
        Plan {
            recall: 4,
            case: 5,
            calc: 3
        }
    );
    let qs = templates(&chunks, want);
    assert_eq!(count_kind(&qs, Kind::Recall), 4);
    assert_eq!(count_kind(&qs, Kind::Case), 3);
    assert_eq!(count_kind(&qs, Kind::Calculation), 3);
    assert!(shortfalls(want, made_of(&qs)).is_empty());
}

/// BUG-0055: partial notes (two definitions, one paragraph, one formula) give what they hold, and each short part
/// is named with its counts.
#[test]
fn partial_material_gives_what_it_holds_and_names_each_short_part() {
    let chunks = chunk_markdown(PARTIAL);
    assert_eq!(
        material(&chunks),
        Plan {
            recall: 2,
            case: 1,
            calc: 1
        }
    );
    let want = plan(10);
    let qs = templates(&chunks, want);
    let made = made_of(&qs);
    assert_eq!(
        made,
        Plan {
            recall: 2,
            case: 1,
            calc: 1
        }
    );
    let short = shortfalls(want, made);
    assert_eq!(short.len(), 3);
    assert_eq!(
        short[0],
        Short {
            kind: Kind::Recall,
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
    assert_eq!(
        material(&chunks),
        Plan {
            recall: 0,
            case: 0,
            calc: 0
        }
    );
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
        assert_eq!(p.recall + p.case + p.calc, total, "total {total}");
    }
}

const FULL: &str = "# Reliabilität\nReliabilität: consistency of measurement over repeated testing occasions.\nValidität: the degree to which a test measures what it claims to measure.\nObjektivität: independence of results from the person who scores the test.\nNormierung: placing a raw score against the norm group of the test.\n\nThe reliability coefficient compares true score variance with observed variance.\n\nValidity needs evidence from content, criteria and construct.\n\n# Formeln\ns_e = s_x * sqrt(1 - r_xx)\nKR = (k / (k - 1)) * (1 - sum / 2)\nt = 50 + 10 * z\n\nTesting conditions shape the results of every candidate.\n";

const PARTIAL: &str = "# Reliabilität\nReliabilität: consistency of measurement over repeated testing occasions.\nValidität: the degree to which a test measures what it claims to measure.\ns_e = s_x * sqrt(1 - r_xx)\n";

fn count_kind(qs: &[GeneratedQuestion], kind: Kind) -> usize {
    qs.iter().filter(|q| q.kind == kind).count()
}

fn made_of(qs: &[GeneratedQuestion]) -> Plan {
    Plan {
        recall: count_kind(qs, Kind::Recall),
        case: count_kind(qs, Kind::Case),
        calc: count_kind(qs, Kind::Calculation),
    }
}
