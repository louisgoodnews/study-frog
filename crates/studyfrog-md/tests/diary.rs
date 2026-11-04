//! Diary Markdown tests: parsing, rendering, validation, and round-tripping.

use studyfrog_core::civil::Date;
use studyfrog_core::diary::RatingValue;
use studyfrog_md::doc::{Doc, Kind, doc_kind, doc_meta};
use studyfrog_md::parse::parse_doc;
use studyfrog_md::render::render_doc;
use studyfrog_md::ulid::parse_ulid;

const SAMPLE_DIARY: &str = r#"---
id: 01K2V9B7M1N3P5R7T9W0XY1234
kind: diary
date: 2026-11-04
title: Evening Biochemistry Rehearsal
created: 2026-11-04T19:45:00.000Z
updated: 2026-11-04T19:52:30.000Z
study_minutes: 35
tags:
  - biochem
  - enzymes
  - quiet-evening
performance: 4
success: 4
confidence: 3
effort: 4
focus: 4
energy: 3
difficulty: 4
satisfaction: 4
understanding: 4
mood: 4
stress: 2
motivation: 4
sleep: 3
runs:
  - 01K2V8X3Y4Z5A6B7C8D9E01234
stacks:
  - biochem/metabolism
items:
  - 01K2P4Q5R6S7T8A9V0W1X2Y3Z4
exams:
  - midterm-2026
---
Felt in the zone tonight after dinner. The Michaelis-Menten kinetics cards from [[Enzyme Kinetics]] were tough at first, but deriving the Lineweaver-Burk equation finally clicked.

![Lineweaver-Burk plot](assets/enzymes-plot-01k2v9.png)

Need to review Km vs Vmax inhibitors tomorrow before the mock exam.
"#;

#[test]
fn parse_and_verify_diary_metadata() {
    let doc = parse_doc(SAMPLE_DIARY).expect("sample diary should parse");
    assert_eq!(doc_kind(&doc), Kind::Diary);

    let Doc::Diary(diary) = doc else {
        panic!("expected Diary variant");
    };

    assert_eq!(
        diary.meta.uid,
        parse_ulid("01K2V9B7M1N3P5R7T9W0XY1234").unwrap()
    );
    assert_eq!(
        diary.date,
        Date {
            year: 2026,
            month: 11,
            day: 4
        }
    );
    assert_eq!(
        diary.custom_title.as_deref(),
        Some("Evening Biochemistry Rehearsal")
    );
    assert_eq!(diary.meta.title, "Evening Biochemistry Rehearsal");
    assert_eq!(diary.study_minutes, Some(35));
    assert_eq!(diary.meta.tags, vec!["biochem", "enzymes", "quiet-evening"]);
    assert!(diary.body.contains("[[Enzyme Kinetics]]"));
}

#[test]
fn parse_and_verify_diary_ratings_and_links() {
    let doc = parse_doc(SAMPLE_DIARY).expect("sample diary should parse");
    let Doc::Diary(diary) = doc else {
        panic!("expected Diary variant");
    };

    assert_eq!(diary.ratings.get("performance"), Some(&RatingValue::Int(4)));
    assert_eq!(diary.ratings.get("confidence"), Some(&RatingValue::Int(3)));
    assert_eq!(diary.ratings.get("stress"), Some(&RatingValue::Int(2)));
    assert_eq!(diary.ratings.get("sleep"), Some(&RatingValue::Int(3)));

    assert_eq!(
        diary.runs,
        vec![parse_ulid("01K2V8X3Y4Z5A6B7C8D9E01234").unwrap()]
    );
    assert_eq!(diary.stacks, vec!["biochem/metabolism"]);
    assert_eq!(
        diary.items,
        vec![parse_ulid("01K2P4Q5R6S7T8A9V0W1X2Y3Z4").unwrap()]
    );
    assert_eq!(diary.exams, vec!["midterm-2026"]);
    assert!(
        diary
            .body
            .contains("![Lineweaver-Burk plot](assets/enzymes-plot-01k2v9.png)")
    );
}

#[test]
fn diary_round_trip_identical() {
    let doc = parse_doc(SAMPLE_DIARY).expect("initial parse");
    let rendered = render_doc(&doc).expect("render doc");
    let reparsed = parse_doc(&rendered).expect("reparse doc");
    assert_eq!(doc, reparsed);
}

#[test]
fn diary_preserves_comments_and_custom_ratings() {
    let text = r#"---
id: 01K2V9B7M1N3P5R7T9W0XY1234
kind: diary
date: 2026-11-04
performance: 5
caffeine: 2
# An unmanaged comment
custom_note: high focus
# Trailing note for custom field
---
Worked in the quiet library room.
"#;
    let doc = parse_doc(text).expect("parse with custom fields");
    let Doc::Diary(ref diary) = doc else {
        panic!("expected Diary variant");
    };

    assert_eq!(diary.ratings.get("performance"), Some(&RatingValue::Int(5)));
    assert_eq!(diary.ratings.get("caffeine"), Some(&RatingValue::Int(2)));
    assert_eq!(
        diary.ratings.get("custom_note"),
        Some(&RatingValue::Text("high focus".into()))
    );

    let rendered = render_doc(&doc).expect("render with comments");
    assert!(rendered.contains("caffeine: 2"));
    assert!(rendered.contains("# An unmanaged comment"));
    assert!(rendered.contains("# Trailing note for custom field"));

    let reparsed = parse_doc(&rendered).expect("reparse with custom fields");
    assert_eq!(doc, reparsed);
}

#[test]
fn diary_optional_title_defaults_and_round_trips_without_title_key() {
    let text = r#"---
id: 01K2V9B7M1N3P5R7T9W0XY1234
kind: diary
date: 2026-11-04
focus: 4
---
No title provided in front-matter.
"#;
    let doc = parse_doc(text).expect("parse untitled diary");
    assert_eq!(doc_meta(&doc).title, "Diary: 2026-11-04");

    let Doc::Diary(ref diary) = doc else {
        panic!("expected diary");
    };
    assert_eq!(diary.custom_title, None);

    let rendered = render_doc(&doc).expect("render untitled diary");
    assert!(!rendered.contains("title:"));

    let reparsed = parse_doc(&rendered).expect("reparse untitled diary");
    assert_eq!(doc, reparsed);
}

#[test]
fn diary_validation_errors() {
    // Missing date
    let no_date = r#"---
id: 01K2V9B7M1N3P5R7T9W0XY1234
kind: diary
---
Body
"#;
    assert!(parse_doc(no_date).is_err());

    // Invalid rating (score 6 > 5)
    let bad_rating = r#"---
id: 01K2V9B7M1N3P5R7T9W0XY1234
kind: diary
date: 2026-11-04
performance: 6
---
Body
"#;
    assert!(parse_doc(bad_rating).is_err());

    // Invalid date (Feb 30)
    let bad_date = r#"---
id: 01K2V9B7M1N3P5R7T9W0XY1234
kind: diary
date: 2026-02-30
---
Body
"#;
    assert!(parse_doc(bad_date).is_err());
}
