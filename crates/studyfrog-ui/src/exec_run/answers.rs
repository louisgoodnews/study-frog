//! Scoring a typed answer, a case check or a choice, and storing the score with its run.

use studyfrog_core::choice_grading::ChoiceGrading;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::choice::grade_choice_answer;
use studyfrog_service::compare::{record_answer_score, score_item_answer_with};
use studyfrog_service::session::VaultSession;
use studyfrog_slm::lexical::Lexical;
use studyfrog_slm::models::{Opened, open_runtime};

use crate::exec::Worker;
use crate::msg::{Msg, MsgRehearsal};
use crate::run_state::{AnswerEffect, Scored};

use super::millis;

/// Compare a typed answer with the item's reference (offline, with the built-in lexical runtime).
pub fn score(
    w: &mut Worker,
    (run, uid): (Uid, Uid),
    (answer, reference): &(String, Option<String>),
    words_only: bool,
    now_ns: i64,
) -> Msg {
    let (models, loader) = (w.models_dir.clone(), w.loader);
    let chosen: Opened = (Box::new(Lexical), Some("chosen in the settings".to_owned()));
    let (runtime, why) = if words_only {
        &chosen
    } else {
        w.slm
            .get_or_insert_with(|| open_runtime(models.as_deref(), loader))
    };
    let result = match &w.session {
        Some(session) => {
            score_item_answer_with(session, runtime.as_ref(), uid, answer, reference.as_deref())
                .map(|s| match why {
                    Some(why) => Scored {
                        engine: format!("{} ({why})", s.engine),
                        ..s
                    },
                    None => s,
                })
                .map(|s| remember(session, run, uid, s, now_ns))
                .map_err(|e| e.to_string())
        }
        None => Err("no vault is open".to_owned()),
    };
    Msg::Rehearsal(MsgRehearsal::AnswerScored(uid, result))
}

/// Run one answer effect: a typed answer's score, a case check's record or a choice's grade.
pub fn answer(w: &mut Worker, effect: AnswerEffect, now_ns: i64) -> Msg {
    match effect {
        AnswerEffect::Score {
            run,
            item,
            answer,
            words_only,
        } => score(w, (run, item), &answer, words_only, now_ns),
        AnswerEffect::Record { run, item, scored } => record_case(w, run, item, &scored, now_ns),
        AnswerEffect::Choice {
            run,
            item,
            picked,
            scheme,
        } => choice(w, (run, item), &picked, scheme, now_ns),
    }
}

/// Grade a choice answer with the service's rules and store it like a typed answer's score (STORY-0222).
pub fn choice(
    w: &mut Worker,
    (run, uid): (Uid, Uid),
    picked: &[bool],
    scheme: ChoiceGrading,
    now_ns: i64,
) -> Msg {
    let result = match &w.session {
        Some(session) => grade_choice_answer(session, uid, picked, scheme)
            .map(|graded| remember(session, run, uid, graded.scored, now_ns))
            .map_err(|e| e.to_string()),
        None => Err("no vault is open".to_owned()),
    };
    Msg::Rehearsal(MsgRehearsal::AnswerScored(uid, result))
}

/// Store the comparison with its run; a failure is noted in the engine text, never hidden.
fn remember(session: &VaultSession, run: Uid, item: Uid, s: Scored, now_ns: i64) -> Scored {
    match record_answer_score(session, run, item, &s, UnixMs(millis(now_ns))) {
        Ok(()) => s,
        Err(e) => Scored {
            engine: format!("{} (not saved: {e})", s.engine),
            ..s
        },
    }
}

/// Store a case check's score for its item, the way a typed answer's score is stored.
pub fn record_case(
    w: &mut Worker,
    run: Uid,
    item: Uid,
    scored: &studyfrog_service::compare::Scored,
    now_ns: i64,
) -> Msg {
    let done = match w.session.as_ref() {
        None => Err("no vault is open".to_owned()),
        Some(s) => record_answer_score(s, run, item, scored, UnixMs(millis(now_ns)))
            .map_err(|e| e.to_string()),
    };
    Msg::Case(crate::setup::case::CaseMsg::Recorded(done))
}
