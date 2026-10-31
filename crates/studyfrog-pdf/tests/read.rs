#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Reading and drawing PDFs.

mod common;

use common::{bytes, make, make_scan, three};
use studyfrog_pdf::doc::pdf_info;
use studyfrog_pdf::error::{PdfError, describe_pdf_error};
use studyfrog_pdf::images::pdf_page_images;
use studyfrog_pdf::render::{PageRect, pdf_render_page, pdf_render_region, pixels_to_png};
use studyfrog_pdf::text::{pdf_all_text, pdf_page_markdown, pdf_page_text};

#[test]
fn info_gives_pages_sizes_title_and_author() {
    let info = pdf_info(&bytes(three())).unwrap();
    assert_eq!(info.page_count, 3);
    assert_eq!((info.pages[0].width, info.pages[0].height), (612.0, 792.0));
    assert_eq!(
        (info.title.as_deref(), info.author.as_deref(), info.subject),
        (Some("Three pages"), Some("Ada"), None)
    );
    assert!(info.outline.is_empty());
}

#[test]
fn words_come_with_boxes_measured_from_the_top_left() {
    let t = pdf_page_text(&bytes(three()), 1).unwrap();
    let words: Vec<&str> = t.words.iter().map(|w| w.text.as_str()).collect();
    assert_eq!(words, ["Beta", "second", "page"]);
    assert!(t.text.contains("Beta second page") && !t.image_only);
    let w = &t.words[0];
    assert!((w.x - 72.0).abs() < 3.0, "x {}", w.x);
    assert!(
        w.y > 60.0 && w.y < 110.0,
        "the text sits 92 points from the top of a 792 point page, not near the bottom: y {}",
        w.y
    );
    assert!(w.width > 10.0 && w.height > 5.0);
    assert_eq!(
        pdf_page_text(&bytes(three()), 7),
        Err(PdfError::NoSuchPage(7))
    );
}

#[test]
fn all_text_and_markdown_and_scanned_pages() {
    let all = pdf_all_text(&bytes(three())).unwrap();
    assert_eq!(all.len(), 3);
    assert!(all[2].text.contains("Gamma"));
    let md = pdf_page_markdown(&bytes(three()), 0).unwrap();
    assert!(md.contains("Alpha first page"), "{md}");
    let scan = pdf_page_text(&bytes(make_scan()), 0).unwrap();
    assert!(scan.image_only && scan.words.is_empty());
    assert!(pdf_all_text(&bytes(make_scan())).unwrap()[0].image_only);
}

#[test]
fn a_page_is_drawn_at_the_zoom_asked_for_with_dark_text_on_white() {
    let doc = bytes(three());
    let p = pdf_render_page(&doc, 0, 1.0).unwrap();
    assert_eq!((p.width, p.height), (612, 792));
    assert_eq!(p.rgba.len(), 612 * 792 * 4);
    assert_eq!(
        &p.rgba[0..4],
        &[255, 255, 255, 255],
        "the corner is white and opaque"
    );
    assert!(
        p.rgba.as_chunks::<4>().0.iter().any(|px| px[0] < 100),
        "some pixels are dark where the text is"
    );
    let big = pdf_render_page(&doc, 0, 2.0).unwrap();
    assert_eq!((big.width, big.height), (1224, 1584));
    let png = pixels_to_png(&p).unwrap();
    assert_eq!(&png[..4], b"\x89PNG");
}

#[test]
fn a_region_is_cut_out_of_the_drawing() {
    let doc = bytes(make(&[vec![("Region text", 72.0, 700.0)]], None));
    let r = pdf_render_region(
        &doc,
        0,
        PageRect {
            x: 60.0,
            y: 60.0,
            width: 200.0,
            height: 60.0,
        },
        2.0,
    )
    .unwrap();
    assert_eq!((r.width, r.height), (400, 120));
    assert!(r.rgba.as_chunks::<4>().0.iter().any(|px| px[0] < 100));
    assert!(
        pdf_render_region(
            &doc,
            0,
            PageRect {
                x: 5000.0,
                y: 0.0,
                width: 10.0,
                height: 10.0
            },
            1.0
        )
        .is_err(),
        "outside the page"
    );
}

#[test]
fn pictures_on_a_page_come_out_as_png_files() {
    let imgs = pdf_page_images(&bytes(make_scan()), 0).unwrap();
    assert_eq!(imgs.len(), 1);
    assert_eq!((imgs[0].width, imgs[0].height), (64, 64));
    assert_eq!(&imgs[0].png[..4], b"\x89PNG");
    assert!(pdf_page_images(&bytes(three()), 0).unwrap().is_empty());
}

#[test]
fn errors_are_sentences() {
    for e in [
        PdfError::TooBig("x".into()),
        PdfError::Invalid("y".into()),
        PdfError::Encrypted,
        PdfError::NoSuchPage(0),
        PdfError::Timeout,
        PdfError::Crashed("z".into()),
    ] {
        assert!(describe_pdf_error(&e).ends_with('.'));
    }
}
