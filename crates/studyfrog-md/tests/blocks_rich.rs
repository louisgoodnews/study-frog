#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Tables, underline, colour and the table of contents marker in the block parser (TASK-0197, 0215, 0216).

use studyfrog_md::blocks::{Align, Block, parse_blocks, plain_text};
use studyfrog_md::hue::{HUES, Hue, hue_from_hex, hue_from_name, hue_hex, hue_name, span_open};

#[test]
fn a_pipe_table_has_a_header_alignments_and_rows_with_styled_cells() {
    let md = "| Name | Qty | Note |\n|:---|---:|:---:|\n| **apple** | 3 | red |\n| pear | 10 | |\n";
    let blocks = parse_blocks(md);
    let [
        Block::Table {
            aligns,
            header,
            rows,
        },
    ] = blocks.as_slice()
    else {
        panic!("{blocks:?}");
    };
    assert_eq!(aligns, &[Align::Left, Align::Right, Align::Center]);
    let head: Vec<String> = header.iter().map(|c| plain_text(c)).collect();
    assert_eq!(head, ["Name", "Qty", "Note"]);
    assert_eq!(rows.len(), 2);
    assert!(rows[0][0][0].style.bold, "inline formatting works in cells");
    assert_eq!(plain_text(&rows[1][1]), "10");
    assert!(rows[1][2].is_empty() || plain_text(&rows[1][2]).is_empty());
}

#[test]
fn text_around_a_table_is_kept_and_a_table_in_a_quote_or_list_does_not_break_the_walk() {
    let blocks = parse_blocks("before\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\nafter\n");
    assert!(matches!(
        blocks.as_slice(),
        [
            Block::Paragraph(_),
            Block::Table { .. },
            Block::Paragraph(_)
        ]
    ));
    let blocks = parse_blocks("> | a | b |\n> |---|---|\n> | 1 | 2 |\n");
    assert!(matches!(&blocks[0], Block::Quote(inner) if matches!(inner[0], Block::Table { .. })));
}

#[test]
fn underline_and_the_palette_colours_are_read_and_anything_else_stays_text() {
    let blocks = parse_blocks("a <u>under</u> b <span style=\"color:#c0392b\">red</span> c");
    let Block::Paragraph(spans) = &blocks[0] else {
        panic!()
    };
    let under = spans.iter().find(|s| s.text == "under").unwrap();
    assert!(under.style.underline && under.style.color.is_none());
    let red = spans.iter().find(|s| s.text == "red").unwrap();
    assert_eq!(red.style.color, Some(Hue::Red));
    assert!(
        !spans
            .iter()
            .find(|s| s.text.contains(" c"))
            .unwrap()
            .style
            .underline
    );

    let other = parse_blocks("x <span style=\"color:#123456\">no</span> <b>bold</b>");
    let Block::Paragraph(spans) = &other[0] else {
        panic!()
    };
    let all = plain_text(spans);
    assert!(
        all.contains("<span style=\"color:#123456\">") && all.contains("<b>"),
        "{all}"
    );
    assert!(spans.iter().all(|s| s.style.color.is_none()));
}

#[test]
fn formatting_nests_with_bold_and_survives_an_unbalanced_tag() {
    let blocks = parse_blocks("**<u>both</u>** and </u> stray <u>open");
    let Block::Paragraph(spans) = &blocks[0] else {
        panic!()
    };
    let both = spans.iter().find(|s| s.text == "both").unwrap();
    assert!(both.style.bold && both.style.underline);
    assert!(
        plain_text(spans).contains("</u> stray"),
        "a stray closing tag is text"
    );
}

#[test]
fn the_palette_round_trips_through_names_and_values_in_the_file() {
    for h in HUES {
        assert_eq!(hue_from_name(hue_name(h)), Some(h));
        assert_eq!(hue_from_hex(hue_hex(h)), Some(h));
        assert_eq!(hue_from_hex(&hue_hex(h).to_uppercase()), Some(h));
        let tag = span_open(h);
        let blocks = parse_blocks(&format!("{tag}x</span>"));
        let Block::Paragraph(spans) = &blocks[0] else {
            panic!()
        };
        assert_eq!(spans[0].style.color, Some(h));
    }
    assert_eq!(hue_from_name("gray"), Some(Hue::Grey));
    assert_eq!(hue_from_name("pink"), None);
}

#[test]
fn a_line_with_only_toc_is_the_table_of_contents_and_other_text_is_not() {
    for marker in ["[TOC]", "[toc]"] {
        let blocks = parse_blocks(&format!("# A\n\n{marker}\n\n## B\n"));
        assert!(blocks.iter().any(|b| matches!(b, Block::Toc)), "{marker}");
    }
    let blocks = parse_blocks("see [TOC] in the text");
    assert!(!blocks.iter().any(|b| matches!(b, Block::Toc)));
}
