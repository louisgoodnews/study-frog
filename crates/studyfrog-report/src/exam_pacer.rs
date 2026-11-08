//! The exam deadline pacer: how many new cards a day and how many reviews a stack needs so that nine in ten of
//! its cards are remembered on the exam day, with the last days kept for consolidation (STORY-0105). Pure.

use studyfrog_core::memory::MemState;
use studyfrog_core::review::Rating;
use studyfrog_srs::fsrs::{next_state, retrievability};
use studyfrog_srs::params::Params;

use crate::sim_engine::{
    Pace, RatingMix, SimCard, SimSetup, day_total, pace_default, recall_on, rollout, stability_of,
    trajectory_seed,
};

/// Share of cards that must reach the target on the exam day.
pub const READY_SHARE: f32 = 0.9;
/// Futures simulated for the pace.
pub const PACER_TRAJECTORIES: u32 = 8;

/// What the pacer is asked.
#[derive(Debug, Clone, PartialEq)]
pub struct PacerInput {
    /// Today.
    pub today: i64,
    /// The exam or due day.
    pub exam_day: i64,
    /// Recall each card should have on the exam day.
    pub target: f32,
    /// Last days kept for consolidation, without new cards.
    pub buffer_days: u32,
    /// How the learner rates.
    pub mix: RatingMix,
    /// The stack's pace.
    pub pace: Pace,
    /// Starts the random sequence.
    pub seed: u64,
}

/// The plan.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExamPace {
    /// Days until the exam.
    pub days_left: i64,
    /// Days new cards can still be started.
    pub study_days: i64,
    /// Cards never seen.
    pub unseen_items: usize,
    /// Seen cards that would be under the target on the exam day without another review.
    pub volatile_items: usize,
    /// New cards to start each study day.
    pub required_new_per_day: u32,
    /// Reviews per day on average, consolidation included.
    pub reviews_per_day: f32,
    /// Minutes per day on average.
    pub minutes_per_day: f32,
    /// Reviews in the last days to lift cards over the target.
    pub consolidation_reviews: f32,
    /// Share of cards at or over the target on the exam day if the plan is kept.
    pub ready_share: f32,
    /// Whether the plan reaches [`READY_SHARE`].
    pub on_track: bool,
}

/// Recall on the exam day of a card reviewed (and recalled, rated Good) the day before.
fn consolidated(card: &SimCard, exam_day: i64, p: &Params) -> f32 {
    let Some(m) = card.memory else { return 0.0 };
    let s = MemState {
        stability: stability_of(&m).max(0.01),
        difficulty: 5.0,
    };
    let elapsed = (exam_day - 1 - card.last_day).max(0) as f32;
    retrievability(&next_state(&s, Rating::Good, elapsed, p), 1.0, p)
}

/// Totals of one simulated future: reviews, seconds, consolidation reviews, ready cards.
fn one_future(
    cards: &[SimCard],
    s: &SimSetup,
    i: &PacerInput,
    p: &Params,
    t: u32,
) -> (f32, f32, f32, f32) {
    let mut deck = cards.to_vec();
    let mut rng = trajectory_seed(s.seed, t);
    let days = rollout(&mut deck, s, p, &mut rng);
    let reviews: u32 = days.iter().map(day_total).sum();
    let seconds: f32 = days.iter().map(|d| d.seconds).sum();
    let (mut extra, mut ready) = (0.0_f32, 0.0_f32);
    for c in &deck {
        let r = recall_on(c, i.exam_day, p).unwrap_or(0.0);
        let lifted = r < i.target && c.memory.is_some();
        extra += f32::from(u8::from(lifted));
        let at_exam = if lifted {
            consolidated(c, i.exam_day, p)
        } else {
            r
        };
        ready += f32::from(u8::from(at_exam >= i.target));
    }
    (
        reviews as f32,
        seconds + extra * i.pace.review_s,
        extra,
        ready,
    )
}

/// The pace for `cards` (one stack's), `None` when the exam is not ahead.
pub fn exam_pace(cards: &[SimCard], i: &PacerInput, p: &Params) -> Option<ExamPace> {
    let days_left = i.exam_day - i.today;
    if days_left <= 0 {
        return None;
    }
    let buffer = i64::from(i.buffer_days).min(days_left - 1);
    let study_days = (days_left - buffer).max(1);
    let unseen = cards.iter().filter(|c| c.memory.is_none()).count();
    let volatile = cards
        .iter()
        .filter(|c| recall_on(c, i.exam_day, p).is_some_and(|r| r < i.target))
        .count();
    let required = (unseen as i64 + study_days - 1) / study_days;
    let setup = SimSetup {
        today: i.today,
        days: days_left as usize,
        trajectories: PACER_TRAJECTORIES,
        seed: i.seed,
        miss_probability: 0.0,
        new_per_day: required as f32,
        new_days: study_days as usize,
        mix: i.mix,
        paces: vec![i.pace],
    };
    let deck: Vec<SimCard> = cards.iter().map(|c| SimCard { pace: 0, ..*c }).collect();
    let n = PACER_TRAJECTORIES as f32;
    let (reviews, seconds, extra, ready) = (0..PACER_TRAJECTORIES)
        .map(|t| one_future(&deck, &setup, i, p, t))
        .fold((0.0, 0.0, 0.0, 0.0), |a, b| {
            (a.0 + b.0, a.1 + b.1, a.2 + b.2, a.3 + b.3)
        });
    let ready_share = if cards.is_empty() {
        1.0
    } else {
        ready / n / cards.len() as f32
    };
    Some(ExamPace {
        days_left,
        study_days,
        unseen_items: unseen,
        volatile_items: volatile,
        required_new_per_day: required as u32,
        reviews_per_day: (reviews + extra) / n / days_left as f32,
        minutes_per_day: seconds / n / 60.0 / days_left as f32,
        consolidation_reviews: extra / n,
        ready_share,
        on_track: ready_share >= READY_SHARE,
    })
}

/// A pacer input with the default pace and buffer (3 days, target 0.9).
pub fn pacer_input(today: i64, exam_day: i64, mix: RatingMix) -> PacerInput {
    PacerInput {
        today,
        exam_day,
        target: 0.9,
        buffer_days: 3,
        mix,
        pace: pace_default(),
        seed: 1,
    }
}
