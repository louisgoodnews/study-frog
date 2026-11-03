#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! PDFs are named by their file name; the title inside the file is added only when it says something.

mod common;

use common::views::{harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_store::pdf_index::PdfRow;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::pdf_names::{file_name, informative_title, is_placeholder, label};
use studyfrog_ui::state::{Screen, Selection};

#[test]
fn the_file_name_is_the_last_part_of_the_path() {
    assert_eq!(file_name("bio/lectures/Vorlesung 3.pdf"), "Vorlesung 3.pdf");
    assert_eq!(file_name("loose.pdf"), "loose.pdf");
}

#[test]
fn placeholder_titles_are_recognised_and_real_ones_are_kept() {
    for title in [
        "",
        "  ",
        "PowerPoint-Präsentation",
        "PowerPoint-Präsentation 3",
        "Presentation1",
        "Untitled",
        "Unbenannt",
        "Microsoft Word - notes.docx",
        "Vorlesung 3",
        "vorlesung 3.pdf",
    ] {
        assert!(is_placeholder(title, "Vorlesung 3.pdf"), "{title:?}");
    }
    for title in ["Test theory and measurement", "Kapitel 4: Reliabilität"] {
        assert!(!is_placeholder(title, "Vorlesung 3.pdf"), "{title:?}");
    }
    assert_eq!(
        informative_title("a/x.pdf", " Real title "),
        Some("Real title")
    );
    assert_eq!(label("a/x.pdf", "PowerPoint-Präsentation"), "x.pdf");
    assert_eq!(label("a/x.pdf", "Real title"), "x.pdf · Real title");
}

fn row(path: &str, pages: usize, title: &str) -> PdfRow {
    PdfRow {
        path: path.into(),
        size: 1,
        mtime_ns: i64::try_from(pages).unwrap(),
        hash: 1,
        pages,
        title: title.into(),
        error: None,
    }
}

#[test]
fn two_files_with_the_same_placeholder_title_are_told_apart_by_name() {
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Library));
    apply(&mut m, Msg::Select(Selection::Stack("french".into())));
    m.state.app.stack_tabs = vec!["french:list".into()];
    m.state.pdf_library.rows = vec![
        row("french/Vorlesung 1.pdf", 48, "PowerPoint-Präsentation"),
        row("french/Vorlesung 2.pdf", 63, "PowerPoint-Präsentation"),
        row("french/Reader.pdf", 10, "Grammar of French"),
    ];
    let mut h = harness(m);
    h.run_steps(4);
    assert!(h.query_by_label("Vorlesung 1.pdf (48 pages)").is_some());
    assert!(h.query_by_label("Vorlesung 2.pdf (63 pages)").is_some());
    assert!(h.query_by_label("Reader.pdf (10 pages)").is_some());
    assert!(
        h.query_by_label("Grammar of French").is_some(),
        "a real title still shows"
    );
    assert!(
        h.query_by_label_contains("PowerPoint").is_none(),
        "a placeholder title is not shown"
    );
}
