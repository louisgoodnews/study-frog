//! `name.pdf.notes.toml`, next to the PDF it belongs to: the highlights and notes the user made.
//! The PDF is never changed. Pages are 1-based in the file, rectangles are in points from the
//! top left, and each mark keeps the text it covers so a new revision of the PDF can be matched.

use std::collections::HashSet;

use serde::Deserialize;

use crate::error::{ConfigError, config_error, from_toml};

/// What is added to the PDF's file name to get its notes file.
pub const MARKS_SUFFIX: &str = ".notes.toml";
/// Most marks one file may hold.
pub const MAX_MARKS: usize = 5000;
/// The colours a mark can have: name and RGB (0 to 1).
pub const MARK_COLORS: [(&str, [f32; 3]); 5] = [
    ("yellow", [1.0, 0.84, 0.0]),
    ("green", [0.4, 0.85, 0.4]),
    ("blue", [0.4, 0.7, 1.0]),
    ("pink", [1.0, 0.55, 0.75]),
    ("orange", [1.0, 0.65, 0.2]),
];

/// A highlight over text, or a note at a point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkKind {
    /// Text marked with a colour (and perhaps a note).
    Highlight,
    /// A note pinned to a point.
    Note,
}

/// A rectangle in points from the top left of the page.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MarkRect {
    /// Left edge.
    pub x: f32,
    /// Top edge.
    pub y: f32,
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
}

/// One highlight or note.
#[derive(Debug, Clone, PartialEq)]
pub struct PdfMark {
    /// Unique in the file.
    pub id: u32,
    /// Page, 0-based here.
    pub page: usize,
    /// Highlight or note.
    pub kind: MarkKind,
    /// A name from [`MARK_COLORS`].
    pub color: String,
    /// Where it is on the page.
    pub rects: Vec<MarkRect>,
    /// The text a highlight covers.
    pub covered: String,
    /// What the user wrote.
    pub note: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFile {
    #[serde(default)]
    mark: Vec<RawMark>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMark {
    id: u32,
    page: usize,
    kind: String,
    color: String,
    rects: Vec<[f32; 4]>,
    #[serde(default)]
    text: String,
    #[serde(default)]
    note: String,
}

/// The notes file that belongs to a PDF path.
pub fn marks_path(pdf: &str) -> String {
    format!("{pdf}{MARKS_SUFFIX}")
}

/// The RGB of a colour name, yellow for an unknown name.
pub fn mark_rgb(name: &str) -> [f32; 3] {
    MARK_COLORS
        .iter()
        .find(|(n, _)| *n == name)
        .map_or(MARK_COLORS[0].1, |(_, c)| *c)
}

/// The next free id.
pub fn next_mark_id(marks: &[PdfMark]) -> u32 {
    marks.iter().map(|m| m.id).max().map_or(1, |m| m + 1)
}

fn convert(raw: RawMark) -> Result<PdfMark, ConfigError> {
    let kind = match raw.kind.as_str() {
        "highlight" => MarkKind::Highlight,
        "note" => MarkKind::Note,
        other => {
            return Err(config_error(
                None,
                format!("mark {}: unknown kind \"{other}\"", raw.id),
            ));
        }
    };
    if raw.page == 0 {
        return Err(config_error(
            None,
            format!("mark {}: pages start at 1", raw.id),
        ));
    }
    if !MARK_COLORS.iter().any(|(n, _)| *n == raw.color) {
        return Err(config_error(
            None,
            format!("mark {}: unknown colour \"{}\"", raw.id, raw.color),
        ));
    }
    if raw.rects.is_empty() || raw.rects.iter().flatten().any(|v| !v.is_finite()) {
        return Err(config_error(
            None,
            format!(
                "mark {}: it needs at least one rectangle of numbers",
                raw.id
            ),
        ));
    }
    let rects = raw
        .rects
        .iter()
        .map(|r| MarkRect {
            x: r[0],
            y: r[1],
            width: r[2],
            height: r[3],
        })
        .collect();
    Ok(PdfMark {
        id: raw.id,
        page: raw.page - 1,
        kind,
        color: raw.color,
        rects,
        covered: raw.text,
        note: raw.note,
    })
}

/// Parse a notes file. Unknown keys, repeated ids, bad colours and too many marks are errors.
pub fn parse_marks(text: &str) -> Result<Vec<PdfMark>, ConfigError> {
    let raw: RawFile = toml::from_str(text).map_err(|e| from_toml(text, &e))?;
    if raw.mark.len() > MAX_MARKS {
        return Err(config_error(None, format!("more than {MAX_MARKS} marks")));
    }
    let marks = raw
        .mark
        .into_iter()
        .map(convert)
        .collect::<Result<Vec<_>, _>>()?;
    let mut seen = HashSet::new();
    match marks.iter().find(|m| !seen.insert(m.id)) {
        Some(m) => Err(config_error(None, format!("the id {} is used twice", m.id))),
        None => Ok(marks),
    }
}

/// A TOML basic string that keeps line breaks.
fn quoted(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push(' '),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn render_mark(m: &PdfMark) -> String {
    let kind = if m.kind == MarkKind::Note {
        "note"
    } else {
        "highlight"
    };
    let rects: Vec<String> = m
        .rects
        .iter()
        .map(|r| format!("[{}, {}, {}, {}]", r.x, r.y, r.width, r.height))
        .collect();
    let mut out = format!(
        "\n[[mark]]\nid = {}\npage = {}\nkind = \"{kind}\"\ncolor = \"{}\"\nrects = [{}]\n",
        m.id,
        m.page + 1,
        m.color,
        rects.join(", ")
    );
    if !m.covered.is_empty() {
        out.push_str(&format!("text = {}\n", quoted(&m.covered)));
    }
    if !m.note.is_empty() {
        out.push_str(&format!("note = {}\n", quoted(&m.note)));
    }
    out
}

/// The file text for a list of marks, in page order.
pub fn render_marks(marks: &[PdfMark]) -> String {
    let mut sorted: Vec<&PdfMark> = marks.iter().collect();
    sorted.sort_by_key(|m| (m.page, m.id));
    let mut out = String::from(
        "# Highlights and notes for the PDF next to this file (edited in StudyFrog).\n",
    );
    sorted
        .into_iter()
        .for_each(|m| out.push_str(&render_mark(m)));
    out
}

/// The page a mark is on now: its own page if the text it covers is still there, else the first page
/// that holds that text, else `None`. `pages` are the pages' plain texts.
pub fn relocate(mark: &PdfMark, pages: &[String]) -> Option<usize> {
    let squash = |s: &str| {
        s.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase()
    };
    let needle = squash(&mark.covered);
    if needle.is_empty() {
        return (mark.page < pages.len()).then_some(mark.page);
    }
    let holds = |i: usize| pages.get(i).is_some_and(|p| squash(p).contains(&needle));
    if holds(mark.page) {
        return Some(mark.page);
    }
    (0..pages.len()).find(|i| holds(*i))
}
