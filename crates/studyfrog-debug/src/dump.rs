//! Value dumps, cut to a fixed length so that one large value cannot flood the log.

use std::fmt::Debug;

use studyfrog_log::Level;

/// The longest dump text, in characters, including the ellipsis when the text is cut.
const MAX_CHARS: usize = 2000;

/// Logs `value` at Debug level for `target`. The message is `label`, and the field `value` holds
/// the `{:?}` text, cut to 2000 characters.
pub fn dump<T: Debug>(target: &str, label: &str, value: &T) {
    if Level::Debug <= studyfrog_log::level() {
        let text = truncate_chars(&format!("{value:?}"), MAX_CHARS);
        studyfrog_log::emit(Level::Debug, target, label, &[("value", text.as_str())]);
    }
}

/// Returns `text` unchanged if it has at most `max` characters. Otherwise returns its first
/// `max - 1` characters followed by an ellipsis, so the result has exactly `max` characters.
pub fn truncate_chars(text: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    if text.chars().count() <= max {
        return text.to_string();
    }
    match text.char_indices().nth(max - 1) {
        Some((end, _)) => format!("{}\u{2026}", &text[..end]),
        None => text.to_string(),
    }
}
