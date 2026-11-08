#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Markdown toolbar commands as pure text edits.

use studyfrog_ui::md_edit::{Cmd, apply_cmd};

#[test]
fn bold_wraps_the_selection_or_inserts_a_placeholder_that_stays_selected() {
    assert_eq!(
        apply_cmd("a word b", (2, 6), Cmd::Bold),
        ("a **word** b".to_owned(), (4, 8))
    );
    assert_eq!(
        apply_cmd("ab", (1, 1), Cmd::Bold),
        ("a**bold**b".to_owned(), (3, 7))
    );
    assert_eq!(apply_cmd("x", (0, 1), Cmd::Italic).0, "*x*");
    assert_eq!(apply_cmd("x", (0, 1), Cmd::Code).0, "`x`");
    assert_eq!(apply_cmd("Beta", (0, 4), Cmd::WikiLink).0, "[[Beta]]");
    assert_eq!(
        apply_cmd("日本", (0, 2), Cmd::Bold),
        ("**日本**".to_owned(), (2, 4)),
        "characters, not bytes"
    );
}

#[test]
fn a_link_leaves_the_address_selected_for_typing() {
    let (text, (a, b)) = apply_cmd("site", (0, 4), Cmd::Link);
    assert_eq!(text, "[site](https://)");
    assert_eq!(
        text.chars().skip(a).take(b - a).collect::<String>(),
        "https://"
    );
}

#[test]
fn line_commands_toggle_on_every_touched_line() {
    assert_eq!(
        apply_cmd("one\ntwo\nthree", (0, 7), Cmd::Bullet).0,
        "- one\n- two\nthree"
    );
    assert_eq!(
        apply_cmd("- one\n- two\nthree", (0, 11), Cmd::Bullet).0,
        "one\ntwo\nthree",
        "all marked: unmark"
    );
    assert_eq!(
        apply_cmd("- one\ntwo", (0, 9), Cmd::Bullet).0,
        "- one\n- two",
        "mixed: mark the rest"
    );
    assert_eq!(apply_cmd("a\nb", (0, 3), Cmd::Numbered).0, "1. a\n2. b");
    assert_eq!(apply_cmd("1. a\n2. b", (0, 9), Cmd::Numbered).0, "a\nb");
    assert_eq!(apply_cmd("x", (1, 1), Cmd::Heading).0, "## x");
    assert_eq!(apply_cmd("## x", (0, 0), Cmd::Heading).0, "x");
    assert_eq!(apply_cmd("q", (0, 0), Cmd::Quote).0, "> q");
}

#[test]
fn a_selection_that_ends_at_a_line_start_does_not_touch_that_line() {
    assert_eq!(apply_cmd("one\ntwo", (0, 4), Cmd::Bullet).0, "- one\ntwo");
}

#[test]
fn the_selection_moves_with_the_inserted_markers() {
    let (text, (a, b)) = apply_cmd("one\ntwo", (4, 7), Cmd::Bullet);
    assert_eq!(text, "one\n- two");
    assert_eq!(text.chars().skip(a).take(b - a).collect::<String>(), "two");
}

#[test]
fn a_rule_goes_on_its_own_line() {
    assert_eq!(apply_cmd("abc", (3, 3), Cmd::Rule).0, "abc\n---\n");
    assert_eq!(apply_cmd("", (0, 0), Cmd::Rule).0, "---\n");
    assert_eq!(apply_cmd("a\n", (2, 2), Cmd::Rule).0, "a\n---\n");
}

#[test]
fn out_of_range_selections_are_clamped_not_a_panic() {
    assert_eq!(apply_cmd("ab", (9, 12), Cmd::Bold).0, "ab**bold**");
    assert_eq!(apply_cmd("", (3, 1), Cmd::Bullet).0, "- ");
}

#[test]
fn strike_underline_and_colour_wrap_the_selection_with_the_markers_the_file_keeps() {
    use studyfrog_md::hue::Hue;
    let (t, sel) = apply_cmd("a word b", (2, 6), Cmd::Strike);
    assert_eq!((t.as_str(), sel), ("a ~~word~~ b", (4, 8)));
    let (t, _) = apply_cmd("a word b", (2, 6), Cmd::Underline);
    assert_eq!(t, "a <u>word</u> b");
    let (t, _) = apply_cmd("a word b", (2, 6), Cmd::Color(Hue::Red));
    assert_eq!(t, "a <span style=\"color:#c0392b\">word</span> b");
    let (t, sel) = apply_cmd("", (0, 0), Cmd::Underline);
    assert_eq!((t.as_str(), sel), ("<u>underlined</u>", (3, 13)));
}

#[test]
fn the_table_goes_on_lines_of_its_own_with_the_first_header_cell_selected() {
    let (t, (a, b)) = apply_cmd("", (0, 0), Cmd::Table);
    assert_eq!(t, "| Column 1 | Column 2 |\n|---|---|\n| | |\n");
    assert_eq!(
        t.chars().skip(a).take(b - a).collect::<String>(),
        "Column 1"
    );
    let (t, _) = apply_cmd("text", (4, 4), Cmd::Table);
    assert!(t.starts_with("text\n\n| Column 1"), "{t:?}");
    let (t, _) = apply_cmd("ab", (1, 1), Cmd::Table);
    assert!(
        t.starts_with("a\n\n| ") && t.ends_with("| | |\n\nb") || t.ends_with("| | |\nb"),
        "{t:?}"
    );
    let blocks = studyfrog_md::blocks::parse_blocks(&apply_cmd("", (0, 0), Cmd::Table).0);
    assert!(matches!(
        blocks[0],
        studyfrog_md::blocks::Block::Table { .. }
    ));
}

#[test]
fn cloze_wraps_the_selection_with_the_next_or_the_same_number() {
    assert_eq!(
        apply_cmd("The sun", (4, 7), Cmd::Cloze(false)),
        ("The {{c1::sun}}".to_owned(), (10, 13))
    );
    let two = "The {{c1::sun}} is a star";
    assert_eq!(
        apply_cmd(two, (21, 25), Cmd::Cloze(false)).0,
        "The {{c1::sun}} is a {{c2::star}}"
    );
    assert_eq!(
        apply_cmd(two, (21, 25), Cmd::Cloze(true)).0,
        "The {{c1::sun}} is a {{c1::star}}"
    );
    assert_eq!(
        apply_cmd("x ", (2, 2), Cmd::Cloze(false)).0,
        "x {{c1::answer}}"
    );
}
