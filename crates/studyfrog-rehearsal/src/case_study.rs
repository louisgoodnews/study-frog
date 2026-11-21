//! Scenario-based case studies (STORY-0179): a vignette, a decision, the options, the reference
//! choice, and the rubric the justification is judged on. Pure validation.

/// Shortest vignette, in characters.
pub const MIN_VIGNETTE_CHARS: usize = 40;
/// Longest vignette, in characters.
pub const MAX_VIGNETTE_CHARS: usize = 4_000;
/// Longest decision prompt, in characters.
pub const MAX_DECISION_CHARS: usize = 500;
/// Longest rubric criterion, in characters.
pub const MAX_CRITERION_CHARS: usize = 200;
/// Most rubric criteria per case.
pub const MAX_RUBRIC_CRITERIA: usize = 12;

/// One case-study item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseStudyItem {
    /// The scenario: setting, target construct, constraints.
    pub vignette: String,
    /// The decision the student must make.
    pub decision_prompt: String,
    /// The choices on offer (tests or procedures).
    pub options: Vec<String>,
    /// The reference choice or accepted procedures; each one is one of `options`.
    pub expected_choices: Vec<String>,
    /// Points the justification must make, for example "addresses fabbing risk".
    pub rubric_criteria: Vec<String>,
}

fn norm(text: &str) -> String {
    text.trim().to_lowercase()
}

fn has_duplicates(items: &[String]) -> bool {
    let mut seen: Vec<String> = items.iter().map(|s| norm(s)).collect();
    seen.sort();
    seen.windows(2).any(|w| w[0] == w[1])
}

/// Check a case study before it is stored or run. The error names the first problem found.
pub fn validate_case_study(c: &CaseStudyItem) -> Result<(), String> {
    let vignette_len = c.vignette.trim().chars().count();
    if !(MIN_VIGNETTE_CHARS..=MAX_VIGNETTE_CHARS).contains(&vignette_len) {
        return Err(format!(
            "the vignette must be between {MIN_VIGNETTE_CHARS} and {MAX_VIGNETTE_CHARS} characters"
        ));
    }
    let prompt = c.decision_prompt.trim();
    if prompt.is_empty() || prompt.chars().count() > MAX_DECISION_CHARS {
        return Err(format!(
            "the decision prompt must be 1 to {MAX_DECISION_CHARS} characters"
        ));
    }
    if c.options.len() < 2 || c.options.iter().any(|o| o.trim().is_empty()) {
        return Err("a case needs at least two options, all non-empty".to_owned());
    }
    if has_duplicates(&c.options) {
        return Err("the options must be distinct".to_owned());
    }
    if c.expected_choices.is_empty() {
        return Err("a case needs at least one expected choice".to_owned());
    }
    let known: Vec<String> = c.options.iter().map(|o| norm(o)).collect();
    if c.expected_choices.iter().any(|e| !known.contains(&norm(e))) {
        return Err("every expected choice must be one of the options".to_owned());
    }
    if c.rubric_criteria.is_empty() || c.rubric_criteria.len() > MAX_RUBRIC_CRITERIA {
        return Err(format!(
            "a rubric has between 1 and {MAX_RUBRIC_CRITERIA} criteria"
        ));
    }
    if c.rubric_criteria
        .iter()
        .any(|r| r.trim().is_empty() || r.chars().count() > MAX_CRITERION_CHARS)
    {
        return Err(format!(
            "each criterion must be 1 to {MAX_CRITERION_CHARS} characters"
        ));
    }
    if has_duplicates(&c.rubric_criteria) {
        return Err("the rubric criteria must be distinct".to_owned());
    }
    Ok(())
}

/// Whether the student's picks are exactly the expected choices (order and case do not matter).
pub fn choice_is_correct(c: &CaseStudyItem, picked: &[String]) -> bool {
    let mut want: Vec<String> = c.expected_choices.iter().map(|s| norm(s)).collect();
    let mut got: Vec<String> = picked.iter().map(|s| norm(s)).collect();
    want.sort();
    want.dedup();
    got.sort();
    got.dedup();
    want == got
}
