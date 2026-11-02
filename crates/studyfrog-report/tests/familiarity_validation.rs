#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Validation of familiarity on the simulated history: before a review, does the value tell the reviews that
//! were recalled from the ones that were forgotten? The result is recorded in docs/specs/reports.md.

use std::collections::HashMap;

use studyfrog_core::review::Rating;
use studyfrog_core::time::UnixMs;
use studyfrog_report::familiarity::{FamContext, FamEvent, fam_params_default, familiarity};
use studyfrog_report::simulate::{SimConfig, simulate_history};
use studyfrog_srs::params::params_default;

const DAY: i64 = 86_400_000;

/// Area under the ROC curve: the chance that a recalled review had a higher value than a forgotten one.
fn auc(points: &[(f32, bool)]) -> f64 {
    let (mut wins, mut pairs) = (0.0_f64, 0.0_f64);
    let lost: Vec<f32> = points.iter().filter(|p| !p.1).map(|p| p.0).collect();
    for (v, _) in points.iter().filter(|p| p.1) {
        for l in &lost {
            pairs += 1.0;
            wins += if v > l {
                1.0
            } else if v == l {
                0.5
            } else {
                0.0
            };
        }
    }
    wins / pairs
}

fn points_for(seed: u64) -> Vec<(f32, bool)> {
    let history = simulate_history(
        &SimConfig {
            seed,
            days: 200,
            new_per_day: 20,
            miss_percent: 10,
        },
        &params_default(),
    );
    let (p, ctx) = (fam_params_default(), FamContext::default());
    let mut seen: HashMap<u64, Vec<FamEvent>> = HashMap::new();
    let mut points = Vec::new();
    for r in &history {
        let at = UnixMs(r.day * DAY);
        let before = seen.entry(r.item).or_default();
        if let Some(f) = familiarity(before, at, &ctx, &p) {
            points.push((f.value, r.rating != Rating::Again));
        }
        before.push(FamEvent::Rating {
            at,
            rating: r.rating,
            answer_ms: None,
            expected_ms: None,
        });
    }
    points
}

#[test]
fn at_the_due_date_recall_does_not_depend_on_familiarity_because_fsrs_equalises_it() {
    let mut all = Vec::new();
    for seed in 1..=3 {
        all.extend(points_for(seed));
    }
    let overall = auc(&all);
    println!("all seeds: {} reviews, AUC {overall:.3}", all.len());
    assert!((0.45..0.55).contains(&overall), "AUC {overall}");
}

/// Items of a simulated learner on a fixed schedule, with their true probability of recall on day 200: the same rules as
/// `simulate_history`, kept here so the memory state can be read.
fn learner(seed: u64) -> Vec<(Vec<FamEvent>, f32)> {
    // Each item with its true probability of recall on day 200.
    use studyfrog_core::memory::MemState;
    use studyfrog_srs::fsrs::{initial_state, interval_days, next_state, retrievability};
    let p = params_default();
    let mut rng = seed;
    let mut next = move || {
        rng = rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        ((z ^ (z >> 31)) >> 40) as f32 / (1_u64 << 24) as f32
    };
    let rating = |ok: bool, roll: f32| match (ok, roll) {
        (false, _) => Rating::Again,
        (true, r) if r < 0.1 => Rating::Hard,
        (true, r) if r < 0.9 => Rating::Good,
        (true, _) => Rating::Easy,
    };
    let mut items: Vec<(MemState, i64, i64, Vec<FamEvent>)> = Vec::new();
    for d in 0..200_i64 {
        for it in &mut items {
            if it.2 > d {
                continue;
            }
            let r = retrievability(&it.0, (d - it.1) as f32, &p);
            let rt = rating(next() < r, next());
            it.0 = next_state(&it.0, rt, (d - it.1) as f32, &p);
            (it.1, it.2) = (d, d + i64::from(interval_days(&it.0, &p)));
            it.3.push(FamEvent::Rating {
                at: UnixMs(d * DAY),
                rating: rt,
                answer_ms: None,
                expected_ms: None,
            });
        }
        for _ in 0..20 {
            let rt = rating(true, next());
            let st = initial_state(rt, &p);
            let ev = FamEvent::Rating {
                at: UnixMs(d * DAY),
                rating: rt,
                answer_ms: None,
                expected_ms: None,
            };
            items.push((st, d, d + i64::from(interval_days(&st, &p)), vec![ev]));
        }
    }
    items
        .into_iter()
        .map(|(st, last, _, ev)| (ev, retrievability(&st, (200 - last) as f32, &p)))
        .collect()
}

/// Spearman rank correlation.
fn spearman(pairs: &[(f32, f32)]) -> f64 {
    fn ranks(v: &[f32]) -> Vec<f64> {
        let mut idx: Vec<usize> = (0..v.len()).collect();
        idx.sort_by(|a, b| v[*a].total_cmp(&v[*b]));
        let mut r = vec![0.0; v.len()];
        let mut i = 0;
        while i < idx.len() {
            let mut j = i;
            while j + 1 < idx.len() && v[idx[j + 1]] == v[idx[i]] {
                j += 1;
            }
            for k in i..=j {
                r[idx[k]] = (i + j) as f64 / 2.0;
            }
            i = j + 1;
        }
        r
    }
    let (a, b): (Vec<f32>, Vec<f32>) = pairs.iter().copied().unzip();
    let (ra, rb) = (ranks(&a), ranks(&b));
    let n = ra.len() as f64;
    let (ma, mb) = (ra.iter().sum::<f64>() / n, rb.iter().sum::<f64>() / n);
    let cov: f64 = ra.iter().zip(&rb).map(|(x, y)| (x - ma) * (y - mb)).sum();
    let (va, vb): (f64, f64) = (
        ra.iter().map(|x| (x - ma).powi(2)).sum(),
        rb.iter().map(|y| (y - mb).powi(2)).sum(),
    );
    cov / (va * vb).sqrt()
}

fn rho_with(gap_factor: f32) -> f64 {
    // Spearman correlation of familiarity with the true probability of recall on day 200.
    let p = studyfrog_report::familiarity::FamParams {
        gap_factor,
        ..fam_params_default()
    };
    let ctx = FamContext::default();
    let mut pairs = Vec::new();
    for seed in 1..=3 {
        for (events, recall) in learner(seed) {
            let f = familiarity(&events, UnixMs(200 * DAY), &ctx, &p).unwrap();
            pairs.push((f.value, recall));
        }
    }
    spearman(&pairs)
}

#[test]
fn familiarity_follows_the_true_chance_of_recalling_an_item_now() {
    let spec = rho_with(0.0);
    let aware: Vec<(f32, f64)> = [2.0, 4.0, 8.0].iter().map(|k| (*k, rho_with(*k))).collect();
    println!(
        "Spearman with the true recall probability: specification {spec:.3}, gap-aware {aware:?}"
    );
    // Measured 0.376 (three seeds, 12 000 items): a moderate rank correlation, recorded in docs/specs/reports.md.
    assert!(
        spec > 0.3,
        "familiarity ranks items by how ready they are: {spec}"
    );
    assert!(
        aware.iter().all(|(_, r)| (*r - spec).abs() < 0.05),
        "the gap-aware fall changes little on this learner: {aware:?}"
    );
}

#[test]
fn the_gap_option_changes_nothing_while_gaps_are_short_so_the_accepted_examples_stand() {
    let events: Vec<FamEvent> = [0, 2, 6, 15, 40, 43, 60, 110, 170, 260]
        .iter()
        .map(|d| FamEvent::Rating {
            at: UnixMs(d * DAY),
            rating: Rating::Good,
            answer_ms: None,
            expected_ms: None,
        })
        .collect();
    let ctx = FamContext::default();
    let off = fam_params_default();
    let on = studyfrog_report::familiarity::FamParams {
        gap_factor: 4.0,
        ..off
    };
    let at = UnixMs(350 * DAY);
    let (a, b) = (
        familiarity(&events, at, &ctx, &off).unwrap().value,
        familiarity(&events, at, &ctx, &on).unwrap().value,
    );
    assert!(
        (a - b).abs() < 1e-6,
        "{a} vs {b}: the longest gap is 90 days, four times it is under a year"
    );
}
