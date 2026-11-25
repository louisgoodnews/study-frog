//! The credited fraction of a choice answer under each grading scheme (STORY-0231). Pure: it takes
//! the chosen flags and the correct flags, one per option in the order written, and returns a
//! fraction from 0 to 1 of the item's maximum.

use studyfrog_core::choice_grading::ChoiceGrading;

/// Counts for one answer: correct options chosen, wrong options chosen, and correct options in all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Tally {
    right: usize,
    wrong: usize,
    total: usize,
}

/// The credited fraction (0 to 1) of `picked` against `correct` under `scheme`.
///
/// All-or-nothing gives 1 only when the chosen options are exactly the correct ones. Zapechelnyuk
/// gives 1/k when at least one chosen option is correct, k being the number chosen, and 0 otherwise
/// (nothing chosen is 0). Count-based partial credit is (correct chosen minus wrong chosen) divided
/// by the correct total, never below 0.
pub fn credited_fraction(scheme: ChoiceGrading, picked: &[bool], correct: &[bool]) -> f32 {
    let t = tally(picked, correct);
    let chosen = t.right + t.wrong;
    match scheme {
        ChoiceGrading::AllOrNothing => {
            if t.total > 0 && t.right == t.total && t.wrong == 0 {
                1.0
            } else {
                0.0
            }
        }
        ChoiceGrading::Zapechelnyuk => {
            if t.right == 0 {
                0.0
            } else {
                1.0 / chosen as f32
            }
        }
        ChoiceGrading::CountPartial => {
            if t.total == 0 {
                0.0
            } else {
                ((t.right as f32 - t.wrong as f32) / t.total as f32).clamp(0.0, 1.0)
            }
        }
    }
}

/// Count the correct and wrong options chosen, over the options in `correct`.
fn tally(picked: &[bool], correct: &[bool]) -> Tally {
    let mut t = Tally {
        right: 0,
        wrong: 0,
        total: 0,
    };
    for (i, &is_correct) in correct.iter().enumerate() {
        let chosen = picked.get(i).copied().unwrap_or(false);
        t.total += usize::from(is_correct);
        if chosen && is_correct {
            t.right += 1;
        } else if chosen {
            t.wrong += 1;
        }
    }
    t
}
