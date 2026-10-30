//! Scoring an answer against a reference: pure functions over embeddings (see the scoring spec).

use crate::runtime::{SlmError, SlmRuntime};
use crate::text::normalize_text;
use crate::vector::cosine;

/// The overall judgement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// At or above the item's minimum score.
    Correct,
    /// Within 0.2 below the minimum score.
    Partial,
    /// Anything lower.
    Incorrect,
}

/// Persisted name of a verdict.
pub fn verdict_name(v: Verdict) -> &'static str {
    match v {
        Verdict::Correct => "correct",
        Verdict::Partial => "partial",
        Verdict::Incorrect => "incorrect",
    }
}

/// How far below the minimum score still counts as partly right.
pub const PARTIAL_MARGIN: f32 = 0.2;
/// Cosine at which a rubric point counts as covered.
pub const POINT_THRESHOLD: f32 = 0.5;
/// Weight of the reference similarity when a rubric is present.
pub const SIMILARITY_WEIGHT: f32 = 0.6;

/// What is compared.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Comparison<'a> {
    /// The reference answer.
    pub reference: &'a str,
    /// What the user typed.
    pub answer: &'a str,
    /// Other acceptable answers.
    pub accepted: &'a [String],
    /// Key points the answer should contain.
    pub rubric: &'a [String],
    /// Score at which the answer counts as correct.
    pub min_score: f32,
}

/// The result of a comparison.
#[derive(Debug, Clone, PartialEq)]
pub struct Score {
    /// 0 to 1.
    pub value: f32,
    /// The judgement.
    pub verdict: Verdict,
    /// Per rubric point: was it covered?
    pub rubric_hits: Vec<bool>,
}

/// The verdict for a score and a minimum.
pub fn verdict_for(value: f32, min_score: f32) -> Verdict {
    if value >= min_score {
        Verdict::Correct
    } else if value >= min_score - PARTIAL_MARGIN {
        Verdict::Partial
    } else {
        Verdict::Incorrect
    }
}

/// Stretch a cosine similarity so that `pivot` becomes 0.7, 0 stays 0 and 1 stays 1 (two straight
/// pieces). Monotone, so the order of answers never changes.
pub fn calibrate(cosine: f32, pivot: f32) -> f32 {
    let (c, p) = (cosine.clamp(0.0, 1.0), pivot.clamp(0.05, 0.95));
    if c <= p {
        c / p * 0.7
    } else {
        0.7 + (c - p) / (1.0 - p) * 0.3
    }
}

/// Combine embeddings into a score. `candidates` are the reference and the accepted answers,
/// `points` the rubric; each slice holds vectors already computed for those texts.
pub fn score_from_embeddings(
    answer: &[f32],
    candidates: &[Vec<f32>],
    points: &[Vec<f32>],
    (min_score, pivot): (f32, f32),
) -> Score {
    let similarity = |other: &Vec<f32>| calibrate(cosine(answer, other), pivot);
    let best = candidates
        .iter()
        .map(similarity)
        .fold(0.0_f32, f32::max)
        .clamp(0.0, 1.0);
    let rubric_hits: Vec<bool> = points
        .iter()
        .map(|p| similarity(p) >= POINT_THRESHOLD)
        .collect();
    let value = if rubric_hits.is_empty() {
        best
    } else {
        let ratio = rubric_hits.iter().filter(|h| **h).count() as f32 / rubric_hits.len() as f32;
        SIMILARITY_WEIGHT * best + (1.0 - SIMILARITY_WEIGHT) * ratio
    };
    let value = value.clamp(0.0, 1.0);
    Score {
        value,
        verdict: verdict_for(value, min_score),
        rubric_hits,
    }
}

/// Embed everything with `runtime` and score the answer. An empty answer scores 0 without
/// calling the runtime.
pub fn score_answer<R: SlmRuntime + ?Sized>(
    runtime: &R,
    c: &Comparison<'_>,
) -> Result<Score, SlmError> {
    let answer = normalize_text(c.answer);
    if answer.is_empty() {
        let rubric_hits = vec![false; c.rubric.len()];
        return Ok(Score {
            value: 0.0,
            verdict: verdict_for(0.0, c.min_score),
            rubric_hits,
        });
    }
    let texts: Vec<String> = std::iter::once(answer)
        .chain(std::iter::once(normalize_text(c.reference)))
        .chain(c.accepted.iter().map(|t| normalize_text(t)))
        .chain(c.rubric.iter().map(|t| normalize_text(t)))
        .collect();
    let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
    let mut vectors = runtime.embed(&refs)?;
    if vectors.len() != texts.len() {
        return Err(SlmError::Failed(format!(
            "expected {} embeddings, got {}",
            texts.len(),
            vectors.len()
        )));
    }
    let points = vectors.split_off(2 + c.accepted.len());
    let answer_vec = vectors.remove(0);
    Ok(score_from_embeddings(
        &answer_vec,
        &vectors,
        &points,
        (c.min_score, runtime.pivot()),
    ))
}
