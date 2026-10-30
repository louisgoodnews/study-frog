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
