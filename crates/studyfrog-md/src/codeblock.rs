//! Fenced code block tracking, shared by every line-oriented body scanner.

/// Open fence: marker byte (`` ` `` or `~`) and its run length. `None` outside code.
pub type FenceState = Option<(u8, usize)>;

/// Marker byte and run length when `line` is a fence line (up to 3 spaces of indent, then 3+ of
/// the same marker). Returns the rest of the line after the run.
fn fence_line(line: &str) -> Option<(u8, usize, &str)> {
    let trimmed = line.trim_end_matches(['\n', '\r']);
    let indent = trimmed.len() - trimmed.trim_start_matches(' ').len();
    let rest = &trimmed[indent..];
    let marker = *rest
        .as_bytes()
        .first()
        .filter(|b| matches!(**b, b'`' | b'~'))?;
    let run = rest.bytes().take_while(|b| *b == marker).count();
    (indent <= 3 && run >= 3).then(|| (marker, run, &rest[run..]))
}

/// Advance over one line. Returns the new state and whether the line is code (inside a block or
/// one of its fence lines) and so must be skipped by scanners.
pub fn advance_fence(state: FenceState, line: &str) -> (FenceState, bool) {
    match (state, fence_line(line)) {
        (None, Some((m, n, _))) => (Some((m, n)), true),
        (Some((m, n)), Some((lm, ln, rest))) if lm == m && ln >= n && rest.trim().is_empty() => {
            (None, true)
        }
        (Some(open), _) => (Some(open), true),
        (None, None) => (None, false),
    }
}
