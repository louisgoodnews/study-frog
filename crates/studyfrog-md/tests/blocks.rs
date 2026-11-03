#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The preview block model.

use proptest::prelude::*;
use studyfrog_md::blocks::{Block, Link, ListItem, Span, Style, parse_blocks, plain_text};
use studyfrog_md::links::{Piece, split_wikilinks};

fn paragraph(blocks: &[Block], i: usize) -> &[Span] {
    match &blocks[i] {
        Block::Paragraph(s) | Block::Heading { spans: s, .. } => s,
        other => panic!("not text: {other:?}"),
    }
}

#[test]
fn headings_paragraphs_rules_and_emphasis() {
    let blocks = parse_blocks("# Title\n\nSome **bold**, *italic*, ~~gone~~ and `code`.\n\n---\n");
    assert!(matches!(&blocks[0], Block::Heading { level: 1, .. }));
    assert_eq!(plain_text(paragraph(&blocks, 0)), "Title");
    let spans = paragraph(&blocks, 1);
    let find = |t: &str| spans.iter().find(|s| s.text == t).unwrap().style;
    assert!(find("bold").bold && find("italic").italic && find("gone").strike && find("code").code);
    assert!(!find("Some ").bold);
    assert_eq!(blocks[2], Block::Rule);
}

#[test]
fn wikilinks_become_link_spans_in_text_but_not_in_code() {
    let blocks = parse_blocks(
        "See [[Alpha]] and [[Beta#Part|the label]] and `[[Gamma]]`.\n\n```\n[[Delta]]\n```\n",
    );
    let spans = paragraph(&blocks, 0);
    let links: Vec<_> = spans
        .iter()
        .filter_map(|s| s.link.as_ref().map(|l| (l.clone(), s.text.as_str())))
        .collect();
    assert_eq!(
        links,
        [
            (Link::Wiki("Alpha".into()), "Alpha"),
            (Link::Wiki("Beta".into()), "the label")
        ]
    );
    assert!(
        spans
            .iter()
            .any(|s| s.text == "[[Gamma]]" && s.style.code && s.link.is_none()),
        "inline code is literal"
    );
    assert_eq!(
        blocks[1],
        Block::Code {
            lang: String::new(),
            text: "[[Delta]]\n".into()
        }
    );
}

#[test]
fn wikilinks_inside_emphasis_keep_the_style_and_url_links_stay_urls() {
    let blocks = parse_blocks("**bold [[Linked]] text** and [site [[x]]](https://example.org)");
    let spans = paragraph(&blocks, 0);
    let linked = spans
        .iter()
        .find(|s| s.link == Some(Link::Wiki("Linked".into())))
        .unwrap();
    assert!(linked.style.bold);
    let url = spans
        .iter()
        .filter(|s| matches!(&s.link, Some(Link::Url(u)) if u == "https://example.org"))
        .map(|s| s.text.as_str())
        .collect::<String>();
    assert_eq!(
        url, "site [[x]]",
        "no wikilinks are made inside an ordinary link"
    );
}

#[test]
fn lists_nest_and_task_items_carry_their_state() {
    let blocks =
        parse_blocks("- one\n- two\n  - nested\n\n3. three\n4. four\n\n- [x] done\n- [ ] todo\n");
    let Block::List { ordered, items, .. } = &blocks[0] else {
        panic!("{blocks:?}")
    };
    assert!(!ordered && items.len() == 2);
    assert!(matches!(
        &items[1].blocks[..],
        [Block::Paragraph(_), Block::List { .. }]
    ));
    let Block::List {
        ordered,
        start,
        items,
    } = &blocks[1]
    else {
        panic!()
    };
    assert_eq!((*ordered, *start, items.len()), (true, 3, 2));
    let Block::List { items, .. } = &blocks[2] else {
        panic!()
    };
    assert_eq!(
        items
            .iter()
            .map(|i: &ListItem| i.checked)
            .collect::<Vec<_>>(),
        [Some(true), Some(false)]
    );
}

#[test]
fn quotes_code_blocks_and_hard_breaks() {
    let blocks =
        parse_blocks("> quoted\n> text\n\n```rust\nlet x = 1;\n```\n\nline one  \nline two\n");
    let Block::Quote(inner) = &blocks[0] else {
        panic!()
    };
    assert_eq!(plain_text(paragraph(inner, 0)), "quoted text");
    assert_eq!(
        blocks[1],
        Block::Code {
            lang: "rust".into(),
            text: "let x = 1;\n".into()
        }
    );
    assert_eq!(plain_text(paragraph(&blocks, 2)), "line one\nline two");
}

#[test]
fn split_wikilinks_handles_escapes_and_junk() {
    assert_eq!(
        split_wikilinks("a [[B]] c [[D|e]]"),
        [
            Piece::Text("a "),
            Piece::Link {
                target: "B",
                shown: "B"
            },
            Piece::Text(" c "),
            Piece::Link {
                target: "D",
                shown: "e"
            }
        ]
    );
    assert_eq!(
        split_wikilinks("\\[[nope]] [[ ]] [[unclosed"),
        [Piece::Text("\\[[nope]] [[ ]] [[unclosed")]
    );
    assert!(split_wikilinks("").is_empty());
}

proptest! {
    #[test]
    fn parsing_never_panics_and_keeps_the_visible_text(text in "[a-zA-Z .,\n#*_`>\\[\\]-]{0,120}") {
        let blocks = parse_blocks(&text);
        fn collect(blocks: &[Block], out: &mut String) {
            for b in blocks {
                match b {
                    Block::Heading { spans, .. } | Block::Paragraph(spans) => out.push_str(&plain_text(spans)),
                    Block::Code { lang, text } => {
                        out.push_str(lang);
                        out.push_str(text);
                    }
                    Block::Quote(inner) => collect(inner, out),
                    Block::List { items, .. } => items.iter().for_each(|i| collect(&i.blocks, out)),
                    Block::Rule | Block::Toc => {}
                    Block::Table { header, rows, .. } => {
                        for cell in header.iter().chain(rows.iter().flatten()) {
                            out.push_str(&plain_text(cell));
                        }
                    }
                }
            }
        }
        let mut seen = String::new();
        collect(&blocks, &mut seen);
        let letters = |s: &str| s.chars().filter(|c| c.is_alphanumeric()).collect::<Vec<_>>();
        // (No parentheses in the alphabet: a link destination such as `[](a)` is not visible text.)
        // Everything alphanumeric the author wrote is still on screen, in order.
        prop_assert_eq!(letters(&seen), letters(&text));
    }

    #[test]
    fn text_without_double_brackets_is_never_split(text in "[a-z \\[\\]|#]{0,40}") {
        prop_assume!(!text.contains("[["));
        let pieces = split_wikilinks(&text);
        let expected: Vec<Piece> = if text.is_empty() { vec![] } else { vec![Piece::Text(&text)] };
        prop_assert_eq!(pieces, expected);
    }

    #[test]
    fn every_link_piece_is_well_formed(text in "[a-z \\[\\]|#\\\\]{0,60}") {
        let links = split_wikilinks(&text).into_iter().filter_map(|p| match p {
            Piece::Link { target, shown } => Some((target, shown)),
            Piece::Text(_) => None,
        }).collect::<Vec<_>>();
        prop_assert!(links.len() <= text.matches("[[").count());
        for (target, shown) in links {
            prop_assert!(!target.is_empty() && target == target.trim());
            prop_assert!(!shown.is_empty() && !target.contains(['[', ']']) && !shown.contains(['[', ']']));
        }
    }
}

#[test]
fn style_default_is_plain() {
    assert_eq!(
        Style::default(),
        Style {
            bold: false,
            italic: false,
            strike: false,
            code: false,
            underline: false,
            color: None
        }
    );
}
