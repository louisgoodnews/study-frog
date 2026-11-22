#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! A Markdown file is read as a card deck (one card per `## ` heading) or as one note.

use studyfrog_core::plan::ItemKind;
use studyfrog_service::xfer_adapters::{adapter_for, describe_input};
use studyfrog_service::xfer_markdown::{from_markdown, sections};

#[test]
fn each_level_two_heading_starts_a_section() {
    let (found, preface) =
        sections("intro\n## Cat\nle chat\n### Sub\nstill cat\n## Dog\nle chien\n");
    assert_eq!(preface, "intro\n");
    assert_eq!(found.len(), 2);
    assert_eq!(found[0].0, "Cat");
    assert_eq!(found[0].1, "le chat\n### Sub\nstill cat\n");
    assert_eq!(found[1], ("Dog".to_owned(), "le chien\n".to_owned()));
}

#[test]
fn a_heading_inside_a_code_fence_is_text() {
    let (found, _) = sections("## Code\n```\n## not a card\n```\nafter\n");
    assert_eq!(found.len(), 1);
    assert!(found[0].1.contains("## not a card"), "{:?}", found[0].1);
}

#[test]
fn a_deck_becomes_one_card_per_heading() {
    let bundle = from_markdown(b"# French\n\n## the cat\n\nle chat\n\n## the dog\n\nle chien\n")
        .expect("reads");
    assert_eq!(bundle.items.len(), 2);
    assert!(matches!(bundle.items[0].kind, ItemKind::Card));
    assert_eq!(bundle.items[0].text, "the cat");
    assert_eq!(bundle.items[0].answer, "le chat");
    assert_eq!(bundle.items[1].answer, "le chien");
    assert!(bundle.warnings.is_empty());
}

#[test]
fn text_before_the_first_heading_and_empty_sections_are_warned_about() {
    let bundle = from_markdown(b"Preface here\n## Empty\n\n## Full\nback\n").expect("reads");
    assert_eq!(bundle.items.len(), 1);
    assert_eq!(bundle.items[0].text, "Full");
    assert!(
        bundle
            .warnings
            .iter()
            .any(|w| w.contains("before the first ## heading"))
    );
    assert!(bundle.warnings.iter().any(|w| w.contains("\"Empty\"")));
}

#[test]
fn a_file_without_sections_is_one_note_titled_by_its_first_heading() {
    let bundle =
        from_markdown(b"---\ntags: [x]\n---\n# Lecture 3\n\nSome notes.\n").expect("reads");
    assert_eq!(bundle.items.len(), 1);
    assert!(matches!(bundle.items[0].kind, ItemKind::Note));
    assert_eq!(bundle.items[0].title, "Lecture 3");
    assert!(bundle.items[0].text.contains("Some notes."));
    assert!(
        !bundle.items[0].text.contains("tags:"),
        "front matter is not text"
    );
}

#[test]
fn an_empty_or_non_utf8_file_is_an_error() {
    assert!(from_markdown(b"  \n").is_err());
    assert!(from_markdown(&[0xff, 0xfe, 0x00]).is_err());
}

#[test]
fn a_md_file_is_detected_as_markdown_cards() {
    assert_eq!(
        adapter_for("deck.MD").map(|a| a.name),
        adapter_for("x.md").map(|a| a.name)
    );
    assert!(describe_input("/notes/deck.md", false).starts_with("Looks like Markdown cards"));
}
