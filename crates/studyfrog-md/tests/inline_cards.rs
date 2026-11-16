//! Integration tests for inline card delimiter scanner (TASK-0269).

use studyfrog_md::inline_cards::{
    InlineCard, contains_inline_cards, scan_inline_cards, strip_inline_card_markers,
};

#[test]
fn test_standard_inline_card() {
    let md = "Photosynthesis :: The process by which plants make food";
    let cards = scan_inline_cards(md);
    assert_eq!(cards.len(), 1);
    assert_eq!(
        cards[0],
        InlineCard {
            term: "Photosynthesis".to_string(),
            definition: "The process by which plants make food".to_string(),
            line_number: 1,
            raw: md.to_string(),
        }
    );
}

#[test]
fn test_bulleted_cards() {
    let md = "- Front A :: Back A\n* Front B :: Back B\n+ Front C :: Back C\n1. Front D :: Back D";
    let cards = scan_inline_cards(md);
    assert_eq!(cards.len(), 4);
    assert_eq!(cards[0].term, "Front A");
    assert_eq!(cards[0].definition, "Back A");
    assert_eq!(cards[0].line_number, 1);
    assert_eq!(cards[0].raw, "- Front A :: Back A");

    assert_eq!(cards[1].term, "Front B");
    assert_eq!(cards[1].line_number, 2);

    assert_eq!(cards[2].term, "Front C");
    assert_eq!(cards[2].line_number, 3);

    assert_eq!(cards[3].term, "Front D");
    assert_eq!(cards[3].line_number, 4);
}

#[test]
fn test_reversed_cards() {
    let md = "Mitosis ::: Cell division resulting in identical cells";
    let cards = scan_inline_cards(md);
    assert_eq!(cards.len(), 2);

    assert_eq!(cards[0].term, "Mitosis");
    assert_eq!(
        cards[0].definition,
        "Cell division resulting in identical cells"
    );
    assert_eq!(cards[0].line_number, 1);
    assert_eq!(cards[0].raw, md);

    assert_eq!(cards[1].term, "Cell division resulting in identical cells");
    assert_eq!(cards[1].definition, "Mitosis");
    assert_eq!(cards[1].line_number, 1);
    assert_eq!(cards[1].raw, md);
}

#[test]
fn test_code_fences_are_ignored() {
    let md = "```rust\nlet x = foo :: bar;\n```\nReal Term :: Real Def\n~~~python\na :: b\n~~~";
    let cards = scan_inline_cards(md);
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].term, "Real Term");
    assert_eq!(cards[0].definition, "Real Def");
    assert_eq!(cards[0].line_number, 4);
}

#[test]
fn test_inline_code_handling() {
    let md = "Here is `foo :: bar` in code.\n`std::fmt::Display` :: Trait for formatting\n`a :: b`";
    let cards = scan_inline_cards(md);
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].term, "`std::fmt::Display`");
    assert_eq!(cards[0].definition, "Trait for formatting");
    assert_eq!(cards[0].line_number, 2);
}

#[test]
fn test_math_and_tables_and_headings_ignored() {
    let md = "# Heading :: Subtitle\n$$\nx :: y\n$$\n| Col 1 | Col 2 :: Def |";
    let cards = scan_inline_cards(md);
    assert!(cards.is_empty());
}

#[test]
fn test_empty_term_or_def_ignored() {
    let md = ":: Def only\nTerm only ::\n- :: \n   ::   ";
    let cards = scan_inline_cards(md);
    assert!(cards.is_empty());
}

#[test]
fn test_cpp_rust_path_collision_avoided() {
    let md = "std::collections::HashMap is a type\nstd::vec::Vec :: Dynamic array";
    let cards = scan_inline_cards(md);
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].term, "std::vec::Vec");
    assert_eq!(cards[0].definition, "Dynamic array");
    assert_eq!(cards[0].line_number, 2);
}

#[test]
fn test_multiple_delimiters_on_line() {
    let md = "First :: Second :: Third";
    let cards = scan_inline_cards(md);
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].term, "First");
    assert_eq!(cards[0].definition, "Second :: Third");
}

#[test]
fn test_contains_inline_cards() {
    assert!(contains_inline_cards("A :: B"));
    assert!(contains_inline_cards("- A ::: B"));
    assert!(!contains_inline_cards("Just standard text"));
    assert!(!contains_inline_cards("```\nA :: B\n```"));
    assert!(!contains_inline_cards("`A :: B`"));
}

#[test]
fn test_strip_inline_card_markers() {
    let input = "# Lecture Notes\n\n- Photosynthesis :: Plant energy process\nMitosis ::: Cell division\nNormal text line\n```\nfoo :: bar\n```\n`code :: inline`\n";
    let expected = "# Lecture Notes\n\n- Photosynthesis: Plant energy process\nMitosis: Cell division\nNormal text line\n```\nfoo :: bar\n```\n`code :: inline`\n";
    assert_eq!(strip_inline_card_markers(input), expected);
}
