#![allow(clippy::unwrap_used)]
//! Tests for format-aware exam generation prompts and JSON parsing (STORY-0207).

use studyfrog_agent::exam_gen::{
    FormatCounts, chunk_system, chunk_system_for_formats, parse_questions,
};

#[test]
fn chunk_system_for_formats_mentions_only_requested_formats() {
    let s = chunk_system_for_formats(FormatCounts {
        fill_in_gaps: 2,
        single_select: 1,
        ..FormatCounts::default()
    });
    assert!(s.contains("2 fill_in_gaps items"), "{s}");
    assert!(s.contains("1 single_select items"), "{s}");
    assert!(!s.contains("multiple_select"), "{s}");
    assert!(!s.contains("calculation"), "{s}");
    assert!(!s.contains("case_scenario"), "{s}");
    assert!(!s.contains("open_question"), "{s}");

    let legacy = chunk_system(2, 1, 1);
    assert!(legacy.contains("2 open_question") && legacy.contains("1 case_scenario"));
}

#[test]
fn parse_questions_reads_all_six_formats_and_optional_aliases() {
    let reply = r#"Here you go: {"questions": [
        {"kind": "open_question", "prompt": "What is reliability?", "sample_answer": "Consistency.", "rubric": ["consistency"], "points": 2},
        {"format": "fill_in_gaps", "prompt": "Reliability is [1] of measurement.", "sample_answer": "consistency", "gaps": ["consistency"], "points": 2},
        {"kind": "single_select", "prompt": "Which defines reliability?", "options": ["Consistency", "Bias"], "correct_index": 0, "sample_answer": "Consistency", "points": 2},
        {"kind": "multiple_select", "prompt": "Select valid properties.", "options": ["Consistency", "Repeatability", "Noise"], "correct_indices": [0, 1], "sample_answer": "Consistency; Repeatability", "points": 3},
        {"kind": "calculation", "prompt": "Compute standard error.", "sample_answer": "s_e = 1", "points": 4},
        {"kind": "case_scenario", "prompt": "Evaluate this study design.", "sample_answer": "Check reliability.", "points": 4}
    ]} done"#;
    let qs = parse_questions(reply).unwrap();
    assert_eq!(qs.len(), 6);
    assert_eq!(qs[1].kind, "fill_in_gaps");
    assert_eq!(qs[1].gaps, vec!["consistency"]);
    assert_eq!(qs[2].correct_index, Some(0));
    assert_eq!(qs[3].correct_indices, vec![0, 1]);
}

#[test]
fn parse_questions_rejects_prose_without_json() {
    assert!(parse_questions("no questions today").is_err());
}
