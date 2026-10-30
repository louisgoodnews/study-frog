//! Wikilink extraction: `[[Target]]`, `[[Target|label]]`, `[[Target#Heading]]` and the combination
//! `[[Target#Heading|label]]`. Links in code spans and fenced blocks are ignored. Borrows the body.

use crate::codeblock::advance_fence;

/// One link, borrowing from the scanned text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WikiLink<'a> {
    /// Note title or alias.
    pub target: &'a str,
    /// Text after `#`, if any.
    pub heading: Option<&'a str>,
    /// Text after `|`, if any.
    pub label: Option<&'a str>,
    /// 1-based line in the scanned text.
    pub line: usize,
}

/// All wikilinks in `body`, in order of appearance.
pub fn extract_wikilinks(body: &str) -> Vec<WikiLink<'_>> {
    let mut links = Vec::new();
    let mut state = None;
    for (i, line) in body.lines().enumerate() {
        let (next, in_code) = advance_fence(state, line);
        state = next;
        if !in_code {
            scan_line(line, i + 1, &mut links);
        }
    }
    links
}

/// End of the code span starting at `i` (a backtick), or `None` when it never closes.
fn code_span_end(bytes: &[u8], i: usize) -> Option<usize> {
    let open = bytes[i..].iter().take_while(|b| **b == b'`').count();
    let mut j = i + open;
    while j < bytes.len() {
        let run = bytes[j..].iter().take_while(|b| **b == b'`').count();
        if run == open {
            return Some(j + run);
        }
        j += run.max(1);
    }
    None
}

fn scan_line<'a>(line: &'a str, number: usize, out: &mut Vec<WikiLink<'a>>) {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        i = match bytes[i] {
            b'\\' => i + 2,
            b'`' => code_span_end(bytes, i).unwrap_or_else(|| i + 1),
            b'[' if bytes.get(i + 1) == Some(&b'[') => link_at(line, i, number, out),
            _ => i + 1,
        };
    }
}

/// Parse a link opening at `start`; returns where scanning resumes.
fn link_at<'a>(line: &'a str, start: usize, number: usize, out: &mut Vec<WikiLink<'a>>) -> usize {
    let Some(len) = line[start + 2..].find("]]") else {
        return start + 2;
    };
    let inner = &line[start + 2..start + 2 + len];
    if let Some(link) = parse_inner(inner, number) {
        out.push(link);
    }
    start + 2 + len + 2
}

fn parse_inner(inner: &str, line: usize) -> Option<WikiLink<'_>> {
    if inner.contains(['[', ']']) {
        return None;
    }
    let (left, label) = split_once_opt(inner, '|');
    let (target, heading) = split_once_opt(left, '#');
    let target = target.trim();
    (!target.is_empty()).then(|| WikiLink {
        target,
        heading: heading.map(str::trim).filter(|h| !h.is_empty()),
        label: label.map(str::trim).filter(|l| !l.is_empty()),
        line,
    })
}

fn split_once_opt(text: &str, sep: char) -> (&str, Option<&str>) {
    match text.split_once(sep) {
        Some((a, b)) => (a, Some(b)),
        None => (text, None),
    }
}

/// A piece of inline text: plain, or a wikilink to show as a link.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Piece<'a> {
    /// Plain text.
    Text(&'a str),
    /// A wikilink: what to look up and what to show.
    Link {
        /// Note title or alias.
        target: &'a str,
        /// The label if given, else the target.
        shown: &'a str,
    },
}

/// Split inline text (a single run, no code) into plain text and wikilinks, in order. Escaped
/// `\[[` and malformed links stay plain text.
pub fn split_wikilinks(text: &str) -> Vec<Piece<'_>> {
    let mut pieces = Vec::new();
    let (mut plain_from, mut i) = (0, 0);
    let bytes = text.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        let link = (bytes[i] == b'[' && bytes.get(i + 1) == Some(&b'['))
            .then(|| inner_at(text, i))
            .flatten();
        match link {
            Some((target, shown, end)) => {
                if plain_from < i {
                    pieces.push(Piece::Text(&text[plain_from..i]));
                }
                pieces.push(Piece::Link { target, shown });
                (plain_from, i) = (end, end);
            }
            None => i += 1,
        }
    }
    if plain_from < text.len() {
        pieces.push(Piece::Text(&text[plain_from..]));
    }
    pieces
}

/// The link opening at `start`: (target, shown text, end offset), if well formed.
fn inner_at(text: &str, start: usize) -> Option<(&str, &str, usize)> {
    let len = text[start + 2..].find("]]")?;
    let inner = &text[start + 2..start + 2 + len];
    let link = parse_inner(inner, 0)?;
    Some((
        link.target,
        link.label.unwrap_or(link.target),
        start + 2 + len + 2,
    ))
}
