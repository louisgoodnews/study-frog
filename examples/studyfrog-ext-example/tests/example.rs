#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The example extension, through the registry like the application uses it.

use proptest::prelude::*;
use studyfrog_ext::registry::{find_card_type, importer_for_file, registry_new};
use studyfrog_ext_example::cloze::expand_cloze;
use studyfrog_ext_example::register;
use studyfrog_ext_example::tsv::import_tsv;

#[test]
fn registering_adds_both_extension_points_once() {
    let mut r = registry_new();
    register(&mut r).unwrap();
    assert_eq!(
        importer_for_file(&r, "deck.tsv").unwrap().id,
        "tab-separated"
    );
    assert_eq!(
        find_card_type(&r, "cloze").unwrap().name,
        "Cloze (fill in the gaps)"
    );
    assert!(
        register(&mut r).unwrap_err().contains("already registered"),
        "twice is refused"
    );
}

#[test]
fn tab_separated_text_becomes_cards_with_tags_and_skips_headers() {
    let text = "#separator:tab\n#html:true\n\nCapital of France?\tParis\tgeography europe\nLine one<br>two\tAnswer\n";
    let cards = import_tsv(text).unwrap();
    assert_eq!(cards.len(), 2);
    assert_eq!(
        (cards[0].front.as_str(), cards[0].back.as_str()),
        ("Capital of France?", "Paris")
    );
    assert_eq!(cards[0].tags, ["geography", "europe"]);
    assert_eq!(cards[1].front, "Line one\ntwo");
    assert!(cards[1].tags.is_empty());
    assert!(import_tsv("").unwrap().is_empty());
}

#[test]
fn a_bad_line_is_reported_with_its_number() {
    let e = import_tsv("a\tb\nonly one column\nc\td\n").unwrap_err();
    assert!(e.starts_with("line 2:"), "{e}");
    assert!(
        import_tsv("front\t\n").unwrap_err().contains("line 1"),
        "an empty back is refused"
    );
}

#[test]
fn a_cloze_note_becomes_one_card_per_gap() {
    let cards =
        expand_cloze("The {{c1::mitochondria}} is the {{c2::powerhouse::energy}} of the cell")
            .unwrap();
    assert_eq!(cards.len(), 2);
    assert_eq!(cards[0].front, "The [...] is the powerhouse of the cell");
    assert_eq!(
        cards[0].back,
        "The **mitochondria** is the powerhouse of the cell"
    );
    assert_eq!(
        cards[1].front, "The mitochondria is the [energy] of the cell",
        "the hint is shown"
    );
    assert_eq!(
        cards[1].back,
        "The mitochondria is the **powerhouse** of the cell"
    );
}

#[test]
fn gaps_with_the_same_number_hide_together() {
    let cards = expand_cloze("{{c1::Paris}} is the capital of {{c1::France}}").unwrap();
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].front, "[...] is the capital of [...]");
}

#[test]
fn broken_cloze_sources_are_explained() {
    for (src, needle) in [
        ("no gaps here", "no gaps found"),
        ("open {{c1::never closed", "never closed"),
        ("{{c0::x}}", "start at 1"),
        ("{{cx::x}}", "not a gap number"),
        ("{{c1::}}", "no answer"),
        ("{{d1::x}}", "starts with c"),
        ("{{c1 x}}", "needs ::"),
    ] {
        assert!(expand_cloze(src).unwrap_err().contains(needle), "{src}");
    }
}

proptest! {
    #[test]
    fn nothing_panics_and_a_front_never_leaks_its_own_gap(src in "[a-z {}:c0-9]{0,40}") {
        let _ = import_tsv(&src);
        if let Ok(cards) = expand_cloze(&src) {
            prop_assert!(!cards.is_empty());
            prop_assert!(cards.iter().all(|c| !c.front.contains("**")));
        }
    }
}
