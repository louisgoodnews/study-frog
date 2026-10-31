#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The PDF index: listing, storing, forgetting and searching.

mod common;

use common::vault::write;
use studyfrog_store::db::open_memory;
use studyfrog_store::migrate::migrate;
use studyfrog_store::pdf_index::{PagePlain, PdfRow, pdf_forget, pdf_rows, pdf_store, pdf_touch};
use studyfrog_store::pdf_search::{search_pdfs, snippet};
use studyfrog_store::schema::MIGRATIONS;
use studyfrog_store::vault::list_pdfs;

fn row(path: &str, title: &str) -> PdfRow {
    PdfRow {
        path: path.into(),
        size: 10,
        mtime_ns: 5,
        hash: u64::MAX,
        pages: 2,
        title: title.into(),
        error: None,
    }
}

fn page(text: &str) -> PagePlain {
    PagePlain {
        text: text.into(),
        image_only: false,
    }
}

fn db() -> rusqlite::Connection {
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    conn
}

#[test]
fn pdfs_are_listed_anywhere_but_not_in_hidden_folders() {
    let dir = common::temp_dir("pdf-list");
    let r = &dir.path;
    for f in [
        "a.pdf",
        "deck/b.PDF",
        "deck/deep/c.pdf",
        ".studyfrog/x.pdf",
        "note.md",
        "d.pdf.txt",
    ] {
        write(r, f, "%PDF-1.4");
    }
    let rels: Vec<String> = list_pdfs(r).unwrap().into_iter().map(|f| f.rel).collect();
    assert_eq!(rels, ["a.pdf", "deck/b.PDF", "deck/deep/c.pdf"]);
}

#[test]
fn a_stored_file_comes_back_with_its_hash_and_is_replaced_not_duplicated() {
    let mut conn = db();
    pdf_store(
        &mut conn,
        &row("a.pdf", "Alpha"),
        &[page("one"), page("two")],
    )
    .unwrap();
    pdf_store(&mut conn, &row("a.pdf", "Alpha 2"), &[page("only")]).unwrap();
    let rows = pdf_rows(&conn).unwrap();
    assert_eq!(
        (rows.len(), rows["a.pdf"].title.as_str(), rows["a.pdf"].hash),
        (1, "Alpha 2", u64::MAX)
    );
    pdf_touch(&conn, "a.pdf", 99, 7).unwrap();
    assert_eq!(pdf_rows(&conn).unwrap()["a.pdf"].size, 99);
    pdf_forget(&mut conn, &["a.pdf".into()]).unwrap();
    let pages: i64 = conn
        .query_row("SELECT COUNT(*) FROM pdf_page", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        (pdf_rows(&conn).unwrap().len(), pages),
        (0, 0),
        "pages go with the file"
    );
}

#[test]
fn search_finds_titles_then_page_text_with_one_based_pages_and_literal_wildcards() {
    let mut conn = db();
    pdf_store(
        &mut conn,
        &row("m.pdf", "Mitosis notes"),
        &[
            page("cells divide"),
            page("The spindle forms in 100% of cells"),
        ],
    )
    .unwrap();
    pdf_store(&mut conn, &row("o.pdf", "Other"), &[page("nothing here")]).unwrap();
    let hits = search_pdfs(&conn, "mitosis", 10).unwrap();
    assert_eq!((hits.len(), hits[0].page), (1, None));
    let hits = search_pdfs(&conn, "SPINDLE", 10).unwrap();
    assert_eq!((hits[0].path.as_str(), hits[0].page), ("m.pdf", Some(2)));
    assert!(hits[0].snippet.contains("spindle"));
    assert_eq!(search_pdfs(&conn, "100%", 10).unwrap().len(), 1);
    assert!(
        search_pdfs(&conn, "9%", 10).unwrap().is_empty(),
        "a percent sign is not a wildcard"
    );
    assert!(search_pdfs(&conn, "  ", 10).unwrap().is_empty());
    assert_eq!(
        search_pdfs(&conn, "e", 2).unwrap().len(),
        2,
        "limit applies"
    );
}

#[test]
fn a_snippet_is_short_and_follows_the_match_even_with_unicode() {
    let text = format!(
        "{} Übung findet statt {}",
        "wort ".repeat(40),
        "ende ".repeat(40)
    );
    let s = snippet(&text, "übung");
    assert!(s.contains("Übung") && s.chars().count() <= 90, "{s}");
    assert_eq!(snippet("", "x"), "");
}
