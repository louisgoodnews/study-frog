#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Service tests for Match mode vault pair extraction, Markdown stripping, and round sampling (TASK-0300).

use std::fs;
use std::path::Path;

use studyfrog_core::time::UnixMs;
use studyfrog_service::match_pairs::{
    MatchFilter, MatchPairRequest, load_match_pairs, sample_match_round, strip_markdown,
};
use studyfrog_service::session::open_vault;

fn write_file(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}

fn card(id: &str, title: &str, front: &str, back: &str) -> String {
    format!("---\nid: {id}\nkind: card\ntitle: {title}\n---\n{front}\n???\n{back}\n")
}

fn question(id: &str, title: &str, prompt: &str, reference: &str) -> String {
    format!("---\nid: {id}\nkind: question\ntitle: {title}\n---\n{prompt}\n???\n{reference}\n")
}

fn stack_doc(id: &str, title: &str) -> String {
    format!("---\nid: {id}\nkind: stack\ntitle: {title}\n---\nStack description.\n")
}

#[test]
fn strip_markdown_cleans_formatting_links_and_wikilinks() {
    let raw = "## **Photosynthesis**\n- Converts `CO2` via [[Chloroplast|chloroplasts]] and [light](https://example.com)";
    let cleaned = strip_markdown(raw);
    assert_eq!(
        cleaned,
        "Photosynthesis Converts CO2 via chloroplasts and light"
    );
}

#[test]
fn load_match_pairs_extracts_cards_and_questions_and_filters_by_stack() {
    let dir = std::env::temp_dir().join(format!(
        "sf_match_pairs_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();

    write_file(
        &dir,
        "psych/_stack.md",
        &stack_doc("00000000000000000000000001", "Psychologie"),
    );
    write_file(
        &dir,
        "psych/c1.md",
        &card(
            "00000000000000000000000002",
            "Nomothetisch",
            "**Nomothetischer Ansatz**",
            "Suche nach allgemeingueltigen Gesetzen",
        ),
    );
    write_file(
        &dir,
        "psych/q1.md",
        &question(
            "00000000000000000000000003",
            "Idiographisch",
            "Was ist der *idiographische* Ansatz?",
            "Fokus auf den Einzelfall",
        ),
    );
    write_file(
        &dir,
        "psych/note.md",
        "---\nid: 00000000000000000000000004\nkind: note\ntitle: Ignore Me\n---\nJust a note.\n",
    );
    write_file(
        &dir,
        "psych/empty.md",
        &card("00000000000000000000000005", "Empty", "No back", ""),
    );
    write_file(
        &dir,
        "math/_stack.md",
        &stack_doc("00000000000000000000000006", "Mathe"),
    );
    write_file(
        &dir,
        "math/c2.md",
        &card(
            "00000000000000000000000007",
            "Pythagoras",
            "a^2 + b^2",
            "c^2",
        ),
    );

    let (session, _) = open_vault(dir.to_str().unwrap()).unwrap();
    let now = UnixMs(1_700_000_000_000);

    let all = load_match_pairs(
        &session,
        &MatchPairRequest {
            stack_folder: None,
            filter: MatchFilter::All,
            now,
        },
    )
    .unwrap();
    assert_eq!(
        all.pairs.len(),
        3,
        "expected 2 psych + 1 math valid pairs: {:?}",
        all.pairs
    );
    assert!(all.pairs.contains(&(
        "Nomothetischer Ansatz".into(),
        "Suche nach allgemeingueltigen Gesetzen".into()
    )));
    assert!(all.pairs.contains(&(
        "Was ist der idiographische Ansatz?".into(),
        "Fokus auf den Einzelfall".into()
    )));

    let psych_only = load_match_pairs(
        &session,
        &MatchPairRequest {
            stack_folder: Some("psych".into()),
            filter: MatchFilter::All,
            now,
        },
    )
    .unwrap();
    assert_eq!(psych_only.pairs.len(), 2);
    assert!(!psych_only.pairs.iter().any(|(p, _)| p == "a^2 + b^2"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn sample_match_round_varies_with_seed_and_respects_count() {
    let pool: Vec<(String, String)> = (0..12)
        .map(|i| (format!("Prompt {i}"), format!("Answer {i}")))
        .collect();
    let r1 = sample_match_round(&pool, 6, 11, MatchFilter::All);
    let r2 = sample_match_round(&pool, 6, 99, MatchFilter::All);
    assert_eq!(r1.len(), 6);
    assert_eq!(r2.len(), 6);
    assert_ne!(r1, r2, "different seeds should deal different rounds");
}
