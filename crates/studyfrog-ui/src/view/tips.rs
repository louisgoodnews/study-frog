//! Hover tooltips (STORY-0169): one line on what a control does, and its keyboard shortcut on its own
//! line. Pure text; the widgets attach it with `on_hover_text`.

use studyfrog_core::event::Difficulty;

/// How long the pointer rests on a control before its tooltip shows, in seconds. egui hides a tooltip as soon as
/// the pointer moves on, or the control is clicked, so a tooltip never stays over the control being worked with.
pub const DELAY_SECS: f32 = 0.5;

/// What each icon button does, by the name the button is given. A name that already states its
/// shortcut, such as "Back (Alt+Left)", keeps it in the tooltip.
const EXPLAIN: &[(&str, &str)] = &[
    (
        "Show or hide the sidebar",
        "Opens or closes the list of screens on the left.",
    ),
    (
        "Switch vault",
        "Opens the list of vaults, to switch to another one.",
    ),
    (
        "Help for this screen (F1)",
        "Shows the help page for the screen you are on.",
    ),
    (
        "Profile and vault",
        "Your profile, and the vault that is open.",
    ),
    (
        "Application settings",
        "Settings for the app and for the open vault.",
    ),
    ("Notifications", "Reminders and messages from the app."),
    (
        "Back (Alt+Left)",
        "Goes back to the screen you were on before.",
    ),
    (
        "Forward (Alt+Right)",
        "Goes forward again after going back.",
    ),
    ("Focus timer", "Shows the timer for focused study."),
    (
        "History",
        "The screens you have visited, to go back to one of them.",
    ),
    (
        "Zen mode (Ctrl+Shift+F)",
        "Hides everything except the work in front of you.",
    ),
    ("Close", "Closes this dialog."),
    ("Dismiss notification", "Hides this message."),
];

/// The explanation of the control called `name` (`None` when no tooltip has that name).
pub fn explain_of(name: &str) -> Option<&'static str> {
    EXPLAIN.iter().find(|(n, _)| *n == name).map(|(_, e)| *e)
}

/// The names of the controls that have a tooltip.
pub fn names() -> Vec<&'static str> {
    EXPLAIN.iter().map(|(n, _)| *n).collect()
}

/// The tooltip of an icon button: its name, and under it what the button does when it is known.
pub fn for_name(name: &str) -> String {
    match EXPLAIN.iter().find(|(n, _)| *n == name) {
        Some((_, explain)) => format!("{name}\n{explain}"),
        None => name.to_owned(),
    }
}

/// An explanation with its keyboard shortcut on its own line.
pub fn with_key(explain: &str, key: &str) -> String {
    format!("{explain}\nShortcut: {key}")
}

/// The four ratings, in button order: what each one means for when the item comes back.
const RATING_EXPLAIN: [&str; 4] = [
    "Forgot it. It comes back soon.",
    "Recalled with effort. It comes back a little later.",
    "Recalled well. It comes back after the usual interval.",
    "Recalled at once. It comes back much later.",
];
const RATING_KEY: [&str; 4] = ["1", "2", "3", "4"];

/// The tooltip of rating button `index` (0 to 3).
pub fn rating(index: usize) -> String {
    with_key(RATING_EXPLAIN[index], RATING_KEY[index])
}

/// The tooltip of a difficulty mark.
pub fn mark(difficulty: Difficulty) -> String {
    let (explain, key) = match difficulty {
        Difficulty::Easy => ("The item felt easy to recall.", "E"),
        Difficulty::Medium => ("The item felt of medium difficulty.", "M"),
        Difficulty::Difficult => ("The item felt difficult.", "D"),
    };
    with_key(explain, key)
}

/// The tooltip of the button that shows the answer.
pub fn show_answer() -> String {
    with_key("Shows the answer.", "Space")
}

/// The tooltip of the button that skips the item.
pub fn skip() -> String {
    with_key("Moves on without rating this item.", "S")
}
