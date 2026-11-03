//! The standard prompts (TASK-0205): ready-made instructions for an AI that make it produce a file StudyFrog imports.
//! The texts are `docs/prompts/*.md`, compiled in; the shared parts (`_layout`, `_files`, `_answer`) are put in
//! where a prompt says `{{layout}}`, `{{files}}` or `{{answer}}`. Every worked example is read by the importer in
//! the tests, so a prompt cannot drift from what the app reads.

/// A prompt as the file has it.
#[derive(Debug, Clone, Copy)]
pub struct PromptSource {
    /// The id, which is the file name without `.md`.
    pub id: &'static str,
    /// The file's text.
    pub text: &'static str,
}

macro_rules! prompt {
    ($id:literal) => {
        PromptSource {
            id: $id,
            text: include_str!(concat!("../../../docs/prompts/", $id, ".md")),
        }
    };
}

/// Every standard prompt, in the order of the list.
pub const PROMPTS: &[PromptSource] = &[
    prompt!("flashcards"),
    prompt!("questions"),
    prompt!("stack"),
    prompt!("notes"),
    prompt!("summary"),
    prompt!("critique"),
];

const LAYOUT: &str = include_str!("../../../docs/prompts/_layout.md");
const FILES: &str = include_str!("../../../docs/prompts/_files.md");
const ANSWER: &str = include_str!("../../../docs/prompts/_answer.md");

/// A prompt ready to copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prompt {
    /// The id.
    pub id: &'static str,
    /// The title.
    pub title: String,
    /// One sentence about what it does.
    pub summary: String,
    /// The whole text to give to an AI.
    pub text: String,
}

/// The title: the first line without its `# `.
fn title_of(text: &str) -> String {
    let first = text.lines().next().unwrap_or_default();
    first.trim_start_matches('#').trim().to_owned()
}

/// The text after the title and the summary line, shared parts put in.
fn expand(text: &str) -> String {
    let body = text.splitn(3, '\n').nth(2).unwrap_or_default();
    body.trim_start()
        .replace("{{layout}}", LAYOUT.trim_end())
        .replace("{{files}}", FILES.trim_end())
        .replace("{{answer}}", ANSWER.trim_end())
}

/// The summary: the line under the title.
fn summary_of(text: &str) -> String {
    text.lines()
        .skip(1)
        .find(|l| !l.trim().is_empty())
        .unwrap_or_default()
        .trim()
        .to_owned()
}

/// The prompt for a source.
pub fn prompt_of(source: &PromptSource) -> Prompt {
    Prompt {
        id: source.id,
        title: title_of(source.text),
        summary: summary_of(source.text),
        text: expand(source.text),
    }
}

/// All the prompts, ready to copy.
pub fn prompts() -> Vec<Prompt> {
    PROMPTS.iter().map(prompt_of).collect()
}

/// The first worked example (the first csv code block) of a prompt text.
pub fn example_of(text: &str) -> Option<String> {
    let start = text.find("```csv\n")? + "```csv\n".len();
    let end = text[start..].find("```")? + start;
    Some(text[start..end].to_owned())
}

/// The file name a prompt is saved under.
pub fn file_name(p: &Prompt) -> String {
    format!("studyfrog-prompt-{}.md", p.id)
}

/// The Markdown file for a prompt: a heading, then the text to give to the AI.
pub fn file_text(p: &Prompt) -> String {
    format!("# {}\n\n{}\n\n{}\n", p.title, p.summary, p.text)
}
