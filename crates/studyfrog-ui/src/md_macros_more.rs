//! Extended macro expansions and macro table (STORY-0143).

use studyfrog_md::hue::{hue_from_name, hue_hex};

use crate::md_macros::{Expansion, text};

/// `CxR`: columns and body rows, each from 1 to the limit.
fn size(arg: Option<&str>) -> Option<(usize, usize)> {
    let Some(arg) = arg else { return Some((2, 1)) };
    let (c, r) = arg.split_once(['x', 'X'])?;
    let (c, r): (usize, usize) = (c.trim().parse().ok()?, r.trim().parse().ok()?);
    ((1..=12).contains(&c) && (1..=50).contains(&r)).then_some((c, r))
}

/// Markdown table.
pub fn table(arg: Option<&str>) -> Option<Expansion> {
    let (c, r) = size(arg)?;
    let head: Vec<String> = (1..=c).map(|i| format!("Column {i}")).collect();
    let mut out = format!("| {} |\n|{}\n", head.join(" | "), "---|".repeat(c));
    for _ in 0..r {
        out.push_str(&format!("|{}\n", "   |".repeat(c)));
    }
    text(out.trim_end(), "Column 1", true)
}

/// Table of contents `[TOC]`.
pub fn toc(arg: Option<&str>) -> Option<Expansion> {
    arg.is_none().then_some(())?;
    text("[TOC]", "[TOC]", true)
}

/// Fenced code block with optional syntax highlighting language.
pub fn code(arg: Option<&str>) -> Option<Expansion> {
    let lang = arg.unwrap_or("").trim();
    if !lang
        .chars()
        .all(|c| c.is_alphanumeric() || "+#-".contains(c))
    {
        return None;
    }
    text(&format!("```{lang}\ncode\n```"), "code", true)
}

/// Block quotation.
pub fn quote(arg: Option<&str>) -> Option<Expansion> {
    arg.is_none().then_some(())?;
    text("> quote", "quote", true)
}

/// Image picker trigger.
pub fn image(arg: Option<&str>) -> Option<Expansion> {
    arg.is_none().then_some(())?;
    Some(Expansion {
        text: String::new(),
        select: (0, 0),
        own_lines: false,
        picture: true,
    })
}

/// Coloured text span.
pub fn colour(arg: Option<&str>) -> Option<Expansion> {
    let hue = hue_from_name(arg?)?;
    text(
        &format!("<span style=\"color:{}\">coloured</span>", hue_hex(hue)),
        "coloured",
        false,
    )
}

/// Divider rule `---`.
pub fn rule(arg: Option<&str>) -> Option<Expansion> {
    arg.is_none().then_some(())?;
    text("---", "---", true)
}

/// Header with given level or default level 2: `# Header`.
pub fn header(arg: Option<&str>) -> Option<Expansion> {
    let level = arg
        .unwrap_or("2")
        .trim()
        .parse::<usize>()
        .unwrap_or(2)
        .clamp(1, 6);
    let hashes = "#".repeat(level);
    text(&format!("{hashes} Header"), "Header", true)
}

/// Right arrow.
pub fn arrow_right(arg: Option<&str>) -> Option<Expansion> {
    arg.is_none().then_some(())?;
    text("->", "->", false)
}

/// Left arrow.
pub fn arrow_left(arg: Option<&str>) -> Option<Expansion> {
    arg.is_none().then_some(())?;
    text("<-", "<-", false)
}

/// Collapsible details/summary block.
pub fn hide(arg: Option<&str>) -> Option<Expansion> {
    let hidden = arg.unwrap_or("Hidden answer").trim();
    let hidden = if hidden.is_empty() {
        "Hidden answer"
    } else {
        hidden
    };
    text(
        &format!("<details><summary>Reveal</summary>{hidden}</details>"),
        hidden,
        true,
    )
}

/// Note/callout panel.
pub fn panel(arg: Option<&str>) -> Option<Expansion> {
    let kind = arg.unwrap_or("NOTE").trim().to_uppercase();
    let kind = if kind.is_empty() { "NOTE" } else { &kind };
    text(
        &format!("> [!{kind}]\n> Note content"),
        "Note content",
        true,
    )
}

/// Hyperlink template.
pub fn link(arg: Option<&str>) -> Option<Expansion> {
    let url = arg.unwrap_or("url").trim();
    let url = if url.is_empty() { "url" } else { url };
    text(&format!("[Link text]({url})"), "Link text", false)
}

/// Bold text formatting.
pub fn bold(arg: Option<&str>) -> Option<Expansion> {
    let content = arg.unwrap_or("bold text").trim();
    let content = if content.is_empty() {
        "bold text"
    } else {
        content
    };
    let s = content.trim_matches('"');
    text(&format!("**{s}**"), s, false)
}

/// Italic text formatting.
pub fn italic(arg: Option<&str>) -> Option<Expansion> {
    let content = arg.unwrap_or("italic text").trim();
    let content = if content.is_empty() {
        "italic text"
    } else {
        content
    };
    let s = content.trim_matches('"');
    text(&format!("*{s}*"), s, false)
}

/// Mermaid graph template.
pub fn graph(arg: Option<&str>) -> Option<Expansion> {
    let data = arg.unwrap_or("Topic").trim();
    let data = if data.is_empty() { "Topic" } else { data };
    text(
        &format!("```mermaid\ngraph LR\n    A[{data}] --> B\n```"),
        data,
        true,
    )
}
