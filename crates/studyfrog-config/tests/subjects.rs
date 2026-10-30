#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! `subjects.toml`: teachers and subjects.

use studyfrog_config::subjects::{Subject, SubjectBook, parse_subjects, render_subjects};

fn book() -> SubjectBook {
    SubjectBook {
        teachers: vec!["Ms \"Ada\" Rivera".into(), "Mr Chen".into()],
        subjects: vec![
            Subject {
                name: "Mathematics".into(),
                teacher: Some("Ms \"Ada\" Rivera".into()),
            },
            Subject {
                name: "Self study".into(),
                teacher: None,
            },
        ],
    }
}

#[test]
fn a_book_survives_render_and_parse_including_quotes() {
    let text = render_subjects(&book());
    assert_eq!(parse_subjects(&text).unwrap(), book());
    assert_eq!(parse_subjects("").unwrap(), SubjectBook::default());
}

#[test]
fn mistakes_are_named() {
    let dup = "[[subject]]\nname = \"A\"\n[[subject]]\nname = \"a\"\n";
    assert!(parse_subjects(dup).unwrap_err().message.contains("twice"));
    let unknown = "[[subject]]\nname = \"A\"\nteacher = \"Nobody\"\n";
    assert!(
        parse_subjects(unknown)
            .unwrap_err()
            .message
            .contains("unknown teacher")
    );
    assert!(
        parse_subjects("[[subject]]\nname = \"A, B\"\n")
            .unwrap_err()
            .message
            .contains("commas")
    );
    assert!(
        parse_subjects("colour = 1\n").is_err(),
        "unknown keys are refused"
    );
}
