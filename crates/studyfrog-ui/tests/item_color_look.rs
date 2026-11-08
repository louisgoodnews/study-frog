#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! STORY-0097, STORY-0123: an item's colour tints a surface visibly, yet every text colour of every
//! theme keeps its contrast; the editor's form reads and writes the colour.

use studyfrog_core::appearance::Accent;
use studyfrog_core::item_color::PALETTE;
use studyfrog_md::extra::extra_raw;
use studyfrog_md::parse::parse_doc;
use studyfrog_ui::edit_form::{apply_form, form_from_doc};
use studyfrog_ui::item_look::{MIN_CONTRAST, card_colors, contrast, page_fill};
use studyfrog_ui::theme::{Look, palette_with};

fn looks() -> Vec<(bool, Look)> {
    let accents = [
        Accent::Blue,
        Accent::Green,
        Accent::Purple,
        Accent::Orange,
        Accent::Pink,
        Accent::Teal,
    ];
    let mut out = Vec::new();
    for dark in [true, false] {
        for accent in accents {
            for high_contrast in [false, true] {
                out.push((
                    dark,
                    Look {
                        accent,
                        high_contrast,
                    },
                ));
            }
        }
    }
    out
}

/// Palette words and a grid of codes from black to white through every hue.
fn colours() -> Vec<String> {
    let mut out: Vec<String> = PALETTE.iter().map(|(w, _, _)| (*w).to_owned()).collect();
    for r in (0..=255).step_by(51) {
        for g in (0..=255).step_by(51) {
            for b in (0..=255).step_by(51) {
                out.push(format!("#{r:02X}{g:02X}{b:02X}"));
            }
        }
    }
    out
}

#[test]
fn a_tint_never_makes_text_harder_to_read_than_the_contrast_floor() {
    for (dark, look) in looks() {
        let p = palette_with(dark, look);
        for c in colours() {
            let (fill, _) = card_colors(&p, dark, Some(&c));
            let page = page_fill(&p, dark, Some(&c)).unwrap();
            for (surface, tinted) in [(p.surface, fill), (p.background, page)] {
                for text in [p.text, p.text_weak] {
                    let floor = MIN_CONTRAST.min(contrast(surface, text)) - 0.01;
                    let got = contrast(tinted, text);
                    assert!(got >= floor, "{c} dark={dark} {look:?}: {got} < {floor}");
                }
            }
        }
    }
}

#[test]
fn a_palette_colour_visibly_tints_the_card_in_both_themes() {
    for dark in [true, false] {
        let p = palette_with(dark, Look::default());
        for (word, _, _) in PALETTE {
            let (fill, border) = card_colors(&p, dark, Some(word));
            assert_ne!(fill, p.surface, "{word} dark={dark}");
            assert_ne!(border, p.border, "{word} dark={dark}");
            assert!(contrast(fill, p.text) >= MIN_CONTRAST, "{word} dark={dark}");
        }
        assert_eq!(card_colors(&p, dark, None), (p.surface, p.border));
        assert_eq!(
            card_colors(&p, dark, Some("chartreuse")),
            (p.surface, p.border)
        );
    }
}

#[test]
fn the_form_reads_and_writes_the_colour_and_keeps_a_hand_written_word() {
    let text =
        "---\nid: 01JBYQ00W9Y7A19B1X00000002\nkind: card\ntitle: C\ncolor: green\n---\nq\n???\na\n";
    let doc = parse_doc(text).unwrap();
    let mut form = form_from_doc(&doc);
    assert_eq!(form.color.as_deref(), Some("sage"));
    let same = apply_form(&doc, &form).unwrap();
    assert_eq!(
        extra_raw(&same, "color").as_deref(),
        Some("green"),
        "unchanged stays as written"
    );
    form.color = Some("#4A90E2".into());
    let coded = apply_form(&doc, &form).unwrap();
    assert_eq!(
        extra_raw(&coded, "color").as_deref(),
        Some("\"#4A90E2\""),
        "a code is quoted for YAML"
    );
    let back = parse_doc(&studyfrog_md::render::render_doc(&coded).unwrap()).unwrap();
    assert_eq!(form_from_doc(&back).color.as_deref(), Some("#4A90E2"));
    form.color = None;
    assert_eq!(extra_raw(&apply_form(&doc, &form).unwrap(), "color"), None);
}
