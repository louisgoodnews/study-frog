//! Inline flashcards in markdown notes (TASK-0269, STORY-0161).
//!
//! Scans markdown documents for `Term :: Definition` and reversed `Term ::: Definition`
//! delimiters outside of code blocks and code fences, tracking 1-based line numbers.
//! Pure functions, zero inherent impls (P1).

/// An inline flashcard parsed from note text.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InlineCard {
    /// Prompt or front of the card.
    pub term: String,
    /// Answer or back of the card.
    pub definition: String,
    /// 1-based line number in source markdown.
    pub line_number: usize,
    /// The unparsed source line.
    pub raw: String,
}

struct ParsedLine {
    term: String,
    def: String,
    reversed: bool,
    span: (usize, usize),
}

/// Masks inline backtick code spans with spaces of identical byte length.
fn mask_inline_code(line: &str) -> String {
    let mut bytes = line.as_bytes().to_vec();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'`' {
            let start = i;
            while i < bytes.len() && bytes[i] == b'`' {
                i += 1;
            }
            let len = i - start;
            let seq = &line.as_bytes()[start..i];
            if let Some(off) = line.as_bytes()[i..].windows(len).position(|w| w == seq) {
                let end = i + off + len;
                for b in &mut bytes[start..end] {
                    *b = b' ';
                }
                i = end;
            }
        } else {
            i += 1;
        }
    }
    String::from_utf8(bytes).unwrap_or_else(|_| line.to_owned())
}

/// Strips list bullet markers (`- `, `* `, `+ `, or numbered lists `1. `, `1) `).
fn strip_bullet(s: &str) -> &str {
    let trimmed = s.trim_start();
    for bullet in &["- ", "* ", "+ "] {
        if let Some(rest) = trimmed.strip_prefix(bullet) {
            return rest.trim_start();
        }
    }
    if let Some(pos) = trimmed.find(['.', ')'])
        && pos > 0
        && trimmed[..pos].chars().all(|c| c.is_ascii_digit())
    {
        let after = &trimmed[pos + 1..];
        if after.starts_with(' ') {
            return after.trim_start();
        }
    }
    trimmed
}

/// Finds the delimiter pattern in masked text, avoiding adjacent colons.
fn find_delim_pat(masked: &str, pat: &str) -> Option<(usize, usize)> {
    let mut from = 0;
    let n = pat.len();
    while let Some(rel) = masked[from..].find(pat) {
        let idx = from + rel;
        let prev = idx > 0 && masked.as_bytes()[idx - 1] == b':';
        let next = masked.as_bytes().get(idx + n) == Some(&b':');
        if prev || next {
            from = idx + n;
            continue;
        }
        let b_sp = idx > 0 && masked.as_bytes()[idx - 1] == b' ';
        let a_sp = masked.as_bytes().get(idx + n) == Some(&b' ');
        if b_sp || a_sp {
            let s = if b_sp { idx - 1 } else { idx };
            let e = if a_sp { idx + n + 1 } else { idx + n };
            return Some((s, e));
        }
        from = idx + n;
    }
    None
}

/// Delimiter match info: (start_idx, end_idx, reversed).
fn find_delimiter(masked: &str) -> Option<(usize, usize, bool)> {
    if let Some((s, e)) = find_delim_pat(masked, ":::") {
        return Some((s, e, true));
    }
    find_delim_pat(masked, "::").map(|(s, e)| (s, e, false))
}

/// Parses a line into term, definition, reversed flag, and delimiter byte span.
fn parse_line(line: &str) -> Option<ParsedLine> {
    let t = line.trim();
    if (t.starts_with('|') && t.ends_with('|')) || t.starts_with('#') {
        return None;
    }
    let masked = mask_inline_code(line);
    let (d_start, d_end, reversed) = find_delimiter(&masked)?;
    let term = strip_bullet(&line[..d_start]).trim().to_owned();
    let def = line[d_end..].trim().to_owned();
    if term.is_empty() || def.is_empty() {
        return None;
    }
    Some(ParsedLine {
        term,
        def,
        reversed,
        span: (d_start, d_end),
    })
}

fn check_fence(line: &str, fenced: &mut bool, math: &mut bool) -> bool {
    let t = line.trim_start();
    if t.starts_with("```") || t.starts_with("~~~") {
        *fenced = !*fenced;
        return true;
    }
    if *fenced {
        return true;
    }
    if let Some(rest) = t.strip_prefix("$$") {
        if !rest.contains("$$") {
            *math = !*math;
        }
        return true;
    }
    *math
}

/// Scans markdown text line by line for inline card delimiters.
pub fn scan_inline_cards(markdown: &str) -> Vec<InlineCard> {
    let mut cards = Vec::new();
    let (mut fenced, mut math) = (false, false);

    for (idx, line) in markdown.lines().enumerate() {
        if check_fence(line, &mut fenced, &mut math) {
            continue;
        }
        let line_number = idx + 1;
        if let Some(parsed) = parse_line(line) {
            let raw = line.to_owned();
            cards.push(InlineCard {
                term: parsed.term.clone(),
                definition: parsed.def.clone(),
                line_number,
                raw: raw.clone(),
            });
            if parsed.reversed {
                cards.push(InlineCard {
                    term: parsed.def,
                    definition: parsed.term,
                    line_number,
                    raw,
                });
            }
        }
    }
    cards
}

/// Returns true if markdown contains at least one inline card.
pub fn contains_inline_cards(markdown: &str) -> bool {
    let (mut fenced, mut math) = (false, false);
    for line in markdown.lines() {
        if check_fence(line, &mut fenced, &mut math) {
            continue;
        }
        if parse_line(line).is_some() {
            return true;
        }
    }
    false
}

/// Strips inline card delimiters (`::` and `:::`) from markdown, replacing them with `: `.
pub fn strip_inline_card_markers(markdown: &str) -> String {
    let mut out = String::with_capacity(markdown.len());
    let (mut fenced, mut math) = (false, false);

    for line in markdown.split_inclusive('\n') {
        if check_fence(line, &mut fenced, &mut math) {
            out.push_str(line);
            continue;
        }
        let line_no_nl = line.trim_end_matches(['\r', '\n']);
        let nl = &line[line_no_nl.len()..];
        if let Some(parsed) = parse_line(line_no_nl) {
            let (s, e) = parsed.span;
            out.push_str(&line_no_nl[..s]);
            out.push_str(": ");
            out.push_str(line_no_nl[e..].trim_start());
            out.push_str(nl);
        } else {
            out.push_str(line);
        }
    }
    out
}
