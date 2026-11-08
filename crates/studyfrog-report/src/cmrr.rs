//! Workload against desired retention, and the minimum recommended retention (CMRR): the deck is simulated at
//! each target from 0.75 to 0.97 and the target that keeps the most knowledge per minute of study is picked
//! (STORY-0106). Pure and seeded; every target sees the same random numbers, so the curve is smooth.

use studyfrog_srs::params::{Params, params_new};

use crate::sim_engine::{SimCard, SimSetup, day_total, recall_on, rollout, trajectory_seed};

/// The targets tried.
pub const RETENTION_GRID: [f32; 12] = [
    0.75, 0.78, 0.80, 0.82, 0.84, 0.86, 0.88, 0.90, 0.92, 0.94, 0.96, 0.97,
];

/// The result at one target.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CmrrPoint {
    /// The desired retention.
    pub retention: f32,
    /// Reviews per day on average.
    pub daily_reviews: f32,
    /// Minutes per day on average.
    pub daily_minutes: f32,
    /// Expected cards remembered at the end (the sum of their recall).
    pub memorized: f32,
    /// Mean recall of the seen cards at the end.
    pub mean_retention: f32,
    /// Cards remembered per daily minute of study.
    pub efficiency: f32,
}

/// The curve and what it recommends.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CmrrReport {
    /// Days simulated.
    pub days: usize,
    /// One point per target, in order.
    pub points: Vec<CmrrPoint>,
    /// The target with the least daily time: below it, forgetting adds more work than it saves.
    pub min_workload_retention: Option<f32>,
    /// The target (at or above the least-work one) with the most cards remembered per minute.
    pub recommended: Option<f32>,
}

fn point(cards: &[SimCard], s: &SimSetup, p: &Params, retention: f32) -> CmrrPoint {
    let p = params_new(p.weights, retention, p.maximum_interval_days);
    let n = s.trajectories.max(1);
    let end = s.today + s.days as i64;
    let (mut reviews, mut seconds, mut memorized, mut seen) = (0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
    for t in 0..n {
        let mut deck = cards.to_vec();
        let mut rng = trajectory_seed(s.seed, t);
        let days = rollout(&mut deck, s, &p, &mut rng);
        reviews += days.iter().map(day_total).sum::<u32>() as f32;
        seconds += days.iter().map(|d| d.seconds).sum::<f32>();
        for r in deck.iter().filter_map(|c| recall_on(c, end, &p)) {
            (memorized, seen) = (memorized + r, seen + 1.0);
        }
    }
    let (n, days) = (n as f32, s.days.max(1) as f32);
    let daily_minutes = seconds / n / days / 60.0;
    let memorized = memorized / n;
    CmrrPoint {
        retention,
        daily_reviews: reviews / n / days,
        daily_minutes,
        memorized,
        mean_retention: if seen > 0.0 {
            memorized * n / seen
        } else {
            0.0
        },
        efficiency: memorized / daily_minutes.max(0.1),
    }
}

/// The curve over [`RETENTION_GRID`] and the recommendation; nothing is recommended for an empty deck.
pub fn cmrr(cards: &[SimCard], s: &SimSetup, p: &Params) -> CmrrReport {
    let points: Vec<CmrrPoint> = RETENTION_GRID
        .iter()
        .map(|r| point(cards, s, p, *r))
        .collect();
    let busy = points.iter().any(|x| x.daily_reviews > 0.0);
    let least = points
        .iter()
        .filter(|_| busy)
        .min_by(|a, b| a.daily_minutes.total_cmp(&b.daily_minutes))
        .map(|x| x.retention);
    let recommended = points
        .iter()
        .filter(|x| least.is_some_and(|l| x.retention >= l))
        .max_by(|a, b| a.efficiency.total_cmp(&b.efficiency))
        .map(|x| x.retention);
    CmrrReport {
        days: s.days,
        points,
        min_workload_retention: least,
        recommended,
    }
}
