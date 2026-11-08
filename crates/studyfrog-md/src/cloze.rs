//! Cloze deletions (STORY-0108, `docs/specs/cloze.md`): `{{c1::answer}}` and `{{c1::answer::hint}}` inside
//! the text of a card or note. Numbers 1 to 99 group the gaps; one group is asked at a time. Pure: the
//! file keeps the syntax as written, these functions only make what a run shows.

use crate::hue::{Hue, span_open};

/// One gap of a text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClozeSpan {
    /// The group number, 1 to 99.
    pub index: u8,
    /// The hidden words.
    pub answer: String,
    /// The hint shown in the gap, if any.
    pub hint: Option<String>,
    /// Byte range of the whole `{{...}}` in the text.
    pub range: (usize, usize),
    /// The gap is inside fenced code.
    pub fenced: bool,
}

/// The highest group number.
pub const MAX_GROUP: u8 = 99;

/// The gap starting at `at` (`{{c`), with the byte just after it.
fn span_at(text: &str, at: usize) -> Option<(u8, &str, Option<&str>, usize)> {
    let rest = text.get(at + 3..)?;
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    let index: u8 = rest.get(..digits)?.parse().ok()?;
    let body = rest.get(digits..)?.strip_prefix("::")?;
    let close = body.find("}}")?;
    let inner = &body[..close];
    if !(1..=MAX_GROUP).contains(&index) || inner.contains('\n') {
        return None;
    }
    let (answer, hint) = match inner.split_once("::") {
        Some((a, h)) => (a, Some(h.trim()).filter(|h| !h.is_empty())),
        None => (inner, None),
    };
    let end = at + 3 + digits + 2 + close + 2;
    (!answer.trim().is_empty()).then_some((index, answer, hint, end))
}

/// Every gap of `text`, in order. Text that only looks like a gap (no number, an empty answer, no
/// closing braces on the line) stays text.
pub fn cloze_spans(text: &str) -> Vec<ClozeSpan> {
    let mut out = Vec::new();
    let mut offset = 0;
    let mut fenced = false;
    for line in text.split_inclusive('\n') {
        let t = line.trim_start();
        if t.starts_with("```") || t.starts_with("~~~") {
            fenced = !fenced;
        }
        let mut from = 0;
        while let Some(i) = line[from..].find("{{c") {
            let at = offset + from + i;
            match span_at(text, at) {
                Some((index, answer, hint, end)) => {
                    out.push(ClozeSpan {
                        index,
                        answer: answer.to_owned(),
                        hint: hint.map(str::to_owned),
                        range: (at, end),
                        fenced,
                    });
                    from = end - offset;
                }
                None => from += i + 3,
            }
        }
        offset += line.len();
    }
    out
}

/// The group numbers used in `text`, ascending, each once.
pub fn cloze_groups(text: &str) -> Vec<u8> {
    let mut groups: Vec<u8> = cloze_spans(text).iter().map(|s| s.index).collect();
    groups.sort_unstable();
    groups.dedup();
    groups
}

/// The group a run asks after `reviews` earlier reviews: the groups take turns. `fixed` (the card's
/// `cloze_group` key) asks always that group when the text has it.
pub fn active_group(groups: &[u8], reviews: usize, fixed: Option<u8>) -> Option<u8> {
    if let Some(g) = fixed.filter(|g| groups.contains(g)) {
        return Some(g);
    }
    (!groups.is_empty()).then(|| groups[reviews % groups.len()])
}

/// `text` with each gap replaced by what `show` makes of it.
fn replace(text: &str, show: impl Fn(&ClozeSpan) -> String) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for span in cloze_spans(text) {
        out.push_str(&text[last..span.range.0]);
        out.push_str(&show(&span));
        last = span.range.1;
    }
    out.push_str(&text[last..]);
    out
}

/// What a run shows before the answer: the gaps of `active` as `[...]` (or `[hint]`), set off as
/// code, the other gaps as their words.
pub fn mask_cloze(text: &str, active: u8) -> String {
    replace(text, |s| {
        if s.index != active {
            return s.answer.clone();
        }
        let gap = format!("[{}]", s.hint.as_deref().unwrap_or("..."));
        if s.fenced { gap } else { format!("`{gap}`") }
    })
}

/// What a run shows after the answer: the words of `active` in bold and green, the others plain.
pub fn reveal_cloze(text: &str, active: u8) -> String {
    replace(text, |s| highlight(s, s.index == active))
}

/// Every gap shown with its words highlighted (the editor's preview).
pub fn preview_cloze(text: &str) -> String {
    replace(text, |s| highlight(s, true))
}

fn highlight(s: &ClozeSpan, on: bool) -> String {
    if !on || s.fenced {
        return s.answer.clone();
    }
    format!("**{}{}</span>**", span_open(Hue::Green), s.answer.trim())
}

/// The hidden words of `active`, for comparing a typed answer.
pub fn cloze_answers(text: &str, active: u8) -> Vec<String> {
    cloze_spans(text)
        .into_iter()
        .filter(|s| s.index == active)
        .map(|s| s.answer.trim().to_owned())
        .collect()
}

/// The number a new gap gets: one more than the highest used (1 in a text without gaps), at most 99.
pub fn next_cloze_index(text: &str) -> u8 {
    cloze_groups(text)
        .last()
        .map_or(1, |g| g.saturating_add(1).min(MAX_GROUP))
}

/// The number a gap of the same group gets: the highest used (1 in a text without gaps).
pub fn last_cloze_index(text: &str) -> u8 {
    cloze_groups(text).last().copied().unwrap_or(1)
}

/// Every gap as a plain `[...]`, for a title that gives nothing away.
pub fn blank_cloze(text: &str) -> String {
    replace(text, |_| "[...]".to_owned())
}
