//! Comparing a typed answer with the item's reference, through whichever runtime is given.

use studyfrog_core::items::DEFAULT_MIN_SCORE;
use studyfrog_core::review::Rating;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::doc::Doc;
use studyfrog_slm::rating::suggested_rating;
use studyfrog_slm::runtime::SlmRuntime;
use studyfrog_slm::score::{Comparison, Score, score_answer, verdict_name};
use studyfrog_store::answer_scores::{AnswerScoreRow, insert_answer_score};

use crate::content::load_doc;
use crate::session::{ServiceError, VaultSession};

/// A score and the rating it suggests.
#[derive(Debug, Clone, PartialEq)]
pub struct Scored {
    /// The comparison result.
    pub score: Score,
    /// What the verdict suggests; the user decides.
    pub rating: Rating,
    /// Which runtime compared the texts.
    pub engine: String,
}

/// The comparison for a document, or why it has none.
fn comparison_of<'a>(doc: &'a Doc, answer: &'a str) -> Result<Comparison<'a>, ServiceError> {
    match doc {
        Doc::Question(q) => Ok(Comparison {
            reference: crate::inline_sync::without_source(&q.reference),
            answer,
            accepted: &q.accepted,
            rubric: &q.rubric,
            min_score: q.min_score,
        }),
        Doc::Card(c) => Ok(Comparison {
            reference: crate::inline_sync::without_source(&c.back),
            answer,
            accepted: &[],
            rubric: &[],
            min_score: DEFAULT_MIN_SCORE,
        }),
        _ => Err(ServiceError::Invalid(
            "only cards and questions have an answer to compare with".to_owned(),
        )),
    }
}

/// Score `answer` against the reference of item `uid` (for a card with cloze gaps, the words of the
/// group it asks now).
pub fn score_item_answer<R: SlmRuntime + ?Sized>(
    session: &VaultSession,
    runtime: &R,
    uid: Uid,
    answer: &str,
) -> Result<Scored, ServiceError> {
    score_item_answer_with(session, runtime, uid, answer, None)
}

/// [`score_item_answer`] against `reference` when given: the words the run showed as the gap.
pub fn score_item_answer_with<R: SlmRuntime + ?Sized>(
    session: &VaultSession,
    runtime: &R,
    uid: Uid,
    answer: &str,
    reference: Option<&str>,
) -> Result<Scored, ServiceError> {
    let doc = load_doc(session, uid)?;
    let reviews = || studyfrog_store::schedule::load_reviews(&session.conn, uid).map(|r| r.len());
    let gap = match reference {
        Some(r) => Some(r.to_owned()),
        None => crate::content_cloze::asked_group(&doc, reviews().map_err(ServiceError::Store)?)
            .and_then(|g| crate::content_cloze::expected_words(&doc, g)),
    };
    let mut comparison = comparison_of(&doc, answer)?;
    if let Some(gap) = &gap {
        comparison.reference = gap;
    }
    let score =
        score_answer(runtime, &comparison).map_err(|e| ServiceError::Invalid(e.to_string()))?;
    let rating = suggested_rating(score.verdict);
    Ok(Scored {
        score,
        rating,
        engine: runtime.name().to_owned(),
    })
}

/// Remember how an answer in a run compared with its reference.
pub fn record_answer_score(
    session: &VaultSession,
    run: Uid,
    item: Uid,
    scored: &Scored,
    now: UnixMs,
) -> Result<(), ServiceError> {
    let row = AnswerScoreRow {
        run,
        item,
        value: scored.score.value,
        verdict: verdict_name(scored.score.verdict).to_owned(),
        engine: scored.engine.clone(),
        at_ms: now.0,
    };
    insert_answer_score(&session.conn, &row).map_err(ServiceError::Store)
}
