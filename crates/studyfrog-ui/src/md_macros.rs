//! Text macros: typing `{table}` (or `{toc}`, `{code:python}`, `{color:red}` ...) in a Markdown text turns it into
//! the Markdown it stands for the moment the closing brace is typed, never earlier. A macro is a record in
//! [`MACROS`] with one free function, so adding one is adding a record. Unknown names stay as typed. Pure.

use crate::md_macros_more as m;

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

pub(crate) fn text(text: &str, select: &str, own_lines: bool) -> Option<Expansion> {
    let start = text.find(select).map_or(0, |b| text[..b].chars().count());
    Some(Expansion {
        text: text.to_owned(),
        select: (start, start + select.chars().count()),
        own_lines,
        picture: false,
    })
}

/// Every macro, in the order the hint lists them.
pub const MACROS: &[Macro] = &[
    Macro {
        name: "table",
        usage: "{table} or {table:3x4}",
        help: "A table: columns by body rows",
        expand: m::table,
    },
    Macro {
        name: "toc",
        usage: "{toc}",
        help: "The table of contents of the note",
        expand: m::toc,
    },
    Macro {
        name: "code",
        usage: "{code} or {code:python}",
        help: "A code block, with a language",
        expand: m::code,
    },
    Macro {
        name: "quote",
        usage: "{quote}",
        help: "A quotation",
        expand: m::quote,
    },
    Macro {
        name: "image",
        usage: "{image}",
        help: "Choose a picture to add",
        expand: m::image,
    },
    Macro {
        name: "color",
        usage: "{color:red}",
        help: "Coloured text",
        expand: m::colour,
    },
    Macro {
        name: "rule",
        usage: "{rule}",
        help: "Horizontal dividing rule (---)",
        expand: m::rule,
    },
    Macro {
        name: "header",
        usage: "{header:2} or {h:3}",
        help: "Section heading with level",
        expand: m::header,
    },
    Macro {
        name: "h",
        usage: "{h:2}",
        help: "Shorthand heading",
        expand: m::header,
    },
    Macro {
        name: "arrow_right",
        usage: "{arrow_right} or {->}",
        help: "Right arrow (->)",
        expand: m::arrow_right,
    },
    Macro {
        name: "->",
        usage: "{->}",
        help: "Right arrow shorthand",
        expand: m::arrow_right,
    },
    Macro {
        name: "arrow_left",
        usage: "{arrow_left} or {<-}",
        help: "Left arrow (<-)",
        expand: m::arrow_left,
    },
    Macro {
        name: "<-",
        usage: "{<-}",
        help: "Left arrow shorthand",
        expand: m::arrow_left,
    },
    Macro {
        name: "hide",
        usage: "{hide} or {spoiler}",
        help: "Hidden answer drawer",
        expand: m::hide,
    },
    Macro {
        name: "spoiler",
        usage: "{spoiler}",
        help: "Collapsible spoiler",
        expand: m::hide,
    },
    Macro {
        name: "panel",
        usage: "{panel} or {panel:WARNING}",
        help: "Callout note or warning block",
        expand: m::panel,
    },
    Macro {
        name: "callout",
        usage: "{callout}",
        help: "Callout block",
        expand: m::panel,
    },
    Macro {
        name: "link",
        usage: "{link} or {link:https://...}",
        help: "Markdown link template",
        expand: m::link,
    },
    Macro {
        name: "bold",
        usage: "{bold:text} or {b:text}",
        help: "Bold text (**text**)",
        expand: m::bold,
    },
    Macro {
        name: "b",
        usage: "{b:text}",
        help: "Bold text shorthand",
        expand: m::bold,
    },
    Macro {
        name: "italic",
        usage: "{italic:text} or {i:text}",
        help: "Italic text (*text*)",
        expand: m::italic,
    },
    Macro {
        name: "i",
        usage: "{i:text}",
        help: "Italic text shorthand",
        expand: m::italic,
    },
    Macro {
        name: "graph",
        usage: "{graph:Topic}",
        help: "Mermaid flowchart diagram template",
        expand: m::graph,
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
    // STORY-0144: '#' prefix suppresses expansion (e.g. {#code})
    if inner.starts_with('#') {
        return None;
    }
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
