#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0205: every standard prompt is complete and its worked example is accepted by the importer.

use studyfrog_core::plan::ItemKind;
use studyfrog_service::prompts::{PROMPTS, example_of, file_name, file_text, prompt_of, prompts};
use studyfrog_service::xfer_adapters::CSV_LAYOUT;
use studyfrog_service::xfer_csv_map::{CsvOptions, read_csv};

#[test]
fn there_are_six_prompts_with_a_title_and_a_summary() {
    let all = prompts();
    let ids: Vec<_> = all.iter().map(|p| p.id).collect();
    assert_eq!(
        ids,
        [
            "flashcards",
            "questions",
            "stack",
            "notes",
            "summary",
            "critique"
        ]
    );
    for p in &all {
        assert!(!p.title.is_empty() && p.summary.ends_with('.'), "{}", p.id);
        assert!(!p.text.contains("{{"), "{} has an unfilled part", p.id);
    }
}

#[test]
fn each_prompt_hand_holds_the_ai() {
    for p in prompts() {
        let t = &p.text;
        for needed in [
            "stack,kind,title,front,back,tags",
            "| Column |",
            "double quotes",
            "UTF-8",
            "six cells",
            "PROBLEM:",
            "one code block",
            "python, r, rust, php, java, cpp",
            "toml, json, html, csv, tsv",
            "Before you answer, check each point",
        ] {
            assert!(t.contains(needed), "{} lacks {needed:?}", p.id);
        }
        assert!(example_of(PROMPTS.iter().find(|s| s.id == p.id).unwrap().text).is_some());
    }
}

#[test]
fn the_worked_example_of_every_prompt_is_read_by_the_importer_without_a_problem() {
    let kinds = |id: &str| match id {
        "flashcards" => vec![ItemKind::Card],
        "questions" => vec![ItemKind::Question],
        "stack" => vec![ItemKind::Note, ItemKind::Card, ItemKind::Question],
        _ => vec![ItemKind::Note],
    };
    for source in PROMPTS {
        let example = example_of(source.text).unwrap();
        let r = read_csv(example.as_bytes(), &CsvOptions::default()).unwrap();
        assert!(r.problems.is_empty(), "{}: {:?}", source.id, r.problems);
        assert!(
            r.sheet.header && r.sheet.names == ["stack", "kind", "title", "front", "back", "tags"]
        );
        let wanted = kinds(source.id);
        for it in &r.bundle.items {
            assert!(wanted.contains(&it.kind), "{}: {:?}", source.id, it.kind);
            assert!(!it.stack.is_empty() && !it.text.is_empty(), "{}", source.id);
        }
        for kind in &wanted {
            assert!(
                r.bundle.items.iter().any(|i| i.kind == *kind),
                "{} has no {kind:?}",
                source.id
            );
        }
    }
}

#[test]
fn the_prompts_name_the_same_columns_as_the_importer_reads() {
    assert!(CSV_LAYOUT.contains("front") && CSV_LAYOUT.contains("tags"));
    let p = prompt_of(&PROMPTS[0]);
    assert!(file_name(&p).ends_with("-flashcards.md"));
    assert!(file_text(&p).starts_with("# Create flashcards from a file\n"));
}
