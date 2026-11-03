#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0198: macros typed in the text turn into Markdown at the closing brace.

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::uid::Uid;
use studyfrog_md::blocks::{Block, parse_blocks};
use studyfrog_ui::md_macros::{MACROS, expand_in, macro_before};
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

fn expanded(text: &str) -> Option<(String, (usize, usize))> {
    let caret = text.chars().count();
    let (range, e) = macro_before(text, caret)?;
    Some(expand_in(text, range, &e))
}

#[test]
fn table_makes_a_default_table_and_a_size_makes_that_many_columns_and_rows() {
    let (t, (a, b)) = expanded("{table}").unwrap();
    assert!(
        t.starts_with("| Column 1 | Column 2 |\n|---|---|\n|"),
        "{t}"
    );
    assert_eq!(
        t.chars().skip(a).take(b - a).collect::<String>(),
        "Column 1",
        "the cursor is in the first cell"
    );
    let (t, _) = expanded("{table:3x4}").unwrap();
    assert_eq!(t.lines().count(), 6, "a header, the rule and four rows");
    assert_eq!(t.lines().next().unwrap().matches("Column").count(), 3);
    for tex in [t.as_str(), expanded("{table}").unwrap().0.as_str()] {
        assert!(matches!(parse_blocks(tex)[0], Block::Table { .. }), "{tex}");
    }
    assert!(
        expanded("{table:0x2}").is_none()
            && expanded("{table:99x99}").is_none()
            && expanded("{table:big}").is_none()
    );
}

#[test]
fn the_other_macros_write_the_markdown_the_preview_reads() {
    assert_eq!(expanded("{toc}").unwrap().0, "[TOC]");
    let (t, (a, b)) = expanded("{code:python}").unwrap();
    assert_eq!(t, "```python\ncode\n```");
    assert_eq!(t.chars().skip(a).take(b - a).collect::<String>(), "code");
    assert_eq!(expanded("{code}").unwrap().0, "```\ncode\n```");
    assert_eq!(expanded("{quote}").unwrap().0, "> quote");
    let (t, _) = expanded("{color:Red}").unwrap();
    assert_eq!(t, "<span style=\"color:#c0392b\">coloured</span>");
    assert!(expanded("{color:pink}").is_none());
    let (range, e) = macro_before("{image}", 7).unwrap();
    assert!(e.picture && range == (0, 7));
}

#[test]
fn unknown_or_unfinished_macros_stay_as_typed() {
    for text in [
        "{nothing}",
        "{toc",
        "toc}",
        "{toc:extra}",
        "a {b {toc}x",
        "{to\nc}",
    ] {
        let caret = text.chars().count();
        if text.ends_with('}') && text.contains("{toc}") {
            continue;
        }
        assert!(macro_before(text, caret).is_none(), "{text:?}");
    }
    assert!(
        macro_before("{toc}", 3).is_none(),
        "the caret must be right after the brace"
    );
}

#[test]
fn a_block_macro_in_the_middle_of_a_line_gets_lines_of_its_own() {
    let text = "see {toc} here";
    let (range, e) = macro_before(&text[..9], 9).unwrap();
    let (out, _) = expand_in(text, range, &e);
    assert_eq!(out, "see \n\n[TOC]\n here");
    let (out, _) = expand_in("a\n{toc}", (2, 7), &e);
    assert_eq!(out, "a\n[TOC]");
}

#[test]
fn the_list_of_macros_is_data_with_a_help_text_each() {
    let names: Vec<_> = MACROS.iter().map(|m| m.name).collect();
    assert_eq!(names, ["table", "toc", "code", "quote", "image", "color"]);
    assert!(
        MACROS
            .iter()
            .all(|m| m.usage.starts_with('{') && !m.help.is_empty())
    );
}

const NOTE: &str = "---\nid: 0000000000000000000000000A\nkind: note\ntitle: Alpha\n---\nstart\n";

fn editing() -> studyfrog_ui::model::Model {
    let mut m = open_model();
    let doc = studyfrog_md::parse::parse_doc(NOTE).unwrap();
    let loaded = studyfrog_service::document::LoadedDoc {
        path: "s/alpha.md".into(),
        doc,
        hash: 1,
    };
    apply(&mut m, Msg::DocumentLoaded(Uid(0xA), Ok(loaded)));
    assert_eq!(m.state.screen, Screen::Editor);
    m
}

fn body(h: &egui_kittest::Harness<'_, common::views::App>) -> String {
    h.state().0.state.editor.as_ref().unwrap().form.fields[0]
        .text
        .clone()
}

#[test]
fn typing_the_closing_brace_in_the_editor_expands_the_macro_but_pasting_does_not() {
    let mut h = harness(editing());
    h.run_steps(2);
    h.get_by_label("Body").focus();
    h.run_steps(2);
    h.get_by_label("Body").type_text("{toc");
    h.run_steps(2);
    assert!(body(&h).ends_with("{toc"), "{:?}", body(&h));
    h.get_by_label("Body").type_text("}");
    h.run_steps(3);
    assert!(
        body(&h).contains("[TOC]") && !body(&h).contains("{toc}"),
        "{:?}",
        body(&h)
    );

    h.get_by_label("Body").type_text("\n{quote}");
    h.run_steps(3);
    assert!(
        body(&h).contains("{quote}"),
        "a pasted macro is left alone: {:?}",
        body(&h)
    );
    assert!(h.query_by_label("Macros for Body").is_some());
    assert_all_named(&h, "editor with macros");
}
