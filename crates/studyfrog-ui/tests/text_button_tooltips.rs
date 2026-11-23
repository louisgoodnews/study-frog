#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Hover explanations for text buttons (STORY-0189), checked by scanning the source. A button call is
//! explained when its chain gives it `on_hover_text` (or a `view::tips` text), or when it is bound to a
//! name that is added with such a chain. A new button without an explanation fails the test.

use std::fs;
use std::path::{Path, PathBuf};

/// Marks a button whose text is set by its caller, which explains it (a shared helper).
const CALLER_HOVER: &str = "hover: from the caller";

/// Words that make a hover text (in a call's chain).
const HOVER: [&str; 3] = ["on_hover_text", "tips::", "on_hover_ui"];

/// Calls that draw a text button. Definitions (`fn primary_button(`) are not calls.
const CALLS: [&str; 8] = [
    ".button(",
    ".menu_button(",
    "primary_button(",
    "quiet_button(",
    "Button::new(",
    "Button::selectable(",
    ".small_button(",
    ".link(",
];

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// The index just after the string literal that starts at `i` (a `"`).
fn after_string(b: &[u8], mut i: usize) -> usize {
    i += 1;
    while i < b.len() {
        match b[i] {
            b'\\' => i += 2,
            b'"' => return i + 1,
            _ => i += 1,
        }
    }
    b.len()
}

/// The index of the `)` that closes the `(` at `open`, skipping string literals.
fn close_of(b: &[u8], open: usize) -> usize {
    let mut depth = 0usize;
    let mut i = open;
    while i < b.len() {
        match b[i] {
            b'"' => {
                i = after_string(b, i);
                continue;
            }
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return i;
                }
            }
            _ => {}
        }
        i += 1;
    }
    b.len().saturating_sub(1)
}

/// The index after a call's `)` and any `.method(...)` calls chained on it.
fn chain_end(b: &[u8], close: usize) -> usize {
    let mut j = close + 1;
    loop {
        let mut k = j;
        while k < b.len() && b[k].is_ascii_whitespace() {
            k += 1;
        }
        if k >= b.len() || b[k] != b'.' {
            return j;
        }
        let mut m = k + 1;
        while m < b.len() && (b[m].is_ascii_alphanumeric() || b[m] == b'_') {
            m += 1;
        }
        if m == k + 1 {
            return j;
        }
        let mut p = m;
        while p < b.len() && (b[p] == b' ' || b[p] == b'\t') {
            p += 1;
        }
        j = if p < b.len() && b[p] == b'(' {
            close_of(b, p) + 1
        } else {
            m
        };
    }
}

/// The index of the unmatched `(` before `at`, if there is one.
fn enclosing_open(b: &[u8], at: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut i = at;
    while i > 0 {
        i -= 1;
        match b[i] {
            b')' => depth += 1,
            b'(' => {
                if depth == 0 {
                    return Some(i);
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    None
}

/// The name bound by the `let [mut] NAME =` that holds the call at `at`, if there is one.
fn bound_name(text: &str, at: usize) -> Option<String> {
    let back = text[..at].rfind("let ")?;
    let mut rest = text[back + 4..].trim_start();
    if let Some(r) = rest.strip_prefix("mut ") {
        rest = r.trim_start();
    }
    let name: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    let after = rest[name.len()..].trim_start();
    (!name.is_empty() && after.starts_with('=') && !after.starts_with("==")).then_some(name)
}

/// Whether a use of `name` after `at` is the argument of an `add` or `add_enabled` that has a hover
/// text in its chain.
fn used_with_hover(text: &str, at: usize, name: &str) -> bool {
    let b = text.as_bytes();
    let end = (at + 4000).min(text.len());
    let is_word = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    for (off, _) in text[at..end].match_indices(name) {
        let u = at + off;
        let before = if u == 0 { b' ' } else { b[u - 1] };
        let after = b.get(u + name.len()).copied().unwrap_or(b' ');
        if is_word(before) || is_word(after) {
            continue;
        }
        let Some(open) = enclosing_open(b, u) else {
            continue;
        };
        if !(text[..open].ends_with(".add") || text[..open].ends_with(".add_enabled")) {
            continue;
        }
        let chain = &text[open..chain_end(b, close_of(b, open))];
        if HOVER.iter().any(|h| chain.contains(h)) {
            return true;
        }
    }
    false
}

/// Whether the call at `at` (its pattern `call`) is explained: its text, with the chain on it, or, for
/// a `Button::new` or `Button::selectable` passed to `add`, the chain on that `add`, names a hover text. A
/// `Button::` call bound to a name is explained when that name is added with a hover text.
fn explained(text: &str, at: usize, call: &str) -> bool {
    let b = text.as_bytes();
    let (start, close) = if call.starts_with("Button::") {
        match enclosing_open(b, at) {
            Some(open)
                if text[..open].ends_with(".add") || text[..open].ends_with(".add_enabled") =>
            {
                let head = text[..open].rfind('.').unwrap_or(open);
                (head, close_of(b, open))
            }
            _ => (at, close_of(b, at + call.len() - 1)),
        }
    } else {
        (at, close_of(b, at + call.len() - 1))
    };
    let window = &text[start..chain_end(b, close)];
    if HOVER.iter().any(|h| window.contains(h)) {
        return true;
    }
    call.starts_with("Button::")
        && bound_name(text, at).is_some_and(|n| used_with_hover(text, at, &n))
}

/// Every call in the UI source that has no explanation: "file:line".
fn unexplained() -> Vec<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    sources(&root, &mut files);
    files.sort();
    let mut missing = Vec::new();
    for file in files {
        let text = fs::read_to_string(&file).unwrap();
        for call in CALLS {
            for (at, _) in text.match_indices(call) {
                let line_start = text[..at].rfind('\n').map_or(0, |i| i + 1);
                let line_end = text[at..].find('\n').map_or(text.len(), |i| at + i);
                let line = &text[line_start..line_end];
                if line.trim_start().starts_with("//") || line.contains("fn ") {
                    continue;
                }
                if line.contains(CALLER_HOVER) {
                    continue;
                }
                if !explained(&text, at, call) {
                    let n = text[..at].matches('\n').count() + 1;
                    let shown = file.strip_prefix(&root).unwrap_or(&file).display();
                    missing.push(format!("{shown}:{n}"));
                }
            }
        }
    }
    missing
}

#[test]
fn no_new_text_button_goes_without_a_hover_explanation() {
    let missing = unexplained();
    assert!(
        missing.is_empty(),
        "{} text buttons have no hover explanation; add `.on_hover_text(...)` from `view::tips` to each:\n{}",
        missing.len(),
        missing.join("\n")
    );
}

#[test]
fn the_scan_finds_a_button_without_an_explanation() {
    let sample = "fn draw(ui: &mut Ui) {\n    if ui.button(\"Plain\").clicked() {\n    }\n    let btn = egui::Button::new(\"Bound\");\n    ui.add(btn).on_hover_text(\"w\");\n    ui.button(\"Explained\").on_hover_text(\"x\");\n    ui.add(egui::Button::new(\"Wrapped\")).on_hover_text(\"y\");\n    ui.menu_button(\"Menu\", |ui| {\n        ui.label(\"x\");\n    }).response.on_hover_text(\"z\");\n}\n";
    let mut found = Vec::new();
    for call in CALLS {
        for (at, _) in sample.match_indices(call) {
            if !explained(sample, at, call) {
                found.push(at);
            }
        }
    }
    assert_eq!(found.len(), 1, "only the plain button is unexplained");
}
