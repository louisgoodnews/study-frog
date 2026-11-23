//! The math drill (STORY-0184): a seeded standard-error problem, optional working rows, the check of
//! the working by the formula trace, and the full solution on demand. The formulas and the diagnosis
//! are in `studyfrog-rehearsal`. Pure: no egui, no I/O.

use studyfrog_rehearsal::formula_gen::{FormulaExercise, generate_formula_exercise};
use studyfrog_rehearsal::formula_trace::{Diagnosis, diagnose_se, se_trace};

/// Relative tolerance for the final answer.
const ANSWER_TOL: f64 = 0.01;
/// The labels of the working rows, in trace order.
pub const STEP_LABELS: [&str; 3] = ["1 - r_xx", "sqrt(1 - r_xx)", "s_e"];

/// Messages of the math drill.
#[derive(Debug, Clone, PartialEq)]
pub enum MathMsg {
    /// Open the drill (a new exercise from the clock).
    Open,
    /// A working row was edited.
    Step(usize, String),
    /// The final answer was edited.
    Answer(String),
    /// A row was selected: the keypad types into it.
    Active(usize),
    /// Reveal the value of the next working row (a hint). The answer is not marked.
    Hint,
    /// A key of the keypad was pressed.
    Key(String),
    /// Check the working and the answer.
    Check,
    /// Show or hide the full solution.
    Solution,
    /// A new exercise.
    Next,
    /// Close the drill.
    Close,
}

/// The drill on screen.
#[derive(Debug, Clone, PartialEq)]
pub struct MathDrill {
    /// The seed of the exercise (the same seed gives the same problem).
    pub seed: u64,
    /// The problem.
    pub exercise: FormulaExercise,
    /// The working rows, as typed.
    pub steps: [String; 3],
    /// The row the keypad types into.
    pub active: usize,
    /// The final answer, as typed.
    pub answer: String,
    /// The check of the working and the answer, once checked.
    pub feedback: Option<Feedback>,
    /// The full solution is shown.
    pub solution: bool,
    /// How many working rows have their value revealed by a hint, in order (0 to 3).
    pub hints: usize,
}

/// What the check said.
#[derive(Debug, Clone, PartialEq)]
pub struct Feedback {
    /// The trace's verdict on the working rows typed so far.
    pub diagnosis: Diagnosis,
    /// Whether the final answer is within tolerance.
    pub answer_correct: bool,
}

/// Handle a math-drill message.
pub fn update(
    state: crate::state::State,
    msg: MathMsg,
) -> (crate::state::State, Vec<crate::effect::Effect>) {
    let state_in = state;
    let drill = match msg {
        MathMsg::Open => Some(new_drill(state_in.wall_ms as u64)),
        MathMsg::Close => None,
        other => state_in.rehearse.math.clone().map(|d| apply(d, other)),
    };
    let rehearse = crate::setup::Rehearse {
        math: drill,
        ..state_in.rehearse.clone()
    };
    (
        crate::state::State {
            rehearse,
            ..state_in
        },
        Vec::new(),
    )
}

/// A fresh drill for `seed`.
pub fn new_drill(seed: u64) -> MathDrill {
    MathDrill {
        seed,
        exercise: generate_formula_exercise(seed),
        steps: Default::default(),
        active: 0,
        answer: String::new(),
        feedback: None,
        solution: false,
        hints: 0,
    }
}

fn apply(d: MathDrill, msg: MathMsg) -> MathDrill {
    match msg {
        MathMsg::Step(i, text) => {
            let mut d = d;
            if let Some(slot) = d.steps.get_mut(i) {
                *slot = text;
            }
            MathDrill {
                feedback: None,
                ..d
            }
        }
        MathMsg::Answer(text) => MathDrill {
            answer: text,
            feedback: None,
            ..d
        },
        MathMsg::Active(i) => MathDrill {
            active: i.min(2),
            ..d
        },
        MathMsg::Key(key) => key_into(d, &key),
        MathMsg::Check => check(d),
        MathMsg::Solution => MathDrill {
            solution: !d.solution,
            ..d
        },
        MathMsg::Hint => MathDrill {
            hints: (d.hints + 1).min(STEP_LABELS.len()),
            ..d
        },
        MathMsg::Next => new_drill(d.seed.wrapping_add(1)),
        MathMsg::Open | MathMsg::Close => d,
    }
}

/// The keypad: digits and the decimal point go into the active row; `Del` deletes; `C` clears it.
fn key_into(d: MathDrill, key: &str) -> MathDrill {
    let mut d = d;
    let i = d.active;
    let row = &mut d.steps[i];
    match key {
        "Del" => {
            row.pop();
        }
        "C" => row.clear(),
        "-" => row.insert(0, '-'),
        k => row.push_str(k),
    }
    d.feedback = None;
    d
}

/// Check the working rows typed so far (in order) and the final answer.
fn check(d: MathDrill) -> MathDrill {
    let submitted: Vec<f64> = d
        .steps
        .iter()
        .map_while(|t| t.trim().parse::<f64>().ok())
        .collect();
    let diagnosis = if submitted.is_empty() {
        Diagnosis {
            deviates_at: None,
            mistake: None,
            message: "Work a row, or type the answer, first.".to_owned(),
        }
    } else {
        diagnose_se(d.exercise.s_x, d.exercise.r_xx, &submitted)
    };
    let correct = se_trace(d.exercise.s_x, d.exercise.r_xx)
        .last()
        .map_or(0.0, |s| s.value);
    let answer_correct = d
        .answer
        .trim()
        .parse::<f64>()
        .is_ok_and(|a| (a - correct).abs() <= ANSWER_TOL * correct.abs().max(0.01));
    MathDrill {
        feedback: Some(Feedback {
            diagnosis,
            answer_correct,
        }),
        ..d
    }
}
