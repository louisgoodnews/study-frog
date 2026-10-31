#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Links on a page: addresses and pages, nothing that runs.

mod common;

use common::{bytes, linked};
use studyfrog_pdf::links::{LinkTarget, pdf_page_links};

#[test]
fn addresses_and_page_links_are_found_with_boxes_from_the_top_left_and_other_actions_are_ignored() {
    let links = pdf_page_links(&bytes(linked()), 0).unwrap();
    let targets: Vec<&LinkTarget> = links.iter().map(|l| &l.target).collect();
    assert_eq!(
        targets,
        [
            &LinkTarget::Uri("https://example.org/x".into()),
            &LinkTarget::Page(2),
            &LinkTarget::Page(1)
        ]
    );
    let first = &links[0];
    assert_eq!(
        (first.x, first.y, first.width, first.height),
        (72.0, 792.0 - 720.0, 100.0, 20.0)
    );
}

#[test]
fn a_page_without_links_gives_none_and_a_missing_page_is_an_error() {
    assert!(pdf_page_links(&bytes(linked()), 1).unwrap().is_empty());
    assert!(pdf_page_links(&bytes(linked()), 9).is_err());
    assert!(pdf_page_links(&bytes(b"nope".to_vec()), 0).is_err());
    assert!(
        pdf_page_links(&bytes(common::three()), 0)
            .unwrap()
            .is_empty()
    );
}
