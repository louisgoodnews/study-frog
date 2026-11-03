//! The Table button: a small pipe table, put on lines of its own at the cursor. Pure.

use crate::md_edit::{Edited, clamp};

const TABLE: &str = "| Column 1 | Column 2 |\n|---|---|\n| | |\n";

/// Insert the table; the first header cell is selected so typing replaces it.
pub(crate) fn insert(chars: &[char], sel: (usize, usize)) -> Edited {
    let (a, b) = clamp(chars, sel);
    let head: String = chars[..a].iter().collect();
    let tail: String = chars[b..].iter().collect();
    let lead = if head.is_empty() || head.ends_with('\n') {
        ""
    } else {
        "\n\n"
    };
    let gap = if tail.is_empty() || tail.starts_with('\n') {
        ""
    } else {
        "\n"
    };
    let start = a + lead.chars().count() + 2;
    (
        format!("{head}{lead}{TABLE}{gap}{tail}"),
        (start, start + "Column 1".chars().count()),
    )
}
