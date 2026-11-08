//! The workload ahead from simulated futures of the deck: reviews per day by stage with a P10/P50/P90 band and
//! the study minutes they take (STORY-0105). Pure and seeded.

use studyfrog_srs::params::Params;

use crate::latency::percentile;
use crate::sim_engine::{DayCount, SimCard, SimSetup, day_total, rollout, trajectory_seed};

/// One day of the forecast.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DayLoad {
    /// Expected first reviews.
    pub new: f32,
    /// Expected reviews after a lapse.
    pub learning: f32,
    /// Expected reviews of young cards (stability under 21 days).
    pub young: f32,
    /// Expected reviews of mature cards.
    pub mature: f32,
    /// Reviews in a low future (10th percentile of the futures).
    pub p10: f32,
    /// Reviews in a typical future (median).
    pub p50: f32,
    /// Reviews in a busy future (90th percentile).
    pub p90: f32,
    /// Expected minutes of study.
    pub minutes: f32,
    /// Minutes in a low future.
    pub minutes_p10: f32,
    /// Minutes in a busy future.
    pub minutes_p90: f32,
}

/// Expected reviews of a day (the sum of the stages).
pub fn load_total(d: &DayLoad) -> f32 {
    d.new + d.learning + d.young + d.mature
}

fn pct(values: &mut [u32], p: u32) -> f32 {
    values.sort_unstable();
    percentile(values, p).unwrap_or(0) as f32
}

fn day_load(runs: &[Vec<DayCount>], d: usize) -> DayLoad {
    let n = runs.len().max(1) as f32;
    let mean = |f: fn(&DayCount) -> f32| runs.iter().map(|r| f(&r[d])).sum::<f32>() / n;
    let mut totals: Vec<u32> = runs.iter().map(|r| day_total(&r[d])).collect();
    let mut seconds: Vec<u32> = runs.iter().map(|r| r[d].seconds.round() as u32).collect();
    DayLoad {
        new: mean(|c| c.new as f32),
        learning: mean(|c| c.learning as f32),
        young: mean(|c| c.young as f32),
        mature: mean(|c| c.mature as f32),
        p10: pct(&mut totals, 10),
        p50: pct(&mut totals, 50),
        p90: pct(&mut totals, 90),
        minutes: mean(|c| c.seconds) / 60.0,
        minutes_p10: pct(&mut seconds, 10) / 60.0,
        minutes_p90: pct(&mut seconds, 90) / 60.0,
    }
}

/// The forecast of `s.days` days from `s.trajectories` simulated futures of `cards`.
pub fn simulate_forecast(cards: &[SimCard], s: &SimSetup, p: &Params) -> Vec<DayLoad> {
    let runs: Vec<Vec<DayCount>> = (0..s.trajectories.max(1))
        .map(|t| {
            let mut deck = cards.to_vec();
            let mut rng = trajectory_seed(s.seed, t);
            rollout(&mut deck, s, p, &mut rng)
        })
        .collect();
    (0..s.days).map(|d| day_load(&runs, d)).collect()
}
