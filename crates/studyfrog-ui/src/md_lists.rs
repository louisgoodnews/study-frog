//! Lists in the Markdown editor: Enter continues a list with the next marker, Enter on an empty item ends the
//! list, and Backspace on an empty item removes its marker. Positions are character indices. Pure.

use crate::md_edit::Edited;

/// What a list marker is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// `-`, `*` or `+`.
    Bullet(char),
    /// A number and the character after it (`.` or `)`).
    Numbered(u64, char),
}

/// The start of a list item: its indent, its marker, a task box if there is one, and how many characters they take.
struct Marker {
    indent: String,
    kind: Kind,
    task: bool,
    width: usize,
}

fn parse(line: &str) -> Option<Marker> {
    let chars: Vec<char> = line.chars().collect();
    let indent: String = chars
        .iter()
        .take_while(|c| **c == ' ' || **c == '\t')
        .collect();
    let mut at = indent.chars().count();
    let kind = match chars.get(at)? {
        c @ ('-' | '*' | '+') if chars.get(at + 1) == Some(&' ') => {
            at += 2;
            Kind::Bullet(*c)
        }
        d if d.is_ascii_digit() => {
            let digits: String = chars[at..]
                .iter()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            let end = at + digits.len();
            let delim = *chars.get(end).filter(|c| matches!(c, '.' | ')'))?;
            (chars.get(end + 1) == Some(&' ')).then_some(())?;
            at = end + 2;
            Kind::Numbered(digits.parse().ok()?, delim)
        }
        _ => return None,
    };
    let boxed = matches!(
        chars.get(at..at + 4),
        Some(['[', ' ' | 'x' | 'X', ']', ' '])
    );
    Some(Marker {
        indent,
        kind,
        task: boxed,
        width: at + if boxed { 4 } else { 0 },
    })
}

/// The marker of the item after one with `m`: the same bullet, the next number, an unchecked box for a task.
fn next_marker(m: &Marker) -> String {
    let head = match m.kind {
        Kind::Bullet(c) => format!("{c} "),
        Kind::Numbered(n, d) => format!("{}{d} ", n + 1),
    };
    let task = if m.task { "[ ] " } else { "" };
    format!("{}{head}{task}", m.indent)
}

/// The line holding `at`: where it starts and ends, and its text.
fn line_at(chars: &[char], at: usize) -> (usize, usize, String) {
    let at = at.min(chars.len());
    let start = chars[..at]
        .iter()
        .rposition(|c| *c == '\n')
        .map_or(0, |i| i + 1);
    let end = chars[at..]
        .iter()
        .position(|c| *c == '\n')
        .map_or(chars.len(), |i| at + i);
    (start, end, chars[start..end].iter().collect())
}

fn without_line_content(chars: &[char], (start, end): (usize, usize)) -> Edited {
    let text: String = chars[..start].iter().chain(&chars[end..]).collect();
    (text, (start, start))
}

/// Enter with the cursor at `at`: `Some` when the line is a list item and the list is to be continued or ended,
/// `None` when the editor's own line break is right (not a list, or the cursor sits inside the marker).
pub fn on_enter(text: &str, at: usize) -> Option<Edited> {
    let chars: Vec<char> = text.chars().collect();
    let (start, end, line) = line_at(&chars, at);
    let marker = parse(&line)?;
    if at < start + marker.width {
        return None;
    }
    let empty = line.chars().skip(marker.width).all(char::is_whitespace);
    if empty && at == end {
        return Some(without_line_content(&chars, (start, end)));
    }
    let next = next_marker(&marker);
    let head: String = chars[..at].iter().collect();
    let tail: String = chars[at..].iter().collect();
    let cursor = at + 1 + next.chars().count();
    Some((format!("{head}\n{next}{tail}"), (cursor, cursor)))
}

/// Backspace at the end of an item that has no text yet removes its marker.
pub fn on_backspace(text: &str, at: usize) -> Option<Edited> {
    let chars: Vec<char> = text.chars().collect();
    let (start, end, line) = line_at(&chars, at);
    let marker = parse(&line)?;
    let empty = line.chars().skip(marker.width).all(char::is_whitespace);
    (empty && at == end).then(|| without_line_content(&chars, (start, end)))
}
