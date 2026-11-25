//! Choice questions (EPIC-072): the answer type, which is stored in the front matter as `answer_type`, and the
//! option list, which is read from and written back to the Markdown body. Pure: no egui, no vault access.
//! The body's option markers must agree with the stated answer type; a disagreement is reported, never fixed.

use studyfrog_md::choice::{ChoiceBlock, parse_choice};

#[cfg(test)]
mod tests;

/// What a question is answered with. Open text is the default and has no option lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnswerType {
    #[default]
    Open,
    Single,
    Multiple,
}

/// One option of the list: its text and whether it is correct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub text: String,
    pub correct: bool,
}

/// The answer type as the front matter and the control protocol spell it.
pub fn answer_type_name(t: AnswerType) -> &'static str {
    match t {
        AnswerType::Open => "open",
        AnswerType::Single => "single",
        AnswerType::Multiple => "multiple",
    }
}

/// The answer type named by `open`, `single` or `multiple`; `None` for any other word.
pub fn answer_type_named(word: &str) -> Option<AnswerType> {
    match word {
        "open" => Some(AnswerType::Open),
        "single" => Some(AnswerType::Single),
        "multiple" => Some(AnswerType::Multiple),
        _ => None,
    }
}

/// The answer type stored in a question's front matter (`answer_type`); absent means open text.
pub fn stated_type(value: Option<&str>) -> AnswerType {
    value.and_then(answer_type_named).unwrap_or_default()
}

/// The label of an answer type in the Create dialog and the editor.
pub fn answer_type_label(t: AnswerType) -> &'static str {
    match t {
        AnswerType::Open => "Open text",
        AnswerType::Single => "Single choice",
        AnswerType::Multiple => "Multiple select",
    }
}

/// The option lines a new choice item starts with: one correct for single choice, two correct for multiple select.
fn template_rows(t: AnswerType) -> Vec<Row> {
    let row = |correct| Row {
        text: String::new(),
        correct,
    };
    match t {
        AnswerType::Open => Vec::new(),
        AnswerType::Single => vec![row(false), row(true)],
        AnswerType::Multiple => vec![row(true), row(true)],
    }
}

/// One option as its Markdown line: `- [x] text` when correct, `- [ ] text` when wrong.
fn render_row(row: &Row) -> String {
    let mark = if row.correct { "x" } else { " " };
    format!("- [{mark}] {}", row.text)
}

/// The body for a prompt and its options: the prompt, a blank line, then one option per line.
pub fn render(prompt: &str, rows: &[Row]) -> String {
    let head = prompt.trim_end();
    let lines: Vec<String> = rows.iter().map(render_row).collect();
    match (head.is_empty(), lines.is_empty()) {
        (_, true) => head.to_owned(),
        (true, false) => lines.join("\n"),
        (false, false) => format!("{head}\n\n{}", lines.join("\n")),
    }
}

/// The body a new question starts with: the prompt, and for a choice type the option template. A prompt that
/// already holds options, or open text, is kept as it is.
pub fn create_body(prompt: &str, t: AnswerType) -> String {
    if t == AnswerType::Open || parse_choice(prompt).is_ok() {
        return prompt.to_owned();
    }
    render(prompt, &template_rows(t))
}

/// The choice in a body, when it has one.
pub fn block_of(body: &str) -> Option<ChoiceBlock> {
    parse_choice(body).ok()
}

/// The options of a parsed choice, in order.
pub fn rows_of(block: &ChoiceBlock) -> Vec<Row> {
    block
        .options
        .iter()
        .map(|o| Row {
            text: o.text.clone(),
            correct: o.correct,
        })
        .collect()
}

/// How many option lines a body has, read line by line (independent of the parser, for logs and checks).
pub fn option_count(body: &str) -> usize {
    body.lines()
        .filter(|l| {
            let l = l.trim_start();
            l.starts_with("- [ ]") || l.starts_with("- [x]") || l.starts_with("- [X]")
        })
        .count()
}

/// How many options are marked correct.
pub fn correct_count(rows: &[Row]) -> usize {
    rows.iter().filter(|r| r.correct).count()
}

/// Whether a list of options agrees with the answer type: open text has none, single choice exactly one
/// correct option, multiple select two or more. A list that does not agree is refused before saving.
pub fn agrees(t: AnswerType, rows: &[Row]) -> bool {
    let correct = correct_count(rows);
    match t {
        AnswerType::Open => rows.is_empty(),
        AnswerType::Single => correct == 1,
        AnswerType::Multiple => correct >= 2,
    }
}

/// Switching to open text removes the option markers, so it needs a confirmation first.
pub fn open_text_needs_confirm(body: &str) -> bool {
    parse_choice(body).is_ok()
}

/// The body of an open-text question: the prompt alone, with the option lines removed.
pub fn open_text_body(body: &str) -> String {
    match parse_choice(body).ok() {
        Some(block) => block.prompt.trim_end().to_owned(),
        None => body.to_owned(),
    }
}
