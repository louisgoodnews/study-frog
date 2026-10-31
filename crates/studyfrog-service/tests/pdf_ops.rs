#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! New PDFs from old ones: rearranged, merged, split, with notes and properties.

#[path = "../../studyfrog-pdf/tests/common/mod.rs"]
mod pdfmake;

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use studyfrog_config::pdf_marks::{MarkKind, MarkRect, PdfMark};
use studyfrog_pdf::doc::pdf_info;
use studyfrog_pdf::edit::{PageSpec, Properties};
use studyfrog_pdf::text::pdf_page_text;
use studyfrog_service::pdf_ops::{SaveRequest, pdf_page_total, save_pdf_copy, split_pdf};
use studyfrog_service::session::{ServiceError, VaultSession, open_vault};

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn session(label: &str) -> (Dir, VaultSession) {
    let dir = std::env::temp_dir().join(format!("studyfrog-ops-{label}-{}", std::process::id()));
    fs::create_dir_all(dir.join("bio")).unwrap();
    fs::write(dir.join("bio/three.pdf"), pdfmake::three()).unwrap();
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    (Dir(dir), s)
}

fn step(source: usize, page: usize, rotate: i32) -> PageSpec {
    PageSpec {
        source,
        page,
        rotate,
    }
}

fn first_word(path: &std::path::Path, page: usize) -> String {
    let b = Arc::new(fs::read(path).unwrap());
    pdf_page_text(&b, page).unwrap().words[0].text.clone()
}

fn request(out: &str, plan: Vec<PageSpec>) -> SaveRequest {
    SaveRequest {
        sources: vec!["bio/three.pdf".into()],
        plan,
        out: out.into(),
        marks: Vec::new(),
        props: None,
    }
}

#[test]
fn a_rearranged_copy_is_written_and_the_original_stays_as_it_was() {
    let (dir, s) = session("copy");
    let before = fs::read(dir.0.join("bio/three.pdf")).unwrap();
    let saved = save_pdf_copy(
        &s,
        &request("bio/new.pdf", vec![step(0, 2, 0), step(0, 0, 90)]),
    )
    .unwrap();
    assert_eq!((saved.path.as_str(), saved.pages), ("bio/new.pdf", 2));
    let out = dir.0.join("bio/new.pdf");
    assert_eq!(
        (first_word(&out, 0).as_str(), first_word(&out, 1).as_str()),
        ("Gamma", "Alpha")
    );
    assert_eq!(fs::read(dir.0.join("bio/three.pdf")).unwrap(), before);
    let e = save_pdf_copy(&s, &request("bio/new.pdf", vec![step(0, 0, 0)])).unwrap_err();
    assert!(matches!(e, ServiceError::Invalid(m) if m.contains("already exists")));
}

#[test]
fn names_outside_the_vault_or_not_pdf_or_in_missing_folders_are_refused() {
    let (_d, s) = session("names");
    for bad in [
        "../x.pdf",
        "/tmp/x.pdf",
        "bio/x.txt",
        "nowhere/x.pdf",
        "bio//x.pdf",
    ] {
        assert!(
            save_pdf_copy(&s, &request(bad, vec![step(0, 0, 0)])).is_err(),
            "{bad}"
        );
    }
    assert!(
        save_pdf_copy(&s, &request("bio/x.pdf", Vec::new())).is_err(),
        "no pages"
    );
    assert!(
        save_pdf_copy(&s, &request("bio/x.pdf", vec![step(0, 9, 0)])).is_err(),
        "no such page"
    );
}

#[test]
fn another_file_can_be_merged_and_its_pages_counted_first() {
    let (dir, s) = session("merge");
    let other = dir.0.join("other.pdf");
    fs::write(&other, pdfmake::make(&[vec![("Delta", 72.0, 700.0)]], None)).unwrap();
    let other_path = other.display().to_string();
    assert_eq!(pdf_page_total(&s, &other_path).unwrap(), 1);
    let mut req = request(
        "merged.pdf",
        vec![step(0, 0, 0), step(1, 0, 0), step(0, 1, 0)],
    );
    req.sources.push(other_path);
    save_pdf_copy(&s, &req).unwrap();
    let out = dir.0.join("merged.pdf");
    let words: Vec<String> = (0..3).map(|i| first_word(&out, i)).collect();
    assert_eq!(words, ["Alpha", "Delta", "Beta"]);
}

#[test]
fn highlights_follow_their_pages_and_properties_are_written() {
    let (dir, s) = session("marks");
    let mut req = request("marked.pdf", vec![step(0, 2, 0), step(0, 1, 0)]);
    req.marks = vec![PdfMark {
        id: 1,
        page: 1,
        kind: MarkKind::Highlight,
        color: "pink".into(),
        rects: vec![MarkRect {
            x: 72.0,
            y: 80.0,
            width: 50.0,
            height: 14.0,
        }],
        covered: "Beta".into(),
        note: "n".into(),
    }];
    req.props = Some(Properties {
        title: Some("Copy".into()),
        ..Properties::default()
    });
    let saved = save_pdf_copy(&s, &req).unwrap();
    assert_eq!(saved.skipped_marks, 0);
    let doc = lopdf::Document::load(dir.0.join("marked.pdf")).unwrap();
    let pages = doc.get_pages();
    let has_annots = |n: u32| {
        doc.get_dictionary(pages[&n])
            .unwrap()
            .get(b"Annots")
            .is_ok()
    };
    assert!(
        !has_annots(1) && has_annots(2),
        "the mark moved with page Beta to position 2"
    );
    let bytes = Arc::new(fs::read(dir.0.join("marked.pdf")).unwrap());
    assert_eq!(pdf_info(&bytes).unwrap().title.as_deref(), Some("Copy"));
}

#[test]
fn a_pdf_is_split_into_one_file_per_range_and_nothing_is_written_if_a_name_is_taken() {
    let (dir, s) = session("split");
    let names = split_pdf(&s, "bio/three.pdf", &[(0, 0), (1, 2)]).unwrap();
    assert_eq!(
        names,
        ["bio/three-pages-1-1.pdf", "bio/three-pages-2-3.pdf"]
    );
    let b = Arc::new(fs::read(dir.0.join(&names[1])).unwrap());
    assert_eq!(pdf_info(&b).unwrap().page_count, 2);
    assert!(
        split_pdf(&s, "bio/three.pdf", &[(0, 0), (2, 2)]).is_err(),
        "first name is taken"
    );
    assert!(!dir.0.join("bio/three-pages-3-3.pdf").exists());
}
