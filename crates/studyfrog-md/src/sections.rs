//! Card and question bodies: front `???` back.

use crate::codeblock::advance_fence;

const SEPARATOR: &str = "???";

/// Split at the first line that is exactly `???` outside a code fence. The newline before the
/// separator is not part of the front; everything after the separator line is the back.
pub fn split_card_body(body: &str) -> Option<(&str, &str)> {
    let mut state = None;
    let mut offset = 0;
    for line in body.split_inclusive('\n') {
        let (next, in_code) = advance_fence(state, line);
        state = next;
        if !in_code && line.trim_end_matches(['\n', '\r']) == SEPARATOR {
            let front = &body[..offset];
            return Some((
                front.strip_suffix('\n').unwrap_or(front),
                &body[offset + line.len()..],
            ));
        }
        offset += line.len();
    }
    None
}

/// Inverse of [`split_card_body`] for text that splits back unchanged.
pub fn join_card_body(front: &str, back: &str) -> String {
    if front.is_empty() {
        format!("{SEPARATOR}\n{back}")
    } else {
        format!("{front}\n{SEPARATOR}\n{back}")
    }
}
