//! Semantic distractor extraction and multiple-choice question building.
//!
//! Complies with Invariant P1: zero inherent impl blocks; domain logic is
//! implemented entirely in pure free functions.

/// A candidate answer pool item for distractor generation.
#[derive(Debug, Clone, PartialEq)]
pub struct DistractorCandidate {
    /// The candidate text / answer.
    pub text: String,
    /// The dense vector embedding of `text`.
    pub embedding: Vec<f32>,
}

/// A multiple-choice question with a prompt, correct answer, distractors, and shuffled choices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultipleChoiceQuestion {
    /// What the question asks.
    pub prompt: String,
    /// The canonical correct answer.
    pub correct_answer: String,
    /// The chosen distractors (without the correct answer).
    pub distractors: Vec<String>,
    /// All choices presented to the learner (distractors + correct answer), shuffled.
    pub all_choices: Vec<String>,
    /// The 0-based index of `correct_answer` within `all_choices`.
    pub correct_index: usize,
}

/// Cosine similarity between two vector embeddings in `[-1.0, 1.0]`.
///
/// Returns 0.0 when either slice is empty, lengths differ, or a norm is zero / non-finite.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let (mut dot, mut norm_a, mut norm_b) = (0.0_f32, 0.0_f32, 0.0_f32);
    for (x, y) in a.iter().zip(b) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }
    let denom = (norm_a * norm_b).sqrt();
    if denom > 0.0 && denom.is_finite() {
        (dot / denom).clamp(-1.0, 1.0)
    } else {
        0.0
    }
}

/// Select top `count` candidates with highest cosine similarity to `target_embedding`,
/// excluding candidates that match any string in `exclude_texts` (case-insensitive or identical text)
/// and deduplicating candidates amongst each other (case-insensitive).
pub fn select_semantic_distractors_excluding(
    target_embedding: &[f32],
    candidates: &[DistractorCandidate],
    count: usize,
    exclude_texts: &[&str],
) -> Vec<String> {
    if count == 0 || candidates.is_empty() {
        return Vec::new();
    }

    let mut scored: Vec<(&DistractorCandidate, f32)> = candidates
        .iter()
        .filter(|c| !c.text.trim().is_empty())
        .filter(|c| {
            let trimmed = c.text.trim();
            !exclude_texts
                .iter()
                .any(|ex| trimmed.eq_ignore_ascii_case(ex.trim()))
        })
        .map(|c| (c, cosine_similarity(target_embedding, &c.embedding)))
        .collect();

    // Sort descending by similarity, tie-breaking deterministically by text
    scored.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.text.cmp(&b.0.text))
    });

    let mut selected = Vec::with_capacity(count);
    let mut seen_normalized = Vec::with_capacity(count);

    for (candidate, _) in scored {
        let norm = candidate.text.trim().to_ascii_lowercase();
        if !seen_normalized.contains(&norm) {
            seen_normalized.push(norm);
            selected.push(candidate.text.clone());
            if selected.len() == count {
                break;
            }
        }
    }

    selected
}

/// Select top `count` candidates with highest cosine similarity to target,
/// excluding exact matches (case-insensitive or identical text).
pub fn select_semantic_distractors(
    target_embedding: &[f32],
    candidates: &[DistractorCandidate],
    count: usize,
) -> Vec<String> {
    select_semantic_distractors_excluding(target_embedding, candidates, count, &[])
}

/// Build a multiple choice question with 1 correct answer and `choice_count - 1` distractors,
/// shuffled using a deterministic PRNG from `seed`.
pub fn build_multiple_choice_question(
    prompt: &str,
    correct_answer: &str,
    target_embedding: &[f32],
    candidates: &[DistractorCandidate],
    choice_count: usize,
    seed: u64,
) -> MultipleChoiceQuestion {
    let distractor_count = choice_count.saturating_sub(1);
    let distractors = select_semantic_distractors_excluding(
        target_embedding,
        candidates,
        distractor_count,
        &[correct_answer],
    );

    let mut all_choices = Vec::with_capacity(distractors.len() + 1);
    all_choices.push(correct_answer.to_string());
    all_choices.extend(distractors.clone());

    crate::queue::shuffle::shuffle(&mut all_choices, seed);

    let correct_index = all_choices
        .iter()
        .position(|c| c == correct_answer)
        .unwrap_or(0);

    MultipleChoiceQuestion {
        prompt: prompt.to_string(),
        correct_answer: correct_answer.to_string(),
        distractors,
        all_choices,
        correct_index,
    }
}
