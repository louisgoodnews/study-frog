#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! STORY-0145: Alphabetical and parenthesized list markers.

use studyfrog_md::blocks::{Block, parse_blocks};

#[test]
fn alphabetical_and_parenthesized_lists_parse_into_structured_list_items() {
    let md = "a. First item\nb. Second item\nc. Third item\n";
    let blocks = parse_blocks(md);
    assert_eq!(blocks.len(), 1);
    let Block::List { items, ordered, .. } = &blocks[0] else {
        panic!("expected list, got {:?}", blocks[0]);
    };
    assert!(ordered);
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].marker.as_deref(), Some("a."));
    assert_eq!(items[1].marker.as_deref(), Some("b."));
    assert_eq!(items[2].marker.as_deref(), Some("c."));

    let md_paren = "(a) Choice A\n(b) Choice B\n";
    let blocks_paren = parse_blocks(md_paren);
    let Block::List { items: p_items, .. } = &blocks_paren[0] else {
        panic!("expected paren list");
    };
    assert_eq!(p_items.len(), 2);
    assert_eq!(p_items[0].marker.as_deref(), Some("(a)"));
    assert_eq!(p_items[1].marker.as_deref(), Some("(b)"));
}
