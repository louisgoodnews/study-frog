#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Hover tooltips (STORY-0169): every icon button explains itself, and shortcuts are shown where
//! the app has them.

mod common;

use studyfrog_core::event::Difficulty;
use studyfrog_ui::view::tips;

/// The names the icon buttons of the rail, the header bars and the dialogs are given.
const ICON_NAMES: [&str; 13] = [
    "Show or hide the sidebar",
    "Switch vault",
    "Help for this screen (F1)",
    "Profile and vault",
    "Application settings",
    "Notifications",
    "Back (Alt+Left)",
    "Forward (Alt+Right)",
    "History",
    "Focus timer",
    "Zen mode (Ctrl+Shift+F)",
    "Close",
    "Dismiss notification",
];

#[test]
fn every_icon_button_has_a_one_line_explanation_under_its_name() {
    for name in ICON_NAMES {
        let tip = tips::for_name(name);
        assert!(tip.starts_with(name), "{tip}");
        let explain = tip.split_once('\n').map(|(_, e)| e).unwrap_or("");
        assert!(
            !explain.is_empty() && !explain.contains('\n'),
            "{name}: {tip}"
        );
    }
}

#[test]
fn a_name_without_an_explanation_is_shown_as_it_is() {
    assert_eq!(tips::for_name("Something new"), "Something new");
}

#[test]
fn ratings_and_marks_show_their_keys() {
    for (i, key) in ["1", "2", "3", "4"].into_iter().enumerate() {
        assert!(
            tips::rating(i).ends_with(&format!("Shortcut: {key}")),
            "{}",
            tips::rating(i)
        );
    }
    assert!(tips::mark(Difficulty::Easy).ends_with("Shortcut: E"));
    assert!(tips::mark(Difficulty::Medium).ends_with("Shortcut: M"));
    assert!(tips::mark(Difficulty::Difficult).ends_with("Shortcut: D"));
    assert!(tips::show_answer().ends_with("Shortcut: Space"));
    assert!(tips::skip().ends_with("Shortcut: S"));
}

#[test]
fn a_key_is_its_own_line_after_the_explanation() {
    assert_eq!(
        tips::with_key("Shows the answer.", "Space"),
        "Shows the answer.\nShortcut: Space"
    );
}

#[test]
fn hovering_a_text_button_shows_its_explanation() {
    use common::views::{harness, open_model};
    use egui_kittest::kittest::Queryable;
    use studyfrog_ui::model::apply;
    use studyfrog_ui::msg::Msg;
    use studyfrog_ui::state::Screen;
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Library));
    let mut h = harness(m);
    h.run_steps(3);
    h.get_by_label("Columns").hover();
    h.run_steps(5);
    assert!(
        h.query_by_label("Shows or hides the columns of the list.")
            .is_some(),
        "the explanation shows while the pointer rests on the button"
    );
}

#[test]
fn the_theme_sets_the_tooltip_delay() {
    use common::views::{harness, open_model};
    let h = harness(open_model());
    let delay = h.ctx.global_style().interaction.tooltip_delay;
    assert!((delay - tips::DELAY_SECS).abs() < f32::EPSILON, "{delay}");
}

#[test]
fn a_click_hides_the_tooltip_so_it_never_stays_over_the_control() {
    use common::views::{harness, open_model};
    use egui_kittest::kittest::Queryable;
    use studyfrog_ui::model::apply;
    use studyfrog_ui::msg::Msg;
    use studyfrog_ui::state::Screen;
    const EXPLAIN: &str = "Shows or hides the columns of the list.";
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Library));
    let mut h = harness(m);
    h.run_steps(3);
    h.get_by_label("Columns").hover();
    h.run_steps(5);
    assert!(h.query_by_label(EXPLAIN).is_some(), "shown while resting");
    // The click also opens the columns menu, which hides tooltips too; the check is what the user sees.
    h.get_by_label("Columns").click();
    h.run_steps(5);
    assert!(h.query_by_label(EXPLAIN).is_none(), "gone after the click");
}
