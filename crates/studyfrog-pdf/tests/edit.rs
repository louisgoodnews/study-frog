#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Page operations, properties and annotations in a copy.

mod common;

use common::{bytes, make, three};
use studyfrog_pdf::annotate::{Mark, MarkKind, pdf_with_marks};
use studyfrog_pdf::doc::pdf_info;
use studyfrog_pdf::edit::{
    PageSpec, Properties, parse_page_ranges, pdf_build, pdf_extract_ranges, pdf_set_properties,
};
use studyfrog_pdf::render::PageRect;
use studyfrog_pdf::text::pdf_page_text;

fn first_word(b: &[u8], page: usize) -> String {
    pdf_page_text(&bytes(b.to_vec()), page).unwrap().words[0]
        .text
        .clone()
}

fn spec(source: usize, page: usize, rotate: i32) -> PageSpec {
    PageSpec {
        source,
        page,
        rotate,
    }
}

#[test]
fn pages_can_be_reordered_dropped_and_repeated() {
    let src = bytes(three());
    let out = pdf_build(std::slice::from_ref(&src), &[spec(0, 2, 0), spec(0, 0, 0)]).unwrap();
    assert_eq!(pdf_info(&bytes(out.clone())).unwrap().page_count, 2);
    assert_eq!(
        (first_word(&out, 0).as_str(), first_word(&out, 1).as_str()),
        ("Gamma", "Alpha")
    );
    assert!(
        out.len() < src.len() + 2000,
        "the dropped page's objects are not carried along"
    );
}

#[test]
fn a_page_is_rotated_in_the_copy_only() {
    let src = bytes(three());
    let out = pdf_build(std::slice::from_ref(&src), &[spec(0, 0, 90), spec(0, 1, 0)]).unwrap();
    let info = pdf_info(&bytes(out.clone())).unwrap();
    assert_eq!(
        (info.pages[0].width, info.pages[0].height),
        (792.0, 612.0),
        "turned"
    );
    assert_eq!((info.pages[1].width, info.pages[1].height), (612.0, 792.0));
    let w = &pdf_page_text(&bytes(out), 0).unwrap().words[0];
    assert!(
        w.x >= 0.0 && w.y >= 0.0 && w.x + w.width <= 792.0 && w.y + w.height <= 612.0,
        "the box follows the page: {w:?}"
    );
    assert_eq!(
        pdf_info(&src).unwrap().pages[0].width,
        612.0,
        "the original is unchanged"
    );
}

#[test]
fn files_are_merged_in_the_order_of_the_plan() {
    let a = bytes(three());
    let b = bytes(make(&[vec![("Delta other file", 72.0, 700.0)]], None));
    let out = pdf_build(
        &[a, b],
        &[spec(0, 0, 0), spec(1, 0, 0), spec(0, 1, 0), spec(0, 2, 0)],
    )
    .unwrap();
    let words: Vec<String> = (0..4).map(|i| first_word(&out, i)).collect();
    assert_eq!(words, ["Alpha", "Delta", "Beta", "Gamma"]);
}

#[test]
fn ranges_are_parsed_checked_and_used_to_split() {
    assert_eq!(parse_page_ranges("1-2, 3", 3).unwrap(), [(0, 1), (2, 2)]);
    for bad in ["", "0", "2-1", "4", "a", "1-9"] {
        assert!(parse_page_ranges(bad, 3).is_err(), "{bad:?}");
    }
    let out = pdf_extract_ranges(&bytes(three()), &[(1, 2)]).unwrap();
    assert_eq!(
        (
            pdf_info(&bytes(out.clone())).unwrap().page_count,
            first_word(&out, 0).as_str()
        ),
        (2, "Beta")
    );
}

#[test]
fn a_bad_plan_is_refused() {
    let src = bytes(three());
    assert!(pdf_build(std::slice::from_ref(&src), &[]).is_err());
    assert!(
        pdf_build(std::slice::from_ref(&src), &[spec(1, 0, 0)]).is_err(),
        "no such source"
    );
    assert!(
        pdf_build(std::slice::from_ref(&src), &[spec(0, 0, 45)]).is_err(),
        "only quarter turns"
    );
    assert!(
        pdf_build(std::slice::from_ref(&src), &[spec(0, 9, 0)]).is_err(),
        "no such page"
    );
}

#[test]
fn properties_are_written_and_unicode_survives() {
    let out = pdf_set_properties(
        &bytes(three()),
        &Properties {
            title: Some("Übung 日本".into()),
            author: None,
            subject: Some("Maths".into()),
        },
    )
    .unwrap();
    let info = pdf_info(&bytes(out)).unwrap();
    assert_eq!(
        (
            info.title.as_deref(),
            info.author.as_deref(),
            info.subject.as_deref()
        ),
        (Some("Übung 日本"), Some("Ada"), Some("Maths"))
    );
    let plain = make(&[vec![("x", 72.0, 700.0)]], None);
    let out = pdf_set_properties(
        &bytes(plain),
        &Properties {
            title: Some("New".into()),
            ..Properties::default()
        },
    )
    .unwrap();
    assert_eq!(
        pdf_info(&bytes(out)).unwrap().title.as_deref(),
        Some("New"),
        "an Info dictionary is created when there is none"
    );
}

fn mark(page: usize, kind: MarkKind) -> Mark {
    Mark {
        page,
        kind,
        rects: vec![PageRect {
            x: 72.0,
            y: 80.0,
            width: 100.0,
            height: 14.0,
        }],
        color: [1.0, 0.9, 0.2],
        contents: "Maths: key idea".into(),
    }
}

#[test]
fn highlights_and_notes_become_standard_annotations_with_flipped_coordinates() {
    let copy = pdf_with_marks(
        &bytes(three()),
        &[
            mark(0, MarkKind::Highlight),
            mark(0, MarkKind::Note),
            mark(2, MarkKind::Highlight),
        ],
        None,
    )
    .unwrap();
    assert_eq!(copy.skipped, 0);
    let doc = lopdf::Document::load_mem(&copy.bytes).unwrap();
    let pages = doc.get_pages();
    let annots = |n: u32| -> Vec<lopdf::Dictionary> {
        let page = doc.get_dictionary(pages[&n]).unwrap();
        page.get(b"Annots")
            .ok()
            .and_then(|a| a.as_array().ok())
            .map(|a| {
                a.iter()
                    .map(|r| {
                        doc.get_dictionary(r.as_reference().unwrap())
                            .unwrap()
                            .clone()
                    })
                    .collect()
            })
            .unwrap_or_default()
    };
    let first = annots(1);
    assert_eq!(first.len(), 2);
    assert_eq!(
        first[0].get(b"Subtype").unwrap().as_name().unwrap(),
        b"Highlight"
    );
    assert_eq!(
        first[1].get(b"Subtype").unwrap().as_name().unwrap(),
        b"Text"
    );
    let rect: Vec<f32> = first[0]
        .get(b"Rect")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_float().unwrap())
        .collect();
    assert_eq!(
        rect,
        [72.0, 792.0 - 94.0, 172.0, 792.0 - 80.0],
        "top-left points become bottom-left points"
    );
    assert_eq!(
        first[0]
            .get(b"QuadPoints")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        8
    );
    assert!(annots(2).is_empty() && annots(3).len() == 1);
}

#[test]
fn marks_on_rotated_pages_are_left_out_and_counted_and_the_file_still_reads() {
    let turned = pdf_build(&[bytes(three())], &[spec(0, 0, 90), spec(0, 1, 0)]).unwrap();
    let copy = pdf_with_marks(
        &bytes(turned),
        &[mark(0, MarkKind::Highlight), mark(1, MarkKind::Highlight)],
        Some(&Properties {
            title: Some("Marked".into()),
            ..Properties::default()
        }),
    )
    .unwrap();
    assert_eq!(copy.skipped, 1);
    let info = pdf_info(&bytes(copy.bytes.clone())).unwrap();
    assert_eq!(
        (info.page_count, info.title.as_deref()),
        (2, Some("Marked"))
    );
    assert!(pdf_with_marks(&bytes(three()), &[mark(9, MarkKind::Note)], None).is_err());
}
