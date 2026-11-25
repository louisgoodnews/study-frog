//! Checks for choice questions (EPIC-072) when they are created: the answer type stated in the front matter
//! must agree with the option lines of the body, and a choice question needs no reference answer.

use crate::create::invalid;
use crate::create_more::{NewItem, NewKind};
use crate::session::ServiceError;

/// A choice question (single or multiple) gets its answers from its option lines: no reference answer.
pub(crate) fn choice_question(item: &NewItem<'_>) -> bool {
    item.kind == NewKind::Question && matches!(item.answer_type, Some("single" | "multiple"))
}

/// The marks of the option lines (`- [ ]` wrong, `- [x]` correct), in order.
fn option_marks(text: &str) -> Vec<bool> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim_start();
            if line.starts_with("- [ ]") {
                Some(false)
            } else if line.starts_with("- [x]") || line.starts_with("- [X]") {
                Some(true)
            } else {
                None
            }
        })
        .collect()
}

/// Refuses a question whose option lines disagree with its stated answer type: open text has no
/// options, single choice exactly one correct option, multiple select two or more.
pub(crate) fn answer_type_agrees(item: &NewItem<'_>) -> Result<(), ServiceError> {
    let Some(stated) = item.answer_type else {
        return Ok(());
    };
    if item.kind != NewKind::Question {
        return Ok(());
    }
    let marks = option_marks(item.text);
    let correct = marks.iter().filter(|c| **c).count();
    let agrees = match stated {
        "open" => marks.is_empty(),
        "single" => correct == 1,
        "multiple" => correct >= 2,
        _ => return Err(invalid("the answer type must be open, single or multiple")),
    };
    if agrees {
        Ok(())
    } else {
        Err(invalid(&format!(
            "the body has {} option(s) with {correct} correct, which does not match the answer type {stated}",
            marks.len()
        )))
    }
}
