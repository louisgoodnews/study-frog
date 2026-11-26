//! Personalized item-complexity baseline and WIP draft scorer (EPIC-077, STORY-0252).
//!
//! Complies with Invariant P1: zero inherent impl blocks; all logic in pure free functions.

/// Minimum number of reviewed items before a personalized complexity baseline is active.
pub const MIN_REVIEWED_SAMPLES: usize = 8;

/// Retention summary for one existing vault item.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemRetentionSample {
    /// Word count of the item's prompt/answer/body.
    pub word_count: usize,
    /// Number of bullet or numbered list lines.
    pub list_items: usize,
    /// Current familiarity on `0.0..=1.0`.
    pub familiarity: f32,
    /// Total reviews recorded for this item.
    pub reviews: u32,
}

/// Personalized retained-complexity baseline derived from the user's vault history.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionComplexityProfile {
    /// Number of well-retained items used to compute the baseline.
    pub sample_count: usize,
    /// Mean word count of well-retained items.
    pub retained_avg_words: f32,
    /// Complexity ceiling above which WIP drafts are flagged.
    pub retained_ceiling_words: f32,
}

/// Complexity metrics of a work-in-progress draft.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DraftComplexity {
    /// Total words across prompt/front and answer/back.
    pub word_count: usize,
    /// Number of bullet or numbered list lines.
    pub list_items: usize,
}

/// Warning produced when a WIP draft significantly exceeds the user's retained complexity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComplexityWarning {
    /// Word count of the WIP draft.
    pub draft_words: usize,
    /// User's average word count on well-retained items.
    pub retained_avg_words: usize,
    /// Human-readable explanation for the toast/banner.
    pub reason: String,
}

/// Score a raw Markdown/text body (or prompt + answer pair) for word and list density.
pub fn score_text_complexity(text: &str, answer: &str) -> DraftComplexity {
    let mut word_count = 0usize;
    let mut list_items = 0usize;
    for part in [text, answer] {
        word_count += part.split_whitespace().count();
        for line in part.lines() {
            let trimmed = line.trim_start();
            let is_bullet = trimmed.starts_with("- ") || trimmed.starts_with("* ");
            let is_numbered = trimmed
                .split_once(". ")
                .is_some_and(|(num, _)| !num.is_empty() && num.chars().all(|c| c.is_ascii_digit()));
            if is_bullet || is_numbered {
                list_items += 1;
            }
        }
    }
    DraftComplexity {
        word_count,
        list_items,
    }
}

/// Estimate the user's retained-complexity baseline from their existing reviewed vault items.
/// Returns `None` when fewer than [`MIN_REVIEWED_SAMPLES`] reviewed items exist.
pub fn estimate_complexity_baseline(
    samples: &[ItemRetentionSample],
) -> Option<RetentionComplexityProfile> {
    let reviewed: Vec<&ItemRetentionSample> = samples.iter().filter(|s| s.reviews > 0).collect();
    if reviewed.len() < MIN_REVIEWED_SAMPLES {
        return None;
    }

    let mut well_retained: Vec<f32> = reviewed
        .iter()
        .filter(|s| s.familiarity >= 0.60)
        .map(|s| (s.word_count + s.list_items * 4) as f32)
        .collect();
    if well_retained.len() < 4 {
        well_retained = reviewed
            .iter()
            .map(|s| (s.word_count + s.list_items * 4) as f32)
            .collect();
    }
    well_retained.sort_by(f32::total_cmp);

    let count = well_retained.len();
    let avg = well_retained.iter().sum::<f32>() / count as f32;
    let p80_idx = ((count as f32 * 0.80).floor() as usize).min(count - 1);
    let p80 = well_retained[p80_idx];
    let ceiling = (avg * 1.6).max(p80 * 1.25).max(24.0);

    Some(RetentionComplexityProfile {
        sample_count: count,
        retained_avg_words: avg,
        retained_ceiling_words: ceiling,
    })
}

/// Evaluate a WIP new item against `profile`, returning a warning if it is much more complex
/// than what the user typically retains.
pub fn check_wip_complexity(
    profile: &RetentionComplexityProfile,
    text: &str,
    answer: &str,
) -> Option<ComplexityWarning> {
    let draft = score_text_complexity(text, answer);
    if draft.word_count < 24 {
        return None;
    }
    let effective = (draft.word_count + draft.list_items * 4) as f32;
    if effective <= profile.retained_ceiling_words {
        return None;
    }
    let avg_rounded = profile.retained_avg_words.round().max(1.0) as usize;
    Some(ComplexityWarning {
        draft_words: draft.word_count,
        retained_avg_words: avg_rounded,
        reason: format!(
            "This draft is ~{} words; items you retain well average ~{} words. Consider splitting it into smaller cards or questions.",
            draft.word_count, avg_rounded
        ),
    })
}
