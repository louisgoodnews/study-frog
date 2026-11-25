//! Choice items in a rehearsal run (STORY-0222). The options are picked here; a submit is graded by
//! the rehearsal grader and the service, then sent through the engine's typed-answer path.

use studyfrog_core::run::RunMode;
use studyfrog_core::uid::Uid;
use studyfrog_md::choice::{ChoiceBlock, correct_indices, is_multiple, parse_choice};
use studyfrog_rehearsal::choice_grade::{ChoiceKey, grade_choice};
use studyfrog_rehearsal::input::Action;
use studyfrog_rehearsal::session::current_item;

use crate::effect::{Effect, EffectRehearsal};
use crate::run_state::choice::{
    ChoiceSelection, can_submit, chosen_count, from_indices, option_for_number, pick, picked_for,
};
use crate::run_state::{AnswerEffect, ItemContent, RunView};
use crate::run_update::{act, awaiting_reveal};
use crate::state::State;

/// A message from the choice options or the control protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChoiceMsg {
    /// A click or number key on option `n` (0-based): toggled, or selected for single choice.
    Pick(usize),
    /// Replace the selection with these options (0-based), from `rehearse choose`.
    Set(Vec<usize>),
    /// Submit the selection (`rehearse submit`, Enter, or the Submit button).
    Submit,
}

/// A loaded item's choice block, when it is a choice item in an answer run. The front matter's
/// `answer_type` must agree with the option markers in the body; a disagreement is logged and the
/// item is answered as typed text instead.
pub(crate) fn note_item(run: &mut RunView, uid: Uid, content: &ItemContent) {
    if run.session.run.mode != RunMode::Answer || content.is_note {
        return;
    }
    let declared = content.answer_type.as_deref();
    if declared == Some("open") {
        return;
    }
    let Ok(block) = parse_choice(&content.prompt) else {
        return;
    };
    let kind = if is_multiple(&block) {
        "multiple"
    } else {
        "single"
    };
    if declared.is_some_and(|d| d != kind) {
        studyfrog_log::log_warn!(
            "choice",
            "answer_type {} does not match the {} correct options",
            declared.unwrap_or_default(),
            kind
        );
        return;
    }
    studyfrog_log::log_debug!(
        "choice",
        "card detects a {} choice item with {} options",
        kind,
        block.options.len()
    );
    run.choices.insert(uid, block);
}

/// The current item and its choice block, when the current item is a choice item.
pub(crate) fn pending_block(run: &RunView) -> Option<(Uid, &ChoiceBlock)> {
    let uid = current_item(&run.session)?;
    run.choices.get(&uid).map(|block| (uid, block))
}

/// Whether a choice item is on screen awaiting its submit.
pub(crate) fn is_pending(state: &State) -> bool {
    state
        .rehearse
        .run
        .as_ref()
        .is_some_and(|run| pending_block(run).is_some())
}

/// Check `rehearse choose` numbers (1-based) against the current choice item, and give them as
/// 0-based option indices.
pub(crate) fn check_choose(state: &State, numbers: &[usize]) -> Result<Vec<usize>, String> {
    let run = state.rehearse.run.as_ref().ok_or("no run is on screen")?;
    let (_, block) = pending_block(run).ok_or("the current item is not a choice item")?;
    let len = block.options.len();
    let indices = numbers
        .iter()
        .map(|&n| option_for_number(n, len).ok_or(format!("option {n} is not one of the {len}")))
        .collect::<Result<Vec<usize>, String>>()?;
    if !is_multiple(block) && indices.len() > 1 {
        return Err("a single choice takes one option: rehearse choose <n>".to_owned());
    }
    Ok(indices)
}

/// Handle a choice message: the selection changes, or a submit is graded and sent on.
pub(crate) fn update(state: State, msg: ChoiceMsg) -> (State, Vec<Effect>) {
    let mut state = state;
    let Some(run) = state.rehearse.run.as_ref() else {
        return (state, Vec::new());
    };
    let (uid, block) = match pending_block(run) {
        Some((uid, block)) => (uid, block.clone()),
        None => {
            studyfrog_log::log_warn!("choice", "a choice message arrived with no choice item");
            return (state, Vec::new());
        }
    };
    if !awaiting_reveal(run) {
        studyfrog_log::log_warn!("choice", "the choice is already answered");
        return (state, Vec::new());
    }
    let multiple = is_multiple(&block);
    let len = block.options.len();
    let current = picked_for(&run.choice, uid, len);
    let kind = if multiple { "multiple" } else { "single" };
    match msg {
        ChoiceMsg::Pick(n) if n < len => {
            let next = pick(&current, n, multiple);
            log_option(&next, kind);
            store(&mut state, uid, next, len);
            (state, Vec::new())
        }
        ChoiceMsg::Set(indices) => {
            let Some(next) = from_indices(len, &indices).filter(|_| multiple || indices.len() <= 1)
            else {
                studyfrog_log::log_warn!("choice", "the chosen options do not fit the item");
                return (state, Vec::new());
            };
            store(&mut state, uid, next, len);
            (state, Vec::new())
        }
        ChoiceMsg::Submit => submit(state, uid, &block, &current),
        ChoiceMsg::Pick(n) => {
            studyfrog_log::log_warn!("choice", "option {} is not on the card", n + 1);
            (state, Vec::new())
        }
    }
}

/// Grade the selection and send it through the typed-answer path; the service then records it.
fn submit(state: State, uid: Uid, block: &ChoiceBlock, picked: &[bool]) -> (State, Vec<Effect>) {
    if !can_submit(picked) {
        studyfrog_log::log_warn!("choice", "submit requested with nothing chosen");
        return (state, Vec::new());
    }
    let key = ChoiceKey {
        options: block.options.iter().map(|o| o.text.clone()).collect(),
        correct_indices: correct_indices(block),
    };
    let Some(grade) = grade_choice(&key, picked) else {
        return (state, Vec::new());
    };
    let mode = if grade.multiple { "multiple" } else { "single" };
    studyfrog_debug::count("choice.submit");
    studyfrog_debug::dump("choice", "submitted selection", &picked);
    studyfrog_log::log_info!(
        "choice",
        "choice answer submitted";
        chosen = chosen_count(picked),
        mode = mode
    );
    let run = state.rehearse.run.as_ref().map(|r| r.session.run.uid);
    let scheme = state.app.choice_grading;
    let (state, mut effects) = act(state, Action::SubmitAnswer(grade.given));
    // The engine's text comparison is not the choice verdict: the service grades the picks instead.
    effects.retain(|e| !matches!(e, Effect::Answer(AnswerEffect::Score { .. })));
    let accepted = effects
        .iter()
        .any(|e| matches!(e, Effect::Rehearsal(EffectRehearsal::RecordStep { .. })));
    if let (Some(run), true) = (run, accepted) {
        effects.push(Effect::Answer(AnswerEffect::Choice {
            run,
            item: uid,
            picked: picked.to_vec(),
            scheme,
        }));
    }
    (state, effects)
}

fn log_option(next: &[bool], kind: &str) {
    studyfrog_log::log_info!(
        "choice",
        "option chosen";
        chosen = chosen_count(next),
        mode = kind
    );
}

/// Keep the selection of `item`; its flags must match the item's `len` options.
fn store(state: &mut State, item: Uid, picked: Vec<bool>, len: usize) {
    studyfrog_debug::check(
        "choice",
        picked.len() == len,
        "the selection has one flag per option",
    );
    studyfrog_debug::dump("choice", "selection", &picked);
    if let Some(run) = &mut state.rehearse.run {
        run.choice = ChoiceSelection {
            item: Some(item),
            picked,
        };
    }
}
