#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Tables, underline, colour and the table of contents in the preview and in the cards of a run
//! (TASK-0197, 0215, 0216).

mod common;

use common::views::{assert_all_named, harness, running_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::run::RunMode;
use studyfrog_core::uid::Uid;
use studyfrog_md::hue::HUES;
use studyfrog_service::content::ItemContent;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::theme::{Look, palette_with};
use studyfrog_ui::theme_hue::hue_color;

fn luminance(c: eframe::egui::Color32) -> f64 {
    let f = |v: u8| {
        let v = f64::from(v) / 255.0;
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * f(c.r()) + 0.7152 * f(c.g()) + 0.0722 * f(c.b())
}

fn contrast(a: eframe::egui::Color32, b: eframe::egui::Color32) -> f64 {
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

#[test]
fn every_text_colour_is_readable_on_the_backgrounds_of_both_themes() {
    for dark in [true, false] {
        let p = palette_with(dark, Look::default());
        for hue in HUES {
            let c = hue_color(hue, dark, false, &p);
            for bg in [p.background, p.surface, p.panel] {
                assert!(
                    contrast(c, bg) >= 4.5,
                    "{hue:?} dark {dark}: {}",
                    contrast(c, bg)
                );
            }
        }
        let c = hue_color(HUES[0], dark, true, &p);
        assert_eq!(c, p.text, "high contrast keeps the text colour");
    }
}

fn card(prompt: &str, answer: &str) -> egui_kittest::Harness<'static, common::views::App> {
    let mut m = running_model(RunMode::Flip);
    let content = ItemContent {
        title: "t".into(),
        prompt: prompt.into(),
        answer: answer.into(),
        previous: None,
        source: None,
        color: None,
    };
    apply(&mut m, Msg::ItemLoaded(Uid(1), Ok(content)));
    let mut h = harness(m);
    h.run_steps(2);
    h
}

#[test]
fn a_table_in_a_card_is_drawn_as_cells_not_as_pipes() {
    let table = "| Word | Meaning |\n|:--|--:|\n| cat | chat |\n| **dog** | chien |\n";
    let mut h = card(table, "x");
    for cell in ["Word", "Meaning", "cat", "chat", "dog", "chien"] {
        assert!(h.query_by_label(cell).is_some(), "{cell}");
    }
    assert!(
        h.query_by_label_contains("|").is_none(),
        "no pipes on screen"
    );
    assert!(h.query_by_label_contains("---").is_none());
    assert_all_named(&h, "card with a table");
    h.run_steps(1);
}

#[test]
fn underlined_and_coloured_text_shows_without_its_tags_and_unknown_tags_stay_visible() {
    let h = card(
        "a <u>under</u> and <span style=\"color:#c0392b\">red</span> but <b>raw</b>",
        "x",
    );
    assert!(h.query_by_label("under").is_some() && h.query_by_label("red").is_some());
    assert!(h.query_by_label_contains("<u>").is_none());
    assert!(
        h.query_by_label_contains("<b>raw</b>").is_some(),
        "only the two forms are read"
    );
}

#[test]
fn the_table_of_contents_lists_the_headings_with_links_and_says_when_there_are_none() {
    let h = card("[TOC]\n\n# One\n\n## Two\n\ntext", "x");
    assert!(h.query_by_label("Contents").is_some());
    assert_eq!(
        h.query_all_by_label("One").count(),
        2,
        "the entry and the heading"
    );
    assert_eq!(h.query_all_by_label("Two").count(), 2);
    let h = card("[TOC]\n\nno headings here", "x");
    assert!(h.query_by_label("This note has no headings yet.").is_some());
}

#[test]
fn a_code_block_shows_its_whole_text_with_or_without_a_known_language() {
    let known = card("```python\ndef f(x):  # note\n    return 'a' + 1\n```", "x");
    assert!(known.query_by_label_contains("def f(x):").is_some());
    assert!(
        known.query_by_label_contains("# note").is_some(),
        "the comment is still text"
    );
    let unknown = card("```brainfuck\n+++[->+<]\n```", "x");
    assert!(unknown.query_by_label_contains("+++[->+<]").is_some());
    let none = card("```\nplain code\n```", "x");
    assert!(none.query_by_label_contains("plain code").is_some());
}
