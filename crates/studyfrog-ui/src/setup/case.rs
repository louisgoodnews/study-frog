//! Case-study rehearsal on the rehearse screen (STORY-0181): the case on screen, the choice and the
//! justification, and the evaluation. The keyword check runs at once; a local model, when one is
//! selected, checks the justification too and replaces that result when it answers. The rules are in
//! `studyfrog-rehearsal::case_study`, the scoring in `studyfrog-agent::rubric_eval`. Pure.

use studyfrog_agent::rubric_eval::EvalReport;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::case_study::CaseStudyItem;

mod check;
pub mod run;

use crate::effect::Effect;
use crate::setup::stack_source::{StackSource, stack_source};
use crate::state::State;

/// Messages of the case-study view.
#[derive(Debug, Clone, PartialEq)]
pub enum CaseMsg {
    /// Open the case studies of the stack that is selected.
    Open,
    /// The case studies read, or why they could not be.
    Loaded(Result<Vec<(Uid, CaseStudyItem)>, String>),
    /// An option was ticked (or unticked): its index and the new value.
    Pick(usize, bool),
    /// The justification was edited.
    Rationale(String),
    /// Check the answer.
    Submit,
    /// The local model answered (its reply text, or why there was none).
    ModelReplied(Result<String, String>),
    /// An option of the case in a run was ticked (or unticked).
    RunPick(usize, bool),
    /// Check the answer of the case in a run.
    RunCheck,
    /// The case's score was stored, or why not.
    Recorded(Result<(), String>),
    /// The next case.
    Next,
    /// Close the case-study view.
    Close,
}

/// The case-study view: the cases of a stack and the answer given to the current one.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CaseUi {
    /// The valid case studies of the stack.
    pub cases: Vec<(Uid, CaseStudyItem)>,
    /// The case on screen.
    pub current: usize,
    /// Which options are ticked, one per option of the current case.
    pub picked: Vec<bool>,
    /// The justification being written.
    pub rationale: String,
    /// The evaluation of the answer, once it is checked.
    pub report: Option<EvalReport>,
    /// The local model is checking the justification.
    pub pending: bool,
    /// The report came from the local model (otherwise from the keyword check).
    pub checked_by_model: bool,
    /// Why the local model did not check it, when it was asked and could not.
    pub note: Option<String>,
    /// The cases are being read.
    pub loading: bool,
    /// Why there is nothing to show.
    pub error: Option<String>,
}

/// Handle a case-study message.
pub fn update(state: State, msg: CaseMsg) -> (State, Vec<Effect>) {
    match msg {
        CaseMsg::Open => open(state),
        CaseMsg::Loaded(result) => (map(state, |cs| loaded(cs, result)), Vec::new()),
        CaseMsg::Pick(i, on) => (map(state, |cs| pick(cs, i, on)), Vec::new()),
        CaseMsg::Rationale(text) => (
            map(state, |cs| CaseUi {
                rationale: text,
                ..cs
            }),
            Vec::new(),
        ),
        CaseMsg::Submit => check::submit(state),
        CaseMsg::ModelReplied(result) => (
            map(state, |cs| check::model_replied(cs, result)),
            Vec::new(),
        ),
        CaseMsg::Next => (map(state, next), Vec::new()),
        CaseMsg::RunPick(i, on) => (run::pick(state, i, on), Vec::new()),
        CaseMsg::RunCheck => run::check(state),
        CaseMsg::Recorded(result) => (recorded(state, result), Vec::new()),
        CaseMsg::Close => (with_case(state, None), Vec::new()),
    }
}

/// The options ticked in the current case, by name.
pub fn picked_names(cs: &CaseUi) -> Vec<String> {
    let Some((_, case)) = cs.cases.get(cs.current) else {
        return Vec::new();
    };
    case.options
        .iter()
        .zip(&cs.picked)
        .filter(|(_, on)| **on)
        .map(|(o, _)| o.clone())
        .collect()
}

pub(super) fn with_case(state: State, case: Option<CaseUi>) -> State {
    State {
        rehearse: crate::setup::Rehearse {
            case_study: case,
            ..state.rehearse.clone()
        },
        ..state
    }
}

/// Apply `f` to the case view if it is open.
fn map(state: State, f: impl FnOnce(CaseUi) -> CaseUi) -> State {
    match state.rehearse.case_study.clone() {
        Some(cs) => with_case(state, Some(f(cs))),
        None => state,
    }
}

fn open(state: State) -> (State, Vec<Effect>) {
    let folder = match stack_source(&state) {
        StackSource::Stack(f) => Some(f),
        StackSource::Inbox => None,
        StackSource::Nothing => {
            let cs = CaseUi {
                error: Some(
                    "Tick a stack in the What step, or open one in the library, then its case studies."
                        .to_owned(),
                ),
                ..CaseUi::default()
            };
            return (with_case(state, Some(cs)), Vec::new());
        }
    };
    let cs = CaseUi {
        loading: true,
        ..CaseUi::default()
    };
    (with_case(state, Some(cs)), vec![Effect::LoadCases(folder)])
}

fn loaded(cs: CaseUi, result: Result<Vec<(Uid, CaseStudyItem)>, String>) -> CaseUi {
    match result {
        Err(e) => CaseUi {
            loading: false,
            error: Some(e),
            ..cs
        },
        Ok(cases) if cases.is_empty() => CaseUi {
            loading: false,
            error: Some("This stack has no case studies yet.".to_owned()),
            ..cs
        },
        Ok(cases) => {
            let picked = vec![false; options_of(&cases, 0)];
            CaseUi {
                cases,
                loading: false,
                error: None,
                picked,
                ..cs
            }
        }
    }
}

fn options_of(cases: &[(Uid, CaseStudyItem)], idx: usize) -> usize {
    cases.get(idx).map_or(0, |(_, c)| c.options.len())
}

fn pick(cs: CaseUi, i: usize, on: bool) -> CaseUi {
    let mut cs = cs;
    if let Some(slot) = cs.picked.get_mut(i) {
        *slot = on;
    }
    cs
}

/// Say when a case's score could not be stored; a stored score needs no word.
fn recorded(state: State, result: Result<(), String>) -> State {
    match result {
        Ok(()) => state,
        Err(e) => State {
            notice: Some(crate::state::Notice {
                kind: crate::state::NoticeKind::Error,
                text: format!("The case's score was not saved: {e}"),
            }),
            ..state
        },
    }
}

fn next(cs: CaseUi) -> CaseUi {
    if cs.cases.is_empty() {
        return cs;
    }
    let current = (cs.current + 1) % cs.cases.len();
    CaseUi {
        current,
        picked: vec![false; options_of(&cs.cases, current)],
        rationale: String::new(),
        report: None,
        checked_by_model: false,
        note: None,
        pending: false,
        ..cs
    }
}
