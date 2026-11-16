//! Tests for the distractor service and question generation orchestration.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::PathBuf;

use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::distractor_service::{
    build_question_for_item, build_question_from_stack, embed_candidates, retrieve_distractors,
};
use studyfrog_service::session::open_vault;
use studyfrog_slm::lexical::Lexical;

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn distractor_vault(label: &str) -> Dir {
    let dir = std::env::temp_dir().join(format!(
        "studyfrog-distractor-{label}-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let id = |n: u128| format_ulid(Uid(n));
    let files = [
        (
            "card_france.md",
            format!(
                "---\nid: {}\nkind: card\ntitle: France Capital\n---\nCapital of France?\n???\nParis\n",
                id(100)
            ),
        ),
        (
            "card_germany.md",
            format!(
                "---\nid: {}\nkind: card\ntitle: Germany Capital\n---\nCapital of Germany?\n???\nBerlin\n",
                id(101)
            ),
        ),
        (
            "card_uk.md",
            format!(
                "---\nid: {}\nkind: card\ntitle: UK Capital\n---\nCapital of UK?\n???\nLondon\n",
                id(102)
            ),
        ),
        (
            "card_italy.md",
            format!(
                "---\nid: {}\nkind: card\ntitle: Italy Capital\n---\nCapital of Italy?\n???\nRome\n",
                id(103)
            ),
        ),
    ];
    for (name, text) in files {
        fs::write(dir.join(name), text).unwrap();
    }
    Dir(dir)
}

#[test]
fn test_embed_candidates_and_retrieve_distractors() {
    let runtime = Lexical;
    let answers = vec![
        "Berlin".to_string(),
        "London".to_string(),
        "Rome".to_string(),
    ];

    let candidates = embed_candidates(&runtime, &answers).unwrap();
    assert_eq!(candidates.len(), 3);
    assert_eq!(candidates[0].text, "Berlin");

    let distractors = retrieve_distractors(&runtime, "Paris", &answers, 2).unwrap();
    assert_eq!(distractors.len(), 2);
    assert!(!distractors.contains(&"Paris".to_string()));
}

#[test]
fn test_build_question_for_item_from_vault() {
    let dir = distractor_vault("item");
    let (session, _) = open_vault(dir.0.to_str().unwrap()).unwrap();
    let runtime = Lexical;

    let target_uid = Uid(100);
    let candidate_uids = [Uid(100), Uid(101), Uid(102), Uid(103)];

    let mcq =
        build_question_for_item(&session, &runtime, target_uid, &candidate_uids, 4, 42).unwrap();

    assert_eq!(mcq.prompt, "Capital of France?");
    assert_eq!(mcq.correct_answer, "Paris");
    assert_eq!(mcq.distractors.len(), 3);
    assert_eq!(mcq.all_choices.len(), 4);
    assert_eq!(mcq.all_choices[mcq.correct_index], "Paris");
    for d in &mcq.distractors {
        assert_ne!(d, "Paris");
    }
}

#[test]
fn test_build_question_from_stack() {
    let dir = distractor_vault("stack");
    let (session, _) = open_vault(dir.0.to_str().unwrap()).unwrap();
    let runtime = Lexical;

    let target_uid = Uid(100);
    let mcq = build_question_from_stack(&session, &runtime, target_uid, None, 4, 100).unwrap();

    assert_eq!(mcq.correct_answer, "Paris");
    assert_eq!(mcq.distractors.len(), 3);
    assert_eq!(mcq.all_choices.len(), 4);
    assert_eq!(mcq.all_choices[mcq.correct_index], "Paris");
}
