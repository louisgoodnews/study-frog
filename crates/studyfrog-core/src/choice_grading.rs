//! How a choice item (single choice or multiple select) earns credit (STORY-0231). The scheme is a
//! global app setting; the fraction it credits is computed by `studyfrog-rehearsal`.

/// The grading scheme for choice items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChoiceGrading {
    /// Full credit only when the chosen options are exactly the correct ones (the default).
    #[default]
    AllOrNothing,
    /// Zapechelnyuk (2015): 1/k when at least one chosen option is correct, k being the number chosen.
    Zapechelnyuk,
    /// (correct chosen minus wrong chosen) divided by the number of correct options, never below zero.
    CountPartial,
}

/// Persisted name of a grading scheme.
pub fn choice_grading_name(g: ChoiceGrading) -> &'static str {
    match g {
        ChoiceGrading::AllOrNothing => "all-or-nothing",
        ChoiceGrading::Zapechelnyuk => "zapechelnyuk",
        ChoiceGrading::CountPartial => "count-partial",
    }
}

/// Parse a persisted grading scheme name.
pub fn choice_grading_from_name(name: &str) -> Option<ChoiceGrading> {
    match name {
        "all-or-nothing" => Some(ChoiceGrading::AllOrNothing),
        "zapechelnyuk" => Some(ChoiceGrading::Zapechelnyuk),
        "count-partial" => Some(ChoiceGrading::CountPartial),
        _ => None,
    }
}

/// The scheme's name as shown to the user, as in "Zapechelnyuk rule".
pub fn choice_grading_label(g: ChoiceGrading) -> &'static str {
    match g {
        ChoiceGrading::AllOrNothing => "all-or-nothing",
        ChoiceGrading::Zapechelnyuk => "Zapechelnyuk",
        ChoiceGrading::CountPartial => "count-based partial",
    }
}
