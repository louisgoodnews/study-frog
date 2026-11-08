//! Markdown editing commands: what the toolbar buttons do to the text and the selection. Positions
//! are character indices, as in egui's text cursor. Pure.

use studyfrog_md::hue::{Hue, span_open};

/// A formatting command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cmd {
    /// `**bold**`
    Bold,
    /// `*italic*`
    Italic,
    /// `` `code` ``
    Code,
    /// `[[wikilink]]`
    WikiLink,
    /// `[text](url)`
    Link,
    /// `![alt](path)`
    Image,
    /// `## ` at the start of the lines.
    Heading,
    /// `- ` at the start of the lines.
    Bullet,
    /// `1. ` at the start of the lines.
    Numbered,
    /// `> ` at the start of the lines.
    Quote,
    /// A horizontal rule on its own line.
    Rule,
    /// `~~strikethrough~~`
    Strike,
    /// `<u>underline</u>`
    Underline,
    /// A coloured span.
    Color(Hue),
    /// A small pipe table on its own lines.
    Table,
    /// A cloze gap `{{cN::...}}`: the next number, or the last one used when `true`.
    Cloze(bool),
}

/// The text and the selection (start, end) after a command.
pub type Edited = (String, (usize, usize));

pub(crate) fn clamp(chars: &[char], (a, b): (usize, usize)) -> (usize, usize) {
    (a.min(b).min(chars.len()), a.max(b).min(chars.len()))
}

fn wrap(
    chars: &[char],
    sel: (usize, usize),
    (open, close): (&str, &str),
    placeholder: &str,
) -> Edited {
    let (a, b) = clamp(chars, sel);
    let inner: String = if a == b {
        placeholder.to_owned()
    } else {
        chars[a..b].iter().collect()
    };
    let head: String = chars[..a].iter().collect();
    let tail: String = chars[b..].iter().collect();
    let start = a + open.chars().count();
    let end = start + inner.chars().count();
    (format!("{head}{open}{inner}{close}{tail}"), (start, end))
}

/// Where the lines touched by the selection start.
fn line_starts(chars: &[char], (a, b): (usize, usize)) -> Vec<usize> {
    let first = chars[..a]
        .iter()
        .rposition(|c| *c == '\n')
        .map_or(0, |i| i + 1);
    let mut starts = vec![first];
    for (i, c) in chars.iter().enumerate().take(b).skip(first) {
        if *c == '\n' && i + 1 < b {
            starts.push(i + 1);
        }
    }
    starts
}

/// The length of the list or quote marker the line at `start` already has.
fn marker_len(chars: &[char], start: usize, prefix: &str, numbered: bool) -> Option<usize> {
    let line: String = chars[start..].iter().take_while(|c| **c != '\n').collect();
    if numbered {
        let digits = line.chars().take_while(char::is_ascii_digit).count();
        (digits > 0 && line[digits..].starts_with(". ")).then_some(digits + 2)
    } else {
        line.starts_with(prefix).then(|| prefix.chars().count())
    }
}

fn prefix_lines(chars: &[char], sel: (usize, usize), prefix: &str, numbered: bool) -> Edited {
    let (a, b) = clamp(chars, sel);
    let starts = line_starts(chars, (a, b));
    let existing: Vec<Option<usize>> = starts
        .iter()
        .map(|s| marker_len(chars, *s, prefix, numbered))
        .collect();
    let remove = existing.iter().all(Option::is_some);
    let mut out = String::new();
    let mut cursor = 0;
    let mut deltas: Vec<(usize, isize)> = Vec::new();
    for (n, s) in starts.iter().enumerate() {
        out.extend(&chars[cursor..*s]);
        cursor = *s;
        match (remove, existing[n]) {
            (true, Some(len)) => {
                cursor += len;
                deltas.push((*s, -(len as isize)));
            }
            (false, None) => {
                let marker = if numbered {
                    format!("{}. ", n + 1)
                } else {
                    prefix.to_owned()
                };
                deltas.push((*s, marker.chars().count() as isize));
                out.push_str(&marker);
            }
            _ => {}
        }
    }
    out.extend(&chars[cursor..]);
    let moved = |p: usize| {
        let shift: isize = deltas.iter().filter(|(s, _)| *s <= p).map(|(_, d)| d).sum();
        usize::try_from(p as isize + shift).unwrap_or(0)
    };
    (out, (moved(a), moved(b)))
}

/// Apply `cmd` to `text` with the selection `(start, end)` (equal when nothing is selected).
pub fn apply_cmd(text: &str, sel: (usize, usize), cmd: Cmd) -> Edited {
    let chars: Vec<char> = text.chars().collect();
    match cmd {
        Cmd::Bold => wrap(&chars, sel, ("**", "**"), "bold"),
        Cmd::Italic => wrap(&chars, sel, ("*", "*"), "italic"),
        Cmd::Code => wrap(&chars, sel, ("`", "`"), "code"),
        Cmd::WikiLink => wrap(&chars, sel, ("[[", "]]"), "note title"),
        Cmd::Link => {
            let (text, (_, end)) = wrap(&chars, sel, ("[", "](https://)"), "text");
            (text, (end + 2, end + 10))
        }
        Cmd::Image => wrap(&chars, sel, ("![", "](image.png)"), "description"),
        Cmd::Heading => prefix_lines(&chars, sel, "## ", false),
        Cmd::Bullet => prefix_lines(&chars, sel, "- ", false),
        Cmd::Numbered => prefix_lines(&chars, sel, "1. ", true),
        Cmd::Quote => prefix_lines(&chars, sel, "> ", false),
        Cmd::Strike => wrap(&chars, sel, ("~~", "~~"), "struck out"),
        Cmd::Underline => wrap(&chars, sel, ("<u>", "</u>"), "underlined"),
        Cmd::Color(hue) => wrap(&chars, sel, (&span_open(hue), "</span>"), "coloured"),
        Cmd::Table => crate::md_edit_table::insert(&chars, sel),
        Cmd::Cloze(same) => {
            let n = if same {
                studyfrog_md::cloze::last_cloze_index(text)
            } else {
                studyfrog_md::cloze::next_cloze_index(text)
            };
            wrap(&chars, sel, (&format!("{{{{c{n}::"), "}}"), "answer")
        }
        Cmd::Rule => {
            let (a, b) = clamp(&chars, sel);
            let head: String = chars[..a].iter().collect();
            let tail: String = chars[b..].iter().collect();
            let lead = if head.is_empty() || head.ends_with('\n') {
                ""
            } else {
                "\n"
            };
            let end = a + lead.len() + 4;
            (format!("{head}{lead}---\n{tail}"), (end, end))
        }
    }
}
