//! Item format definitions, labels, and section headings for mock exams (STORY-0207).

pub use crate::exam_format_mix::{
    DEFAULT_EXAM_QUESTIONS, ExamFormatMix, FormatAllocation, FormatPlan, MAX_EXAM_QUESTIONS,
    MixMode, default_format_mix, legacy_plan, plan_count, plan_total, resolve_format_plan,
    resolved_mix_total, set_plan_count, validate_format_mix,
};

/// Supported mock-exam question item formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExamItemFormat {
    /// Free-text conceptual recall question.
    OpenQuestion,
    /// Cloze / fill-in-the-gaps prompt with inline blanks.
    FillInGaps,
    /// Single-choice question with radio options and one correct answer.
    SingleSelect,
    /// Multi-choice question with checkboxes and one or more correct answers.
    MultipleSelect,
    /// Quantitative / formula calculation problem.
    Calculation,
    /// Applied methodological case scenario.
    CaseScenario,
}

/// All six item formats in the configuration modal's display order.
pub const ALL_EXAM_FORMATS: [ExamItemFormat; 6] = [
    ExamItemFormat::OpenQuestion,
    ExamItemFormat::FillInGaps,
    ExamItemFormat::SingleSelect,
    ExamItemFormat::MultipleSelect,
    ExamItemFormat::Calculation,
    ExamItemFormat::CaseScenario,
];

/// Array index (0..6) of `format` in [`ALL_EXAM_FORMATS`].
pub fn format_index(format: ExamItemFormat) -> usize {
    match format {
        ExamItemFormat::OpenQuestion => 0,
        ExamItemFormat::FillInGaps => 1,
        ExamItemFormat::SingleSelect => 2,
        ExamItemFormat::MultipleSelect => 3,
        ExamItemFormat::Calculation => 4,
        ExamItemFormat::CaseScenario => 5,
    }
}

/// Human-readable label shown in the UI and Markdown export.
pub fn format_label(format: ExamItemFormat) -> &'static str {
    match format {
        ExamItemFormat::OpenQuestion => "Open question",
        ExamItemFormat::FillInGaps => "Fill in the gaps",
        ExamItemFormat::SingleSelect => "Single select",
        ExamItemFormat::MultipleSelect => "Multiple select",
        ExamItemFormat::Calculation => "Calculation",
        ExamItemFormat::CaseScenario => "Case scenario",
    }
}

/// Canonical JSON/wire identifier used in SLM prompts and responses.
pub fn format_wire_name(format: ExamItemFormat) -> &'static str {
    match format {
        ExamItemFormat::OpenQuestion => "open_question",
        ExamItemFormat::FillInGaps => "fill_in_gaps",
        ExamItemFormat::SingleSelect => "single_select",
        ExamItemFormat::MultipleSelect => "multiple_select",
        ExamItemFormat::Calculation => "calculation",
        ExamItemFormat::CaseScenario => "case_scenario",
    }
}

/// Parse a format wire name or legacy alias from an SLM JSON response.
pub fn parse_format_wire_name(raw: &str) -> Option<ExamItemFormat> {
    let normalized = raw.trim().to_ascii_lowercase().replace(['-', ' '], "_");
    match normalized.as_str() {
        "open_question" | "open" | "recall" | "conceptual_recall" => {
            Some(ExamItemFormat::OpenQuestion)
        }
        "fill_in_gaps" | "fill_in_the_gaps" | "fill_in_blank" | "cloze" | "gaps" => {
            Some(ExamItemFormat::FillInGaps)
        }
        "single_select" | "single_choice" | "single" | "mcq" => Some(ExamItemFormat::SingleSelect),
        "multiple_select" | "multi_select" | "multiple_choice" | "multi_choice" | "multiple" => {
            Some(ExamItemFormat::MultipleSelect)
        }
        "calculation" | "calc" => Some(ExamItemFormat::Calculation),
        "case_scenario" | "case" => Some(ExamItemFormat::CaseScenario),
        _ => None,
    }
}

/// Exam section heading for `format` (`Part A`..`Part F`).
pub fn section_heading(format: ExamItemFormat) -> &'static str {
    match format {
        ExamItemFormat::OpenQuestion => "Part A: conceptual recall",
        ExamItemFormat::CaseScenario => "Part B: case scenarios",
        ExamItemFormat::Calculation => "Part C: calculations",
        ExamItemFormat::FillInGaps => "Part D: fill in the gaps",
        ExamItemFormat::SingleSelect => "Part E: single select",
        ExamItemFormat::MultipleSelect => "Part F: multiple select",
    }
}

/// Formats in exam section order (`Part A` through `Part F`).
pub fn section_order() -> [ExamItemFormat; 6] {
    [
        ExamItemFormat::OpenQuestion,
        ExamItemFormat::CaseScenario,
        ExamItemFormat::Calculation,
        ExamItemFormat::FillInGaps,
        ExamItemFormat::SingleSelect,
        ExamItemFormat::MultipleSelect,
    ]
}
