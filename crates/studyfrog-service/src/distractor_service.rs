//! Semantic distractor extraction service using an SLM runtime.
//!
//! Orchestrates embedding generation and distractor candidate selection
//! for multiple-choice questions across cards in the active stack or vault.

use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::distractor::{
    DistractorCandidate, MultipleChoiceQuestion, build_multiple_choice_question,
    select_semantic_distractors_excluding,
};
use studyfrog_slm::runtime::SlmRuntime;
use studyfrog_store::items::list_items;

use crate::content::load_item_content;
use crate::session::{ServiceError, VaultSession};

/// Collect distractor candidates by embedding the given answer strings.
pub fn embed_candidates<R: SlmRuntime + ?Sized>(
    runtime: &R,
    answers: &[String],
) -> Result<Vec<DistractorCandidate>, ServiceError> {
    if answers.is_empty() {
        return Ok(Vec::new());
    }
    let refs: Vec<&str> = answers.iter().map(|s| s.as_str()).collect();
    let embeddings = runtime
        .embed(&refs)
        .map_err(|e| ServiceError::Invalid(e.to_string()))?;

    Ok(answers
        .iter()
        .zip(embeddings)
        .map(|(text, embedding)| DistractorCandidate {
            text: text.clone(),
            embedding,
        })
        .collect())
}

/// Retrieve the top `count` semantic distractors for a given answer among candidate answers.
pub fn retrieve_distractors<R: SlmRuntime + ?Sized>(
    runtime: &R,
    target_answer: &str,
    candidate_answers: &[String],
    count: usize,
) -> Result<Vec<String>, ServiceError> {
    if count == 0 || candidate_answers.is_empty() {
        return Ok(Vec::new());
    }
    let target_embs = runtime
        .embed(&[target_answer])
        .map_err(|e| ServiceError::Invalid(e.to_string()))?;
    let target_embedding = target_embs.first().ok_or_else(|| {
        ServiceError::Invalid("runtime returned empty embedding for target".to_string())
    })?;

    let candidates = embed_candidates(runtime, candidate_answers)?;
    Ok(select_semantic_distractors_excluding(
        target_embedding,
        &candidates,
        count,
        &[target_answer],
    ))
}

/// Build a multiple-choice question for a target card or question item using candidate answers
/// loaded from `candidate_uids`.
pub fn build_question_for_item<R: SlmRuntime + ?Sized>(
    session: &VaultSession,
    runtime: &R,
    target_uid: Uid,
    candidate_uids: &[Uid],
    choice_count: usize,
    seed: u64,
) -> Result<MultipleChoiceQuestion, ServiceError> {
    let target_content = load_item_content(session, target_uid)?;
    let target_embs = runtime
        .embed(&[&target_content.answer])
        .map_err(|e| ServiceError::Invalid(e.to_string()))?;
    let target_embedding = target_embs.first().ok_or_else(|| {
        ServiceError::Invalid("runtime returned empty embedding for target".to_string())
    })?;

    let mut candidate_answers = Vec::new();
    for &uid in candidate_uids {
        if uid == target_uid {
            continue;
        }
        let Ok(content) = load_item_content(session, uid) else {
            continue;
        };
        if !content.answer.trim().is_empty() {
            candidate_answers.push(content.answer);
        }
    }

    let candidates = embed_candidates(runtime, &candidate_answers)?;

    Ok(build_multiple_choice_question(
        &target_content.prompt,
        &target_content.answer,
        target_embedding,
        &candidates,
        choice_count,
        seed,
    ))
}

/// Build a multiple-choice question for an item using all other items in the same stack (or inbox).
pub fn build_question_from_stack<R: SlmRuntime + ?Sized>(
    session: &VaultSession,
    runtime: &R,
    target_uid: Uid,
    folder: Option<&str>,
    choice_count: usize,
    seed: u64,
) -> Result<MultipleChoiceQuestion, ServiceError> {
    let rows = list_items(&session.conn, folder).map_err(ServiceError::Store)?;
    let candidate_uids: Vec<Uid> = rows
        .into_iter()
        .filter(|r| r.uid != target_uid)
        .map(|r| r.uid)
        .collect();

    build_question_for_item(
        session,
        runtime,
        target_uid,
        &candidate_uids,
        choice_count,
        seed,
    )
}
