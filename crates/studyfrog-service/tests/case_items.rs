#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Case studies in a vault (STORY-0179): created as files in a stack, read back, plain questions skipped.

mod fam_support;

use fam_support::vault;
use studyfrog_rehearsal::case_study::CaseStudyItem;
use studyfrog_service::case_items::{create_case_study, load_case_studies};
use studyfrog_service::create::IdSource;
use studyfrog_service::session::open_vault;

fn case() -> CaseStudyItem {
    CaseStudyItem {
        vignette: "A clinic wants a short screen for anxiety. Staff are rushed and the test must be quick to score.".to_owned(),
        decision_prompt: "Which instrument should the clinic use?".to_owned(),
        options: vec!["Short screen".to_owned(), "Full interview".to_owned()],
        expected_choices: vec!["Short screen".to_owned()],
        rubric_criteria: vec!["addresses scoring speed".to_owned()],
    }
}

fn ids() -> IdSource {
    IdSource {
        now_ms: 1_700_000_000_000,
        random: [7; 10],
    }
}

#[test]
fn a_case_is_created_in_a_stack_and_read_back_beside_plain_questions() {
    let dir = vault("case-items", None);
    std::fs::write(
        dir.0.join("s/plain.md"),
        "---\nid: 0000000000000000000000000Q\nkind: question\ntitle: Plain\n---\nWhat is validity?\n???\nWhether it measures the construct.\n",
    )
    .unwrap();
    let (mut session, _) = open_vault(dir.0.to_str().unwrap()).unwrap();
    let (_, created) =
        create_case_study(&mut session, Some("s"), "Screen", &case(), ids()).expect("created");
    assert!(
        created.path.starts_with("s/") && created.path.ends_with(".md"),
        "{}",
        created.path
    );

    let cases = load_case_studies(&session, Some("s")).expect("loaded");
    assert_eq!(cases.len(), 1, "{cases:?}");
    assert_eq!(cases[0].1, case());
}
