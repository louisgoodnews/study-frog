#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Hover tooltips (STORY-0169): every icon button explains itself, and shortcuts are shown where
//! the app has them.

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
