//! Grading a choice answer in a rehearsal run (STORY-0221). The options come from the item's
//! prompt (`studyfrog-md`), the grade from the mock exam's rules (`studyfrog-rehearsal`), and the
//! suggested rating from the verdict, as a typed answer's is.

use studyfrog_core::uid::Uid;
use studyfrog_md::choice::{
    ChoiceBlock, correct_count, correct_indices, is_multiple, parse_choice,
};
use studyfrog_md::doc::Doc;
use studyfrog_rehearsal::choice_grade::{ChoiceGrade, ChoiceKey, grade_choice};
use studyfrog_slm::rating::suggested_rating;
use studyfrog_slm::score::{Score, Verdict};

use crate::compare::Scored;
use crate::content::load_doc;
use crate::session::{ServiceError, VaultSession};

/// A graded choice answer.
#[derive(Debug, Clone, PartialEq)]
pub struct ChoiceScored {
    /// The grade. Its `given` text is the answer to submit to the run.
    pub grade: ChoiceGrade,
    /// The score and suggested rating, recorded as a typed answer's are.
    pub scored: Scored,
}

/// Grade `picked` (one flag per option, in option order) for item `uid`.
///
/// A card's choice block is its front; a question's is its prompt. Returns an error when the item
/// is not a card or question, or when its prompt is not a usable choice block.
pub fn grade_choice_answer(
    session: &VaultSession,
    uid: Uid,
    picked: &[bool],
) -> Result<ChoiceScored, ServiceError> {
    let doc = load_doc(session, uid)?;
    let block = choice_block(&doc, uid)?;
    let key = ChoiceKey {
        options: block.options.iter().map(|o| o.text.clone()).collect(),
        correct_indices: correct_indices(&block),
    };
    let grade = grade_timed(&key, picked).ok_or_else(|| {
        ServiceError::Invalid("the choice block has no correct option".to_owned())
    })?;
    let mode = if grade.multiple { "multiple" } else { "single" };
    studyfrog_log::log_info!(
        "choice",
        "choice answer graded for {:?}",
        uid;
        correct = grade.correct,
        mode = mode
    );
    let verdict = if grade.correct {
        Verdict::Correct
    } else {
        Verdict::Incorrect
    };
    let score = Score {
        value: if grade.correct { 1.0 } else { 0.0 },
        verdict,
        rubric_hits: Vec::new(),
    };
    let scored = Scored {
        score,
        rating: suggested_rating(verdict),
        engine: "choice".to_owned(),
    };
    Ok(ChoiceScored { grade, scored })
}

/// The choice block of a card (its front) or a question (its prompt), logged when it is parsed.
fn choice_block(doc: &Doc, uid: Uid) -> Result<ChoiceBlock, ServiceError> {
    let text = match doc {
        Doc::Card(card) => &card.front,
        Doc::Question(question) => &question.prompt,
        _ => {
            return Err(ServiceError::Invalid(
                "only cards and questions take choice answers".to_owned(),
            ));
        }
    };
    let block = parse_choice(text).map_err(|e| {
        studyfrog_log::log_warn!("choice", "rejected choice block for {:?}: {}", uid, e);
        ServiceError::Invalid(format!("choice block: {e}"))
    })?;
    let shape = if is_multiple(&block) {
        "multiple"
    } else {
        "single"
    };
    studyfrog_log::log_debug!(
        "choice",
        "parsed {} options, {} correct, {}",
        block.options.len(),
        correct_count(&block),
        shape
    );
    studyfrog_debug::check(
        "choice",
        block.options.len() >= 2,
        "a choice block needs at least two options",
    );
    studyfrog_debug::dump("choice", "parsed choice block", &block);
    Ok(block)
}

/// [`grade_choice`] timed at Debug level.
fn grade_timed(key: &ChoiceKey, picked: &[bool]) -> Option<ChoiceGrade> {
    let _timer = studyfrog_debug::timed("choice", "grade");
    grade_choice(key, picked)
}
