//! Format mix validation and proportional count resolution for mock exams (STORY-0207).

use crate::exam_format::{ALL_EXAM_FORMATS, ExamItemFormat, format_index};

/// Maximum number of questions in one generated mock exam.
pub const MAX_EXAM_QUESTIONS: usize = 50;

/// Default total question count for a generated mock exam.
pub const DEFAULT_EXAM_QUESTIONS: usize = 10;

/// How the user specifies the item format mix in the modal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MixMode {
    /// Proportional weights scaled to `total_questions`.
    #[default]
    Weights,
    /// Exact question counts per enabled format.
    Counts,
}

/// Per-format enable toggle and count or weight value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatAllocation {
    /// The item format this row configures.
    pub format: ExamItemFormat,
    /// Whether this format is included in the generated exam.
    pub enabled: bool,
    /// Exact count (in [`MixMode::Counts`]) or relative weight (in [`MixMode::Weights`]).
    pub value: usize,
}

/// User-configured item format mix for mock exam generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExamFormatMix {
    /// Whether `value` means exact counts or proportional weights.
    pub mode: MixMode,
    /// Target total questions when `mode == MixMode::Weights`.
    pub total_questions: usize,
    /// Per-format allocations in [`ALL_EXAM_FORMATS`] order.
    pub formats: [FormatAllocation; 6],
}

impl Default for ExamFormatMix {
    fn default() -> Self {
        default_format_mix()
    }
}

/// Resolved exact question counts per format, indexed by [`format_index`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FormatPlan {
    /// Counts in [`ALL_EXAM_FORMATS`] order.
    pub counts: [usize; 6],
}

/// Sensible default format mix: all 6 formats enabled, 10 questions total.
pub fn default_format_mix() -> ExamFormatMix {
    let weights = [3, 1, 1, 1, 2, 2];
    let formats = [0, 1, 2, 3, 4, 5].map(|i| FormatAllocation {
        format: ALL_EXAM_FORMATS[i],
        enabled: true,
        value: weights[i],
    });
    ExamFormatMix {
        mode: MixMode::Weights,
        total_questions: DEFAULT_EXAM_QUESTIONS,
        formats,
    }
}

/// Read the planned count for `format` from `plan`.
pub fn plan_count(plan: &FormatPlan, format: ExamItemFormat) -> usize {
    plan.counts[format_index(format)]
}

/// Set the planned count for `format` inside `plan`.
pub fn set_plan_count(plan: &mut FormatPlan, format: ExamItemFormat, count: usize) {
    plan.counts[format_index(format)] = count;
}

/// Sum of all format counts in `plan`.
pub fn plan_total(plan: &FormatPlan) -> usize {
    plan.counts.iter().sum()
}

/// Total questions implied by `mix` (sum of enabled counts in Counts mode, or `total_questions` in Weights mode).
pub fn resolved_mix_total(mix: &ExamFormatMix) -> usize {
    match mix.mode {
        MixMode::Counts => mix
            .formats
            .iter()
            .filter(|a| a.enabled)
            .map(|a| a.value)
            .sum(),
        MixMode::Weights => mix.total_questions,
    }
}

/// Legacy 40/30/30 plan across OpenQuestion, CaseScenario, and Calculation.
pub fn legacy_plan(total: usize) -> FormatPlan {
    let mut recall = (total as f64 * 0.4).round() as usize;
    let mut case = (total as f64 * 0.3).round() as usize;
    let mut calc = total.saturating_sub(recall + case);
    if total >= 3 {
        for slot in [&mut recall, &mut case, &mut calc] {
            if *slot == 0 {
                *slot = 1;
            }
        }
        let over = (recall + case + calc).saturating_sub(total);
        recall = recall.saturating_sub(over);
    }
    let mut out = FormatPlan::default();
    set_plan_count(&mut out, ExamItemFormat::OpenQuestion, recall);
    set_plan_count(&mut out, ExamItemFormat::CaseScenario, case);
    set_plan_count(&mut out, ExamItemFormat::Calculation, calc);
    out
}

/// Validate a user-configured [`ExamFormatMix`].
pub fn validate_format_mix(mix: &ExamFormatMix) -> Result<(), String> {
    if !mix.formats.iter().any(|a| a.enabled) {
        return Err("Select at least one item format.".to_owned());
    }
    match mix.mode {
        MixMode::Counts => {
            let sum: usize = mix
                .formats
                .iter()
                .filter(|a| a.enabled)
                .map(|a| a.value)
                .sum();
            if sum == 0 {
                return Err("Set at least 1 question across the selected formats.".to_owned());
            }
            if sum > MAX_EXAM_QUESTIONS {
                return Err(format!(
                    "Total questions must be at most {MAX_EXAM_QUESTIONS}."
                ));
            }
        }
        MixMode::Weights => {
            if mix.total_questions == 0 || mix.total_questions > MAX_EXAM_QUESTIONS {
                return Err(format!(
                    "Total questions must be between 1 and {MAX_EXAM_QUESTIONS}."
                ));
            }
            if mix.formats.iter().any(|a| a.enabled && a.value == 0) {
                return Err("Every enabled format needs a weight of at least 1.".to_owned());
            }
            let wsum: usize = mix
                .formats
                .iter()
                .filter(|a| a.enabled)
                .map(|a| a.value)
                .sum();
            if wsum == 0 {
                return Err("Set a positive weight for at least one format.".to_owned());
            }
        }
    }
    Ok(())
}

/// Resolve `mix` into exact per-format counts using Hamilton's largest-remainder method for weights.
pub fn resolve_format_plan(mix: &ExamFormatMix) -> Result<FormatPlan, String> {
    validate_format_mix(mix)?;
    let mut plan = FormatPlan::default();
    if mix.mode == MixMode::Counts {
        for (i, a) in mix.formats.iter().enumerate() {
            plan.counts[i] = if a.enabled { a.value } else { 0 };
        }
        return Ok(plan);
    }
    let total = mix.total_questions;
    let enabled: Vec<(usize, usize)> = mix
        .formats
        .iter()
        .enumerate()
        .filter(|(_, a)| a.enabled)
        .map(|(i, a)| (i, a.value))
        .collect();
    let wsum: usize = enabled.iter().map(|(_, w)| *w).sum();
    let mut rems: Vec<(usize, usize)> = Vec::with_capacity(enabled.len());
    for &(i, w) in &enabled {
        let num = w * total;
        plan.counts[i] = num / wsum;
        rems.push((i, num % wsum));
    }
    rems.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let leftover = total.saturating_sub(plan_total(&plan));
    for (i, _) in rems.into_iter().take(leftover) {
        plan.counts[i] += 1;
    }
    if total >= enabled.len() {
        ensure_each_enabled_has_one(&mut plan, &enabled);
    }
    Ok(plan)
}

fn ensure_each_enabled_has_one(plan: &mut FormatPlan, enabled: &[(usize, usize)]) {
    for &(i, _) in enabled {
        if plan.counts[i] == 0
            && let Some(&(donor, _)) = enabled.iter().max_by_key(|&&(j, _)| plan.counts[j])
            && plan.counts[donor] > 1
        {
            plan.counts[donor] -= 1;
            plan.counts[i] = 1;
        }
    }
}
