//! From a verdict to the rating the scheduler understands.

use studyfrog_core::review::Rating;

use crate::score::Verdict;

/// The rating a verdict suggests; the user can always choose another.
pub fn suggested_rating(v: Verdict) -> Rating {
    match v {
        Verdict::Correct => Rating::Good,
        Verdict::Partial => Rating::Hard,
        Verdict::Incorrect => Rating::Again,
    }
}
