#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0205: every standard prompt is complete and its worked example is accepted by the importer.

use studyfrog_core::plan::ItemKind;
use studyfrog_service::prompts::{
    PROMPTS, example_of, file_name, file_text, prompt_of, prompt_with, prompts,
};
use studyfrog_service::prompts_meta::{
    CATEGORIES, PromptCategory, category_name, category_of, params_of,
};
use studyfrog_service::xfer_adapters::CSV_LAYOUT;
use studyfrog_service::xfer_csv_map::{CsvOptions, read_csv};

#[test]
fn there_are_ten_prompts_with_a_title_and_a_summary() {
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
            "critique",
            "explain",
            "exam",
            "socratic",
            "concept_map"
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
        "flashcards" | "exam" => vec![ItemKind::Card],
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

#[test]
fn the_csv_of_a_reply_is_found_with_or_without_chatter_and_a_problem_is_recognised() {
    use studyfrog_service::prompts::{
        ATTACH_LIMIT, clip, csv_of_reply, problem_of_reply, run_message,
    };
    let csv = "stack,kind,title,front,back,tags\na,card,,q,a,\n";
    assert_eq!(
        csv_of_reply(&format!("Here you go:\n```csv\n{csv}```\nEnjoy")).as_deref(),
        Some(csv)
    );
    assert_eq!(
        csv_of_reply(&format!("```\n{csv}```")).as_deref(),
        Some(csv)
    );
    assert_eq!(
        csv_of_reply("```python\nprint(1)\n```\n```csv\nx\n```").as_deref(),
        Some("x\n")
    );
    assert!(
        csv_of_reply("no block at all").is_none() && csv_of_reply("```csv\nunclosed").is_none()
    );
    assert_eq!(
        problem_of_reply("PROBLEM: the file is empty"),
        Some("the file is empty")
    );
    assert!(problem_of_reply("```csv\n```").is_none());
    let (kept, cut) = clip(&"é".repeat(ATTACH_LIMIT + 5));
    assert!(cut && kept.chars().count() == ATTACH_LIMIT);
    assert!(!clip("short").1);
    let p = prompt_of(&PROMPTS[0]);
    let m = run_message(&p, "notes.txt", "hello");
    assert!(m.starts_with(&p.text) && m.contains("`notes.txt`") && m.trim_end().ends_with("hello"));
}

#[test]
fn every_prompt_has_a_category_and_every_category_has_prompts() {
    for category in CATEGORIES {
        let ids: Vec<_> = PROMPTS
            .iter()
            .filter(|p| category_of(p.id) == category)
            .map(|p| p.id)
            .collect();
        assert!(!ids.is_empty(), "{}", category_name(category));
    }
    assert_eq!(category_of("exam"), PromptCategory::InteractiveTutoring);
    assert_eq!(category_of("socratic"), PromptCategory::InteractiveTutoring);
    assert_eq!(category_of("flashcards"), PromptCategory::ContentGeneration);
    assert_eq!(
        category_of("concept_map"),
        PromptCategory::AnalysisAndInsights
    );
}

#[test]
fn a_chosen_option_is_put_into_the_text_and_the_default_is_the_first() {
    let explain = PROMPTS.iter().find(|p| p.id == "explain").unwrap();
    assert!(prompt_of(explain).text.contains("at all three levels."));
    let technical = prompt_with(explain, &[("depth", 3)]).text;
    assert!(
        technical.contains("the technical level only"),
        "{technical}"
    );
    let exam = PROMPTS.iter().find(|p| p.id == "exam").unwrap();
    assert!(prompt_of(exam).text.contains("Ask 3 challenging"));
    assert!(
        prompt_with(exam, &[("count", 1)])
            .text
            .contains("Ask 5 challenging")
    );
}

#[test]
fn no_choice_leaves_a_placeholder_and_a_bad_index_falls_back_to_the_default() {
    for source in PROMPTS {
        for param in params_of(source.id) {
            for at in 0..=param.options.len() {
                let text = prompt_with(source, &[(param.key, at)]).text;
                assert!(!text.contains("{{"), "{} option {at}", source.id);
            }
        }
    }
    let explain = PROMPTS.iter().find(|p| p.id == "explain").unwrap();
    assert_eq!(
        prompt_with(explain, &[("depth", 99)]).text,
        prompt_of(explain).text
    );
}
