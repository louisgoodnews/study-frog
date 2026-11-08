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
