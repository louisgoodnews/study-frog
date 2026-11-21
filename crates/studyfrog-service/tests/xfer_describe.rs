#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The import dialog says what a chosen path looks like before anything is read.

use studyfrog_service::xfer_adapters::describe_input;

#[test]
fn a_known_file_says_which_importer_reads_it() {
    assert_eq!(describe_input("", false), "");
    assert!(describe_input("/home/me/deck.csv", false).starts_with("Looks like CSV"));
    assert!(describe_input("C:\\decks\\cards.tsv", false).starts_with("Looks like CSV"));
    assert!(
        describe_input("/x/export.json", false).starts_with("Looks like StudyFrog export (JSON)")
    );
    assert!(describe_input("/x/Biology.apkg", false).starts_with("Looks like Anki"));
}

#[test]
fn a_folder_is_read_as_an_obsidian_vault() {
    assert_eq!(
        describe_input("/home/me/notes", true),
        "Looks like an Obsidian vault folder."
    );
}

#[test]
fn an_unsupported_file_says_so_and_lists_what_is_supported() {
    let text = describe_input("/docs/lecture.pdf", false);
    assert!(
        text.starts_with("lecture.pdf is not a file an importer reads yet"),
        "{text}"
    );
    assert!(text.contains("csv") && text.contains("apkg"), "{text}");
}
