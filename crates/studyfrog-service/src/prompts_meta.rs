//! Prompt categories and parameters (STORY-0092): how the library groups the standard prompts, and
//! the choices (such as the depth of an explanation) that are put into a prompt's text.

/// The group a prompt is listed under.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptCategory {
    /// Flashcards, questions, a whole stack, notes.
    ContentGeneration,
    /// A dialogue with a tutor or an examiner.
    InteractiveTutoring,
    /// Explanations, concept maps, summaries, critiques.
    AnalysisAndInsights,
}

/// Every category, in the order of the tabs.
pub const CATEGORIES: [PromptCategory; 3] = [
    PromptCategory::ContentGeneration,
    PromptCategory::InteractiveTutoring,
    PromptCategory::AnalysisAndInsights,
];

/// The name shown on the category's tab.
pub fn category_name(category: PromptCategory) -> &'static str {
    match category {
        PromptCategory::ContentGeneration => "Create content",
        PromptCategory::InteractiveTutoring => "Tutoring",
        PromptCategory::AnalysisAndInsights => "Analysis",
    }
}

/// The category of the prompt `id`.
pub fn category_of(id: &str) -> PromptCategory {
    match id {
        "exam" | "socratic" => PromptCategory::InteractiveTutoring,
        "explain" | "concept_map" | "summary" | "critique" => PromptCategory::AnalysisAndInsights,
        _ => PromptCategory::ContentGeneration,
    }
}

/// A choice in a prompt: `{{key}}` in the text becomes the text of the chosen option.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PromptParam {
    /// The name in the prompt file.
    pub key: &'static str,
    /// What the selector is called.
    pub label: &'static str,
    /// The options as (name on the selector, text put into the prompt); the first is the default.
    pub options: &'static [(&'static str, &'static str)],
}

const EXPLAIN: &[PromptParam] = &[PromptParam {
    key: "depth",
    label: "Depth",
    options: &[
        ("All three levels", "all three levels"),
        (
            "Intuitive (ELI5)",
            "the intuitive level only (write only that one note)",
        ),
        (
            "Core concepts",
            "the core concepts level only (write only that one note)",
        ),
        (
            "Technical",
            "the technical level only (write only that one note)",
        ),
    ],
}];

const EXAM: &[PromptParam] = &[PromptParam {
    key: "count",
    label: "Questions",
    options: &[("3 questions", "3"), ("5 questions", "5")],
}];

/// The choices the prompt `id` offers; most prompts offer none.
pub fn params_of(id: &str) -> &'static [PromptParam] {
    match id {
        "explain" => EXPLAIN,
        "exam" => EXAM,
        _ => &[],
    }
}

/// The text put into the prompt for `param`: the picked option, else the first. `picks` holds
/// (key, option index).
fn chosen_text(param: &PromptParam, picks: &[(&str, usize)]) -> &'static str {
    let at = picks
        .iter()
        .find(|(key, _)| *key == param.key)
        .map_or(0, |(_, at)| *at);
    param
        .options
        .get(at)
        .or(param.options.first())
        .map_or("", |(_, text)| text)
}

/// `text` of the prompt `id` with every `{{key}}` of its parameters replaced.
pub fn fill_params(id: &str, text: &str, picks: &[(&str, usize)]) -> String {
    params_of(id).iter().fold(text.to_owned(), |text, param| {
        text.replace(&format!("{{{{{}}}}}", param.key), chosen_text(param, picks))
    })
}
