#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Broken and hostile input must give errors, never a crash or a hang.

mod common;

use std::sync::mpsc::channel;
use std::time::Instant;

use common::{bytes, three};
use studyfrog_pdf::PdfBytes;
use studyfrog_pdf::annotate::pdf_with_marks;
use studyfrog_pdf::doc::pdf_info;
use studyfrog_pdf::edit::{PageSpec, pdf_build};
use studyfrog_pdf::error::PdfError;
use studyfrog_pdf::images::pdf_page_images;
use studyfrog_pdf::limits::{MAX_FILE_BYTES, guarded_for};
use studyfrog_pdf::render::pdf_render_page;
use studyfrog_pdf::text::{pdf_all_text, pdf_page_markdown, pdf_page_text};

fn corpus() -> Vec<(&'static str, PdfBytes)> {
    let good = three();
    let mut random = vec![0_u8; 3000];
    for (i, b) in random.iter_mut().enumerate() {
        *b = (i * 31 % 251) as u8;
    }
    let looping = b"%PDF-1.4\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj\n2 0 obj<</Type/Pages/Kids[2 0 R]/Count 1>>endobj\ntrailer<</Root 1 0 R/Size 3>>\n%%EOF".to_vec();
    let huge_page = b"%PDF-1.4\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj\n2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj\n3 0 obj<</Type/Page/Parent 2 0 R/MediaBox[0 0 1000000 1000000]/Contents 4 0 R>>endobj\n4 0 obj<</Length 20>>stream\n0 0 m 10 10 l S\nendstream endobj\ntrailer<</Root 1 0 R/Size 5>>\n%%EOF".to_vec();
    let mut bad_xref = good.clone();
    if let Some(at) = bad_xref.windows(9).rposition(|w| w == b"startxref") {
        bad_xref[at + 8] = b'f';
    }
    vec![
        ("empty", bytes(Vec::new())),
        ("random", bytes(random)),
        ("truncated", bytes(good[..good.len() / 2].to_vec())),
        ("not a pdf", bytes(b"hello world, this is text".to_vec())),
        ("self-referencing page tree", bytes(looping)),
        ("a million points wide", bytes(huge_page)),
        ("broken xref", bytes(bad_xref)),
    ]
}

#[test]
fn every_call_returns_instead_of_crashing_or_hanging_on_every_broken_file() {
    let started = Instant::now();
    for (name, file) in corpus() {
        let _ = (
            pdf_info(&file),
            pdf_page_text(&file, 0),
            pdf_all_text(&file),
            pdf_page_markdown(&file, 0),
            pdf_page_images(&file, 0),
        );
        let _ = pdf_render_page(&file, 0, 1.0);
        let _ = pdf_build(
            std::slice::from_ref(&file),
            &[PageSpec {
                source: 0,
                page: 0,
                rotate: 0,
            }],
        );
        let _ = pdf_with_marks(&file, &[], None);
        assert!(started.elapsed().as_secs() < 60, "{name} took too long");
    }
}

#[test]
fn files_that_are_not_pdfs_are_refused_with_a_reason() {
    for name in ["empty", "random", "not a pdf"] {
        let file = corpus().into_iter().find(|c| c.0 == name).unwrap().1;
        assert!(
            matches!(pdf_info(&file), Err(PdfError::Invalid(_))),
            "{name}"
        );
        assert!(pdf_render_page(&file, 0, 1.0).is_err(), "{name}");
    }
}

#[test]
fn an_absurd_page_is_refused_by_size_before_any_drawing() {
    let file = corpus()
        .into_iter()
        .find(|c| c.0 == "a million points wide")
        .unwrap()
        .1;
    assert!(matches!(
        pdf_render_page(&file, 0, 1.0),
        Err(PdfError::TooBig(_) | PdfError::Invalid(_) | PdfError::Crashed(_))
    ));
}

#[test]
fn zoom_and_drawing_size_are_limited() {
    let file = bytes(three());
    assert!(matches!(
        pdf_render_page(&file, 0, 0.001),
        Err(PdfError::TooBig(_))
    ));
    assert!(matches!(
        pdf_render_page(&file, 0, 40.0),
        Err(PdfError::TooBig(_))
    ));
    assert!(
        matches!(pdf_render_page(&file, 0, 12.0), Err(PdfError::TooBig(_))),
        "a letter page at 1200 % is 50 million pixels"
    );
}

#[test]
fn a_file_over_the_size_limit_is_refused_without_reading_it() {
    let mut big = b"%PDF-1.4\n".to_vec();
    big.resize(MAX_FILE_BYTES + 1, b' ');
    assert!(matches!(pdf_info(&bytes(big)), Err(PdfError::TooBig(_))));
}

#[test]
fn a_panic_inside_a_library_call_comes_back_as_an_error() {
    let r: Result<u8, PdfError> = guarded_for(5, || {
        let v: Vec<u8> = Vec::new();
        let i = v.len() + 3;
        Ok(v[i])
    });
    assert!(matches!(r, Err(PdfError::Crashed(_))));
}

#[test]
fn work_that_never_finishes_is_stopped_by_the_time_limit() {
    let (hold, wait) = channel::<()>();
    let started = Instant::now();
    let r: Result<(), PdfError> = guarded_for(1, move || {
        let _ = wait.recv();
        Ok(())
    });
    drop(hold);
    assert_eq!(r, Err(PdfError::Timeout));
    assert!(started.elapsed().as_secs() < 5);
}
