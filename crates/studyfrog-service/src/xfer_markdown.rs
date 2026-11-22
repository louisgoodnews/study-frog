//! The Markdown reader: a `.md` file is a card deck. Each level-2 heading starts a card: the heading is
//! the front and the text below it, up to the next heading, is the back. A file with no such heading is
//! one note. Headings inside code fences do not count.

use studyfrog_core::plan::ItemKind;

use crate::xfer_model::{Bundle, item};
use crate::xfer_obsidian_front::split_front;

/// What a Markdown card deck has to look like, in plain words (shown in the import dialog and the formats list).
pub const MARKDOWN_LAYOUT: &str = "A Markdown file. Each level-2 heading (## Question) starts a card: the heading is the front and the text below it, up to the next heading, is the back. Text before the first heading is left out with a warning. A file with no level-2 heading is one note, titled by its first level-1 heading.";

/// A sample Markdown deck.
pub const MARKDOWN_EXAMPLE: &str = "# French\n\n## the cat\n\nle chat\n\n## the dog\n\nle chien\n";

/// Whether a line opens or closes a fenced code block.
fn is_fence(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("```") || t.starts_with("~~~")
}

/// Split a body into its `## ` sections (heading, text below it) and the text before the first one.
/// Pure. A `## ` line inside a code fence is text, not a heading.
pub fn sections(body: &str) -> (Vec<(String, String)>, String) {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut preface = String::new();
    let mut in_fence = false;
    for line in body.lines() {
        let heading = if in_fence {
            None
        } else {
            line.strip_prefix("## ")
        };
        if let Some(heading) = heading {
            out.push((heading.trim().to_owned(), String::new()));
            continue;
        }
        if is_fence(line) {
            in_fence = !in_fence;
        }
        let target = match out.last_mut() {
            Some((_, text)) => text,
            None => &mut preface,
        };
        target.push_str(line);
        target.push('\n');
    }
    (out, preface)
}

/// The first level-1 heading of a body, or an empty string. Pure.
fn first_title(body: &str) -> String {
    body.lines()
        .find_map(|l| l.strip_prefix("# "))
        .map(|t| t.trim().to_owned())
        .unwrap_or_default()
}

/// Read a Markdown file's bytes into cards (one per `## ` section) or, with no section, one note.
pub fn from_markdown(bytes: &[u8]) -> Result<Bundle, String> {
    let text =
        std::str::from_utf8(bytes).map_err(|_| "the file is not valid UTF-8 text".to_owned())?;
    let (_, body) = split_front(text);
    if body.trim().is_empty() {
        return Err("the file has no text".to_owned());
    }
    let (sections, preface) = sections(body);
    let mut bundle = Bundle::default();
    if sections.is_empty() {
        let mut note = item(ItemKind::Note, "", body.trim(), "");
        note.title = first_title(body);
        bundle.items.push(note);
        return Ok(bundle);
    }
    let stray = preface
        .lines()
        .any(|l| !l.trim().is_empty() && !l.starts_with("# "));
    if stray {
        bundle
            .warnings
            .push("text before the first ## heading was left out".to_owned());
    }
    for (front, back) in sections {
        if back.trim().is_empty() {
            bundle
                .warnings
                .push(format!("\"{front}\" has no text below it and was left out"));
            continue;
        }
        bundle
            .items
            .push(item(ItemKind::Card, "", &front, back.trim()));
    }
    Ok(bundle)
}
