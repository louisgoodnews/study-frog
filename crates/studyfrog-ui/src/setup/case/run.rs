//! A case study inside a rehearsal run (STORY-0199): its options and the check of the answer. The check
//! is the rubric check of the case view. The run then reveals the answer as usual, and the rating stays
//! the student's, with the check's suggestion filled in. Pure.

use studyfrog_agent::rubric_eval::{EvalReport, fallback_evaluate};
use studyfrog_core::review::Rating;
use studyfrog_rehearsal::case_study::choice_is_correct;
use studyfrog_rehearsal::session::current_item;
use studyfrog_service::compare::Scored;
use studyfrog_slm::score::{Score, verdict_for};

use crate::effect::Effect;
use crate::run_state::{AnswerEffect, RunView};
use crate::state::State;

/// The engine label of a case check: the rubric check, not a model.
pub const ENGINE: &str = "case rubric check (word overlap, no model)";
/// Score at which a rubric check counts as correct.
const MIN_SCORE: f32 = 0.8;

/// What the student did with the case in the run: the ticked options and the check's report.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CaseRun {
    /// Which options are ticked, one per option of the case.
    pub picks: Vec<bool>,
    /// The check's report, once the answer is checked.
    pub report: Option<EvalReport>,
}

/// Tick or untick an option of the case on screen.
pub fn pick(state: State, i: usize, on: bool) -> State {
    with_run(state, |run| {
        let mut picks = run.case_run.picks.clone();
        let len = picks.len().max(i + 1);
        picks.resize(len, false);
        picks[i] = on;
        RunView {
            case_run: CaseRun {
                picks,
                ..run.case_run.clone()
            },
            ..run
        }
    })
}

/// Check the answer against the case's expected choice and rubric, and suggest a rating.
pub fn check(state: State) -> (State, Vec<Effect>) {
    let mut effects = Vec::new();
    let state = with_run(state, |run| {
        let Some(uid) = current_item(&run.session) else {
            return run;
        };
        let Some(case) = run.content.get(&uid).and_then(|c| c.case.clone()) else {
            return run;
        };
        let names: Vec<String> = case
            .options
            .iter()
            .zip(&run.case_run.picks)
            .filter(|(_, on)| **on)
            .map(|(o, _)| o.clone())
            .collect();
        let valid = choice_is_correct(&case, &names);
        let report = fallback_evaluate(&run.answer, valid, &case.rubric_criteria);
        let scored = Scored {
            score: Score {
                value: report.overall_pct / 100.0,
                verdict: verdict_for(report.overall_pct / 100.0, MIN_SCORE),
                rubric_hits: report.criteria.iter().map(|c| c.met).collect(),
            },
            rating: suggested_rating(&report),
            engine: ENGINE.to_owned(),
        };
        effects.push(Effect::Answer(AnswerEffect::Record {
            run: run.session.run.uid,
            item: uid,
            scored: Box::new(scored.clone()),
        }));
        RunView {
            scored: Some(scored),
            case_run: CaseRun {
                report: Some(report),
                ..run.case_run.clone()
            },
            ..run
        }
    });
    (state, effects)
}

/// The rating the check suggests: by the overall score, at most Hard when the choice is wrong, and Again
/// when the justification makes no rubric point (a right choice alone does not count as a recall).
pub fn suggested_rating(report: &EvalReport) -> Rating {
    if !report.criteria.iter().any(|c| c.met) {
        return Rating::Again;
    }
    let by_score = if report.overall_pct >= 90.0 {
        Rating::Easy
    } else if report.overall_pct >= 70.0 {
        Rating::Good
    } else if report.overall_pct >= 40.0 {
        Rating::Hard
    } else {
        Rating::Again
    };
    if report.choice_valid {
        by_score
    } else {
        by_score.min(Rating::Hard)
    }
}

fn with_run(state: State, f: impl FnOnce(RunView) -> RunView) -> State {
    let mut state = state;
    if let Some(run) = state.rehearse.run.take() {
        state.rehearse.run = Some(f(run));
    }
    state
}
