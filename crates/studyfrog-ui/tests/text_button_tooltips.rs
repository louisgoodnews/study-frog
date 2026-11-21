#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Hover explanations for text buttons (STORY-0189), checked by scanning the source. A button call is
//! explained when the statement it sits in gives it `on_hover_text` (or a `view::tips` text). The
//! count of unexplained calls may only go down: a new button without an explanation fails the test.

use std::fs;
use std::path::{Path, PathBuf};

/// Calls that draw a text button. Definitions (`fn primary_button(`) are not calls.
const CALLS: [&str; 6] = [
    ".button(",
    ".menu_button(",
    "primary_button(",
    "quiet_button(",
    "Button::new(",
    ".small_button(",
];

/// Unexplained calls when this test was written. Lower it when a button gets an explanation; it must
/// never go up.
const BASELINE: usize = 415;

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

/// The text of the statement a call sits in: up to the next `;` or `{`, so a closure body is not part
/// of it.
fn statement(text: &str, from: usize) -> &str {
    let rest = &text[from..];
    let end = rest.find([';', '{']).unwrap_or(rest.len()).min(400);
    &rest[..end]
}

fn explained(statement: &str) -> bool {
    statement.contains("on_hover_text")
        || statement.contains("tips::")
        || statement.contains("on_hover_ui")
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
                let line = &text[line_start..text[at..].find('\n').map_or(text.len(), |i| at + i)];
                if line.trim_start().starts_with("//") || line.contains("fn ") {
                    continue;
                }
                if !explained(statement(&text, at)) {
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
        missing.len() <= BASELINE,
        "{} text buttons have no hover explanation (the limit is {BASELINE}); add `.on_hover_text(...)` \
         from `view::tips`, or lower BASELINE when you add them:\n{}",
        missing.len(),
        missing.join("\n")
    );
}

#[test]
fn the_scan_finds_a_button_without_an_explanation() {
    let sample = "fn draw(ui: &mut Ui) {\n    if ui.button(\"Plain\").clicked() {\n    }\n    ui.button(\"Explained\").on_hover_text(\"x\");\n}\n";
    let mut found = Vec::new();
    for call in CALLS {
        for (at, _) in sample.match_indices(call) {
            if !explained(statement(sample, at)) {
                found.push(at);
            }
        }
    }
    assert_eq!(found.len(), 1, "only the plain button is unexplained");
}
