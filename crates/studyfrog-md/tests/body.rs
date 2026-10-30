#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Body scanners: `???` splitting and wikilink extraction.

use proptest::prelude::*;
use studyfrog_md::links::extract_wikilinks;
use studyfrog_md::sections::{join_card_body, split_card_body};

#[test]
fn separator_splits_once_and_ignores_code() {
    assert_eq!(
        split_card_body("q\n???\na\n???\nmore"),
        Some(("q", "a\n???\nmore"))
    );
    assert_eq!(split_card_body("???\nonly back"), Some(("", "only back")));
    assert_eq!(split_card_body("q\r\n???\r\na"), Some(("q\r", "a")));
    assert_eq!(split_card_body("```\n???\n```\nno separator"), None);
    assert_eq!(
        split_card_body("```\ncode\n```\n???\nback"),
        Some(("```\ncode\n```", "back"))
    );
    assert_eq!(split_card_body("what is ???\nno"), None);
    assert_eq!(split_card_body("no separator"), None);
}

#[test]
fn wikilink_forms() {
    let body = "See [[Alpha]], [[Beta|the label]] and [[Gamma#Part]] or [[Delta#H|L]].\n";
    let links = extract_wikilinks(body);
    let shown: Vec<_> = links
        .iter()
        .map(|l| (l.target, l.heading, l.label, l.line))
        .collect();
    assert_eq!(
        shown,
        [
            ("Alpha", None, None, 1),
            ("Beta", None, Some("the label"), 1),
            ("Gamma", Some("Part"), None, 1),
            ("Delta", Some("H"), Some("L"), 1),
        ]
    );
}

#[test]
fn wikilinks_in_code_escapes_and_junk_are_ignored() {
    let body = "`[[inline]]` and ``[[double `tick`]]`` \\[[escaped]] [[ ]] [[#only heading]] [[a[b]]\n```\n[[fenced]]\n```\n~~~\n[[tilde]]\n~~~\n[[real]]\n[[unclosed\n";
    let targets: Vec<_> = extract_wikilinks(body).iter().map(|l| l.target).collect();
    assert_eq!(targets, ["real"]);
}

proptest! {
    #[test]
    fn join_then_split_is_identity_when_unambiguous(front in "[a-z \n]{0,20}", back in "(\\PC|\n){0,30}") {
        let body = join_card_body(&front, &back);
        if let Some((_, b)) = split_card_body(&body).filter(|(f, _)| *f == front) {
            prop_assert_eq!(b, back.as_str());
        }
    }

    #[test]
    fn link_extraction_never_panics_and_borrows_from_input(body in "(\\PC|\n|\\[|\\]|`){0,120}") {
        for link in extract_wikilinks(&body) {
            prop_assert!(body.contains(link.target));
            prop_assert!(!link.target.is_empty());
        }
    }
}
