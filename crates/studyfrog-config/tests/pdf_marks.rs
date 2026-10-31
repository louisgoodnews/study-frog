#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The notes file next to a PDF.

use studyfrog_config::pdf_marks::{
    MarkKind, MarkRect, PdfMark, marks_path, next_mark_id, parse_marks, relocate, render_marks,
};

fn mark(id: u32, page: usize, kind: MarkKind, covered: &str, note: &str) -> PdfMark {
    PdfMark {
        id,
        page,
        kind,
        color: "green".into(),
        rects: vec![MarkRect {
            x: 72.5,
            y: 80.0,
            width: 100.25,
            height: 14.0,
        }],
        covered: covered.into(),
        note: note.into(),
    }
}

#[test]
fn marks_survive_a_round_trip_with_quotes_line_breaks_and_unicode() {
    let marks = vec![
        mark(
            2,
            4,
            MarkKind::Note,
            "",
            "Ask \"why\"\nsecond line \\ ü 日本",
        ),
        mark(1, 0, MarkKind::Highlight, "key idea", ""),
    ];
    let text = render_marks(&marks);
    let back = parse_marks(&text).unwrap();
    assert_eq!(back[0].id, 1, "written in page order");
    assert_eq!(back[1], marks[0]);
    assert!(text.contains("page = 5"), "pages are 1-based in the file");
    assert_eq!(parse_marks("").unwrap(), []);
}

#[test]
fn bad_files_are_refused_with_a_reason() {
    let ok = render_marks(&[mark(1, 0, MarkKind::Highlight, "x", "")]);
    for (bad, why) in [
        (ok.replace("page = 1", "page = 0"), "start at 1"),
        (ok.replace("green", "purple"), "colour"),
        (ok.replace("highlight", "scribble"), "kind"),
        (
            ok.replace("rects = [[72.5, 80, 100.25, 14]]", "rects = []"),
            "rectangle",
        ),
        (format!("{ok}{ok}"), "used twice"),
        (format!("{ok}\nsurprise = 1\n"), ""),
        ("[[mark]]\nid = \"a\"".to_owned(), ""),
    ] {
        let e = parse_marks(&bad).expect_err(&bad);
        assert!(e.message.contains(why), "{why}: {}", e.message);
    }
}

#[test]
fn ids_and_paths() {
    assert_eq!(next_mark_id(&[]), 1);
    assert_eq!(next_mark_id(&[mark(7, 0, MarkKind::Note, "", "")]), 8);
    assert_eq!(marks_path("bio/cells.pdf"), "bio/cells.pdf.notes.toml");
}

#[test]
fn a_mark_is_found_again_on_its_page_or_on_the_page_the_text_moved_to() {
    let pages = [
        "intro".to_owned(),
        "the  Cell\ndivides".to_owned(),
        "other".to_owned(),
    ];
    let m = mark(1, 1, MarkKind::Highlight, "cell divides", "");
    assert_eq!(relocate(&m, &pages), Some(1));
    assert_eq!(
        relocate(
            &PdfMark {
                page: 0,
                ..m.clone()
            },
            &pages
        ),
        Some(1),
        "moved"
    );
    assert_eq!(
        relocate(
            &PdfMark {
                covered: "gone".into(),
                ..m.clone()
            },
            &pages
        ),
        None
    );
    let point = mark(2, 2, MarkKind::Note, "", "n");
    assert_eq!(
        relocate(&point, &pages),
        Some(2),
        "a point note stays on its page"
    );
    assert_eq!(relocate(&PdfMark { page: 9, ..point }, &pages), None);
}
