//! Text macros: typing `{table}` (or `{toc}`, `{code:python}`, `{color:red}` ...) in a Markdown text turns it into
//! the Markdown it stands for the moment the closing brace is typed, never earlier. A macro is a record in
//! [`MACROS`] with one free function, so adding one is adding a record. Unknown names stay as typed. Pure.

use studyfrog_md::hue::{hue_from_name, hue_hex};

/// What a macro writes in place of itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expansion {
    /// The Markdown.
    pub text: String,
    /// The selection afterwards, in characters from the start of `text`.
    pub select: (usize, usize),
    /// The macro needs lines of its own (a table, a code block, the contents).
    pub own_lines: bool,
    /// Open the picture chooser instead of writing text.
    pub picture: bool,
}

/// A macro: its name, how it is written, what it does, and the function that writes it.
#[derive(Debug, Clone, Copy)]
pub struct Macro {
    /// The name, lower case.
    pub name: &'static str,
    /// How to type it.
    pub usage: &'static str,
    /// What it writes.
    pub help: &'static str,
    /// The expansion for the argument after the colon, if there is one; `None` when the argument is not valid.
    pub expand: fn(Option<&str>) -> Option<Expansion>,
}

fn text(text: &str, select: &str, own_lines: bool) -> Option<Expansion> {
    let start = text.find(select).map_or(0, |b| text[..b].chars().count());
    Some(Expansion {
        text: text.to_owned(),
        select: (start, start + select.chars().count()),
        own_lines,
        picture: false,
    })
}

/// `CxR`: columns and body rows, each from 1 to the limit.
fn size(arg: Option<&str>) -> Option<(usize, usize)> {
    let Some(arg) = arg else { return Some((2, 1)) };
    let (c, r) = arg.split_once(['x', 'X'])?;
    let (c, r): (usize, usize) = (c.trim().parse().ok()?, r.trim().parse().ok()?);
    ((1..=12).contains(&c) && (1..=50).contains(&r)).then_some((c, r))
}

fn table(arg: Option<&str>) -> Option<Expansion> {
    let (c, r) = size(arg)?;
    let head: Vec<String> = (1..=c).map(|i| format!("Column {i}")).collect();
    let mut out = format!("| {} |\n|{}\n", head.join(" | "), "---|".repeat(c));
    for _ in 0..r {
        out.push_str(&format!("|{}\n", "   |".repeat(c)));
    }
    text(out.trim_end(), "Column 1", true)
}

fn toc(arg: Option<&str>) -> Option<Expansion> {
    arg.is_none().then_some(())?;
    text("[TOC]", "[TOC]", true)
}

fn code(arg: Option<&str>) -> Option<Expansion> {
    let lang = arg.unwrap_or("").trim();
    if !lang
        .chars()
        .all(|c| c.is_alphanumeric() || "+#-".contains(c))
    {
        return None;
    }
    text(&format!("```{lang}\ncode\n```"), "code", true)
}

fn quote(arg: Option<&str>) -> Option<Expansion> {
    arg.is_none().then_some(())?;
    text("> quote", "quote", true)
}

fn image(arg: Option<&str>) -> Option<Expansion> {
    arg.is_none().then_some(())?;
    Some(Expansion {
        text: String::new(),
        select: (0, 0),
        own_lines: false,
        picture: true,
    })
}

fn colour(arg: Option<&str>) -> Option<Expansion> {
    let hue = hue_from_name(arg?)?;
    text(
        &format!("<span style=\"color:{}\">coloured</span>", hue_hex(hue)),
        "coloured",
        false,
    )
}

/// Every macro, in the order the hint lists them.
pub const MACROS: &[Macro] = &[
    Macro {
        name: "table",
        usage: "{table} or {table:3x4}",
        help: "A table: columns by body rows (2 by 1 without a size)",
        expand: table,
    },
    Macro {
        name: "toc",
        usage: "{toc}",
        help: "The table of contents of the note",
        expand: toc,
    },
    Macro {
        name: "code",
        usage: "{code} or {code:python}",
        help: "A code block, with a language for colours",
        expand: code,
    },
    Macro {
        name: "quote",
        usage: "{quote}",
        help: "A quotation",
        expand: quote,
    },
    Macro {
        name: "image",
        usage: "{image}",
        help: "Choose a picture to add",
        expand: image,
    },
    Macro {
        name: "color",
        usage: "{color:red}",
        help: "Coloured text: red, orange, green, blue, purple or grey",
        expand: colour,
    },
];

/// The macro that ends at `caret` (in characters) with a closing brace, with the range it covers (start, end in
/// characters) and what it writes. `None` when the text before the caret is not a complete, known macro.
pub fn macro_before(text: &str, caret: usize) -> Option<((usize, usize), Expansion)> {
    let chars: Vec<char> = text.chars().collect();
    if caret == 0 || caret > chars.len() || chars[caret - 1] != '}' {
        return None;
    }
    let open = chars[..caret - 1]
        .iter()
        .rposition(|c| matches!(c, '{' | '}' | '\n'))?;
    if chars[open] != '{' {
        return None;
    }
    let inner: String = chars[open + 1..caret - 1].iter().collect();
    let (name, arg) = match inner.split_once(':') {
        Some((n, a)) => (n, Some(a)),
        None => (inner.as_str(), None),
    };
    let found = MACROS
        .iter()
        .find(|m| m.name.eq_ignore_ascii_case(name.trim()))?;
    Some(((open, caret), (found.expand)(arg)?))
}

/// Put the expansion where the macro was typed: lines of its own get blank lines around them. Returns the new
/// text and the selection.
pub fn expand_in(text: &str, range: (usize, usize), e: &Expansion) -> (String, (usize, usize)) {
    let chars: Vec<char> = text.chars().collect();
    let head: String = chars[..range.0].iter().collect();
    let tail: String = chars[range.1..].iter().collect();
    let on_own_line = head.is_empty() || head.ends_with('\n');
    let lead = if e.own_lines && !on_own_line {
        "\n\n"
    } else {
        ""
    };
    let gap = if e.own_lines && !tail.starts_with('\n') && !tail.is_empty() {
        "\n"
    } else {
        ""
    };
    let start = range.0 + lead.chars().count();
    (
        format!("{head}{lead}{}{gap}{tail}", e.text),
        (start + e.select.0, start + e.select.1),
    )
}
