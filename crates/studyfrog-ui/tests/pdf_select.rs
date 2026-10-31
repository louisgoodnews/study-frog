#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Picking words with a dragged rectangle.

use studyfrog_pdf::text::WordBox;
use studyfrog_ui::pdf_select::{matching_words, selected_text, words_in_rect};

fn word(text: &str, x: f32, y: f32) -> WordBox {
    WordBox {
        text: text.into(),
        x,
        y,
        width: 30.0,
        height: 10.0,
    }
}

#[test]
fn a_dragged_rectangle_picks_words_and_makes_text_with_line_breaks() {
    let words = [
        word("The", 10.0, 10.0),
        word("cell", 50.0, 10.0),
        word("divides", 10.0, 30.0),
        word("far", 200.0, 200.0),
    ];
    let rect = studyfrog_pdf::render::PageRect {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 45.0,
    };
    let picked = words_in_rect(&words, &rect);
    assert_eq!(picked, [0, 1, 2]);
    assert_eq!(selected_text(&words, &picked), "The cell\ndivides");
    assert_eq!(selected_text(&words, &[]), "");
    assert_eq!(
        selected_text(&words, &[99]),
        "",
        "stale indexes are ignored"
    );
    assert_eq!(matching_words(&words, "CELL div"), [1, 2]);
    assert!(matching_words(&words, "  ").is_empty());
}
