#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Inline cards and questions in a note body.

use studyfrog_md::inline_extract::{InlineKind, extract_inline};

#[test]
fn terms_and_question_pairs_are_found_under_their_heading_outside_code() {
    let body = "Intro :: not under a heading\n# Cells\n- Mitosis :: cell division\n```\nskip :: me\n```\nQ: What splits?\nsome text\nA: The cell\n## Why\n??: Why divide?\nA: To grow.\nA: stray answer\nempty ::\n";
    let found = extract_inline(body);
    let short: Vec<(InlineKind, &str, &str, Option<&str>)> = found
        .iter()
        .map(|b| {
            (
                b.kind,
                b.front.as_str(),
                b.back.as_str(),
                b.heading.as_deref(),
            )
        })
        .collect();
    assert_eq!(
        short,
        [
            (InlineKind::Card, "Intro", "not under a heading", None),
            (InlineKind::Card, "Mitosis", "cell division", Some("Cells")),
            (InlineKind::Card, "What splits?", "The cell", Some("Cells")),
            (InlineKind::Question, "Why divide?", "To grow.", Some("Why")),
        ]
    );
}

use proptest::prelude::*;
use studyfrog_md::inline_extract::{InlineBlock, extract_key, source_target};

fn word() -> impl Strategy<Value = String> {
    "[A-Za-z][A-Za-z0-9 ]{0,12}[A-Za-z0-9]"
}

proptest! {
    #[test]
    fn any_text_extracts_without_panicking(body in "(\\PC|\n){0,200}") {
        let _ = extract_inline(&body);
    }

    #[test]
    fn term_lines_are_found_and_fenced_ones_never(
        terms in prop::collection::vec((word(), word()), 1..6)
    ) {
        let lines: Vec<String> = terms.iter().map(|(t, d)| format!("- {t} :: {d}")).collect();
        let body = format!("# H\n{}\n```\n{}\n```\n", lines.join("\n"), lines.join("\n"));
        let found = extract_inline(&body);
        prop_assert_eq!(found.len(), terms.len());
        for (b, (t, d)) in found.iter().zip(&terms) {
            prop_assert_eq!(&b.front, t.trim());
            prop_assert_eq!(&b.back, d.trim());
            prop_assert_eq!(b.heading.as_deref(), Some("H"));
        }
    }

    #[test]
    fn the_key_ignores_case_spacing_and_the_answer(front in word(), a in word(), b in word()) {
        let block = |front: String, back: String| InlineBlock {
            kind: InlineKind::Card, front, back, heading: None,
        };
        let one = extract_key(&block(front.clone(), a));
        let two = extract_key(&block(format!("  {}  ", front.to_uppercase()), b));
        prop_assert_eq!(&one, &two);
        prop_assert_eq!(one.len(), 16);
    }
}

#[test]
fn the_source_target_names_the_heading_when_there_is_one() {
    assert_eq!(source_target("Lecture", Some("Cells")), "Lecture#Cells");
    assert_eq!(source_target("Lecture", None), "Lecture");
}
