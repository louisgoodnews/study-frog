//! Case-study rehearsal on the rehearse screen (STORY-0181): the case on screen, the choice and the
//! justification, and the evaluation. The rules are in `studyfrog-rehearsal::case_study`, the scoring
//! in `studyfrog-agent::rubric_eval`. Pure: no egui, no I/O.

use studyfrog_agent::rubric_eval::{EvalReport, fallback_evaluate};
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::case_study::{CaseStudyItem, choice_is_correct};

use crate::effect::Effect;
use crate::state::{Selection, State};

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
        CaseMsg::Submit => (map(state, submit), Vec::new()),
        CaseMsg::Next => (map(state, next), Vec::new()),
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

fn with_case(state: State, case: Option<CaseUi>) -> State {
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
    let folder = match &state.browser.selection {
        Selection::Stack(f) => Some(f.clone()),
        Selection::Inbox => None,
        Selection::Nothing => {
            let cs = CaseUi {
                error: Some("Open a stack first, then its case studies.".to_owned()),
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

/// Score the answer: whether the ticked options are the expected choices, and the justification
/// against the rubric.
fn submit(cs: CaseUi) -> CaseUi {
    let Some((_, case)) = cs.cases.get(cs.current) else {
        return cs;
    };
    let valid = choice_is_correct(case, &picked_names(&cs));
    let report = fallback_evaluate(&cs.rationale, valid, &case.rubric_criteria);
    CaseUi {
        report: Some(report),
        ..cs
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
        ..cs
    }
}
