#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Cloze deletions: parsing, masking, revealing, numbering and the round trip (STORY-0108).

use proptest::prelude::*;
use studyfrog_md::cloze::{
    active_group, cloze_answers, cloze_groups, cloze_spans, last_cloze_index, mask_cloze,
    next_cloze_index, preview_cloze, reveal_cloze,
};
use studyfrog_md::parse::parse_doc;
use studyfrog_md::render::render_doc;

const TEXT: &str = "The {{c1::sun}} is a {{c2::star::kind of body}} and {{c1::hot}}.";
const GREEN: &str = "<span style=\"color:#1e8449\">";

#[test]
fn gaps_are_found_with_their_group_answer_and_hint() {
    let spans = cloze_spans(TEXT);
    let short: Vec<(u8, &str, Option<&str>)> = spans
        .iter()
        .map(|s| (s.index, s.answer.as_str(), s.hint.as_deref()))
        .collect();
    assert_eq!(
        short,
        [
            (1, "sun", None),
            (2, "star", Some("kind of body")),
            (1, "hot", None)
        ]
    );
    assert_eq!(cloze_groups(TEXT), [1, 2]);
    assert_eq!(&TEXT[spans[0].range.0..spans[0].range.1], "{{c1::sun}}");
}

#[test]
fn the_active_group_is_masked_and_the_others_read_as_text() {
    assert_eq!(mask_cloze(TEXT, 1), "The `[...]` is a star and `[...]`.");
    assert_eq!(
        mask_cloze(TEXT, 2),
        "The sun is a `[kind of body]` and hot."
    );
    assert_eq!(
        reveal_cloze(TEXT, 2),
        format!("The sun is a **{GREEN}star</span>** and hot.")
    );
    assert_eq!(cloze_answers(TEXT, 1), ["sun", "hot"]);
    assert!(preview_cloze(TEXT).matches(GREEN).count() == 3);
}

#[test]
fn text_that_only_looks_like_a_gap_stays_text() {
    for t in [
        "{{c0::zero}}",
        "{{c100::too big}}",
        "{{cx::no number}}",
        "{{c1::}}",
        "{{c1::open",
        "{{c1::across\nlines}}",
        "{{c1 no colons}}",
    ] {
        assert!(cloze_spans(t).is_empty(), "{t}");
        assert_eq!(mask_cloze(t, 1), t);
    }
}

#[test]
fn gaps_in_fenced_code_are_masked_without_code_marks() {
    let t = "```rust\nlet x = {{c1::vec![]}};\n```\n";
    assert_eq!(mask_cloze(t, 1), "```rust\nlet x = [...];\n```\n");
    assert_eq!(reveal_cloze(t, 1), "```rust\nlet x = vec![];\n```\n");
}

#[test]
fn groups_take_turns_unless_one_is_fixed() {
    let groups = [1, 2, 3];
    let asked: Vec<Option<u8>> = (0..4).map(|r| active_group(&groups, r, None)).collect();
    assert_eq!(asked, [Some(1), Some(2), Some(3), Some(1)]);
    assert_eq!(active_group(&groups, 0, Some(3)), Some(3));
    assert_eq!(
        active_group(&groups, 1, Some(9)),
        Some(2),
        "a missing group is ignored"
    );
    assert_eq!(active_group(&[], 0, None), None);
}

#[test]
fn a_new_gap_takes_the_next_number_or_the_last_one() {
    assert_eq!(next_cloze_index("plain"), 1);
    assert_eq!(last_cloze_index("plain"), 1);
    assert_eq!(next_cloze_index(TEXT), 3);
    assert_eq!(last_cloze_index(TEXT), 2);
    assert_eq!(next_cloze_index("{{c99::x}}"), 99);
}

fn answer() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 ,.()-]{1,12}".prop_filter("blank", |s| !s.trim().is_empty())
}

fn gapped() -> impl Strategy<Value = (String, Vec<(u8, String)>)> {
    prop::collection::vec((1_u8..=5, answer(), "[a-z ]{0,8}"), 1..6).prop_map(|parts| {
        let text = parts
            .iter()
            .map(|(n, a, filler)| format!("{filler}{{{{c{n}::{a}}}}}"))
            .collect();
        let gaps = parts.into_iter().map(|(n, a, _)| (n, a)).collect();
        (text, gaps)
    })
}

proptest! {
    #[test]
    fn any_text_is_safe_to_mask_and_reveal(text in "(\\PC|\n){0,120}", active in 1_u8..=99) {
        let _ = mask_cloze(&text, active);
        let _ = reveal_cloze(&text, active);
        let _ = next_cloze_index(&text);
    }

    #[test]
    fn masking_hides_exactly_the_active_answers((text, gaps) in gapped(), pick in 0_usize..6) {
        let active = gaps[pick % gaps.len()].0;
        let masked = mask_cloze(&text, active);
        prop_assert!(!masked.contains("{{c"));
        let hidden = gaps.iter().filter(|(n, _)| *n == active).count();
        prop_assert_eq!(masked.matches("`[...]`").count(), hidden);
        let expected: Vec<String> = gaps
            .iter()
            .filter(|(n, _)| *n == active)
            .map(|(_, a)| a.trim().to_owned())
            .collect();
        prop_assert_eq!(cloze_answers(&text, active), expected);
    }

    #[test]
    fn a_card_and_a_note_with_gaps_survive_the_round_trip((text, _) in gapped()) {
        let card = format!("---\nid: 0000000000000000000000000A\nkind: card\ntitle: T\n---\n{text}\n???\nextra\n");
        let doc = parse_doc(&card).unwrap();
        prop_assert_eq!(parse_doc(&render_doc(&doc).unwrap()).unwrap(), doc.clone());
        let note = format!("---\nid: 0000000000000000000000000B\nkind: note\ntitle: N\n---\n{text}\n");
        let doc = parse_doc(&note).unwrap();
        let again = render_doc(&doc).unwrap();
        prop_assert!(again.contains(&text));
        prop_assert_eq!(parse_doc(&again).unwrap(), doc);
    }
}

#[test]
fn a_title_can_blank_every_gap() {
    assert_eq!(
        studyfrog_md::cloze::blank_cloze(TEXT),
        "The [...] is a [...] and [...]."
    );
}
