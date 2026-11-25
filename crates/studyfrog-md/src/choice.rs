//! Choice blocks in a card or question (STORY-0220): the prompt text, then a list of options. An
//! item `- [ ]` is a wrong option and `- [x]` a correct one. Pure: text in, plain data out.

use std::fmt;

/// One option of a choice block, with its text exactly as written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChoiceOption {
    /// The option text.
    pub text: String,
    /// True for `- [x]`, false for `- [ ]`.
    pub correct: bool,
}

/// A choice block: the prompt before the list, and the options in the order written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChoiceBlock {
    /// The prompt text before the first option.
    pub prompt: String,
    /// The options, in the order written.
    pub options: Vec<ChoiceOption>,
}

/// Why a body is not a usable choice block. Nothing is guessed: the caller decides what to show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChoiceError {
    /// The body has no `- [ ]` or `- [x]` item, so it is not a choice block.
    NoOptions,
    /// No option is marked `- [x]`, so the right answer is unknown.
    NoCorrect,
    /// There is no prompt text before the list.
    NoPrompt,
    /// An option has no text after its marker.
    EmptyOption,
    /// Non-blank text follows the list.
    TextAfterOptions,
}

impl fmt::Display for ChoiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::NoOptions => "the body has no `- [ ]` or `- [x]` option",
            Self::NoCorrect => "no option is marked `- [x]`",
            Self::NoPrompt => "there is no prompt text before the options",
            Self::EmptyOption => "an option has no text",
            Self::TextAfterOptions => "text follows the options",
        };
        f.write_str(text)
    }
}

impl std::error::Error for ChoiceError {}

/// Parse a card or question body into a choice block.
///
/// Lines before the first option form the prompt. Every option line must be `- [ ]` or `- [x]`
/// (`[X]` is accepted too), with leading indentation allowed. Blank lines are ignored.
pub fn parse_choice(body: &str) -> Result<ChoiceBlock, ChoiceError> {
    let mut prompt_lines: Vec<&str> = Vec::new();
    let mut options: Vec<ChoiceOption> = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(option) = option_line(trimmed) {
            options.push(option);
        } else if options.is_empty() {
            prompt_lines.push(line);
        } else if !trimmed.is_empty() {
            return Err(ChoiceError::TextAfterOptions);
        }
    }
    if options.is_empty() {
        return Err(ChoiceError::NoOptions);
    }
    if options.iter().any(|o| o.text.is_empty()) {
        return Err(ChoiceError::EmptyOption);
    }
    let prompt = prompt_lines.join("\n").trim().to_owned();
    if prompt.is_empty() {
        return Err(ChoiceError::NoPrompt);
    }
    if !options.iter().any(|o| o.correct) {
        return Err(ChoiceError::NoCorrect);
    }
    Ok(ChoiceBlock { prompt, options })
}

/// The option on a trimmed line, if the line is an option item.
fn option_line(trimmed: &str) -> Option<ChoiceOption> {
    let (correct, rest) = if let Some(rest) = trimmed.strip_prefix("- [ ]") {
        (false, rest)
    } else if let Some(rest) = trimmed.strip_prefix("- [x]") {
        (true, rest)
    } else if let Some(rest) = trimmed.strip_prefix("- [X]") {
        (true, rest)
    } else {
        return None;
    };
    Some(ChoiceOption {
        text: rest.trim().to_owned(),
        correct,
    })
}

/// How many options are correct.
pub fn correct_count(block: &ChoiceBlock) -> usize {
    block.options.iter().filter(|o| o.correct).count()
}

/// The positions (0-based) of the correct options, in order.
pub fn correct_indices(block: &ChoiceBlock) -> Vec<usize> {
    block
        .options
        .iter()
        .enumerate()
        .filter(|(_, o)| o.correct)
        .map(|(i, _)| i)
        .collect()
}

/// Multiple select when two or more options are correct; single choice otherwise.
pub fn is_multiple(block: &ChoiceBlock) -> bool {
    correct_count(block) >= 2
}
