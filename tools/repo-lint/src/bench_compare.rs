//! Regression rule for benchmark runs (EPIC-067, docs/contributing/BENCHMARKING.md).
//!
//! Input: one run per line, the run's median in nanoseconds (`#` lines and blank lines are skipped).
//! A regression is flagged only when both hold: the bootstrap 95% interval of the new/baseline ratio
//! of medians excludes 1.0 on the slower side, and the effect exceeds max(5%, 3 x the baseline's own
//! run-to-run spread). The 5% floor is provisional (EPIC-067 Decisions). Improvements are the mirror
//! image and are reported, never failed. Standard library only.

use std::path::Path;

/// Provisional effect-size floor (EPIC-067 Decisions).
pub const FLOOR_MIN: f64 = 0.05;
/// The floor is at least this many times the baseline's run-to-run spread.
pub const SPREAD_FACTOR: f64 = 3.0;
/// A side whose runs spread more than this is repeated up to `TEN_RUNS`.
pub const REPEAT_SPREAD: f64 = 0.05;
/// Runs per side when the spread is above `REPEAT_SPREAD`.
pub const TEN_RUNS: usize = 10;

const RESAMPLES: usize = 2000;
const SEED: u64 = 0x5EED_F00D;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Regression,
    Improvement,
    NoChange,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Comparison {
    pub ratio: f64,
    pub ci_low: f64,
    pub ci_high: f64,
    pub base_spread: f64,
    pub floor: f64,
    pub base_runs: usize,
    pub new_runs: usize,
    /// Either side should be repeated to `TEN_RUNS` before the verdict is trusted.
    pub more_runs_needed: bool,
    pub verdict: Verdict,
}

/// Parses one median per line; rejects anything that is not a positive, finite time.
pub fn parse_samples(text: &str) -> Result<Vec<f64>, String> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(parse_one)
        .collect()
}

fn parse_one(line: &str) -> Result<f64, String> {
    let v: f64 = line
        .parse()
        .map_err(|_| format!("not a number: `{line}`"))?;
    if v.is_finite() && v > 0.0 {
        Ok(v)
    } else {
        Err(format!("not a positive time: `{line}`"))
    }
}

/// Median of the values; NaN for an empty slice.
pub fn median(xs: &[f64]) -> f64 {
    let mut v = xs.to_vec();
    v.sort_by(f64::total_cmp);
    let n = v.len();
    match n {
        0 => f64::NAN,
        _ if n % 2 == 1 => v[n / 2],
        _ => (v[n / 2 - 1] + v[n / 2]) / 2.0,
    }
}

/// Run-to-run spread: (max - min) / mean. The same definition as `scripts/bench-spread.sh`.
pub fn spread(xs: &[f64]) -> f64 {
    let mean = xs.iter().sum::<f64>() / xs.len() as f64;
    let (lo, hi) = xs
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &x| {
            (lo.min(x), hi.max(x))
        });
    (hi - lo) / mean
}

/// True when a side with fewer than `TEN_RUNS` runs has spread above `REPEAT_SPREAD`.
pub fn needs_ten_runs(samples: &[f64]) -> bool {
    samples.len() < TEN_RUNS && spread(samples) > REPEAT_SPREAD
}

/// Bootstrap 95% interval of median(new) / median(base), each side resampled with replacement.
/// Seeded, so the same input always gives the same interval.
pub fn bootstrap_ci(base: &[f64], new: &[f64]) -> (f64, f64) {
    let mut state = SEED;
    let mut ratios: Vec<f64> = (0..RESAMPLES)
        .map(|_| {
            let b = resample(base, &mut state);
            let n = resample(new, &mut state);
            median(&n) / median(&b)
        })
        .collect();
    ratios.sort_by(f64::total_cmp);
    (percentile(&ratios, 0.025), percentile(&ratios, 0.975))
}

fn resample(xs: &[f64], state: &mut u64) -> Vec<f64> {
    (0..xs.len())
        .map(|_| xs[next_index(state, xs.len())])
        .collect()
}

/// xorshift64* step, returning an index below `len`. `state` must not be zero.
fn next_index(state: &mut u64, len: usize) -> usize {
    *state ^= *state >> 12;
    *state ^= *state << 25;
    *state ^= *state >> 27;
    let r = state.wrapping_mul(0x2545_F491_4F6C_DD1D);
    (r % len as u64) as usize
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    let idx = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted[idx]
}

/// The decision, given the ratio, its interval and the effect-size floor.
pub fn verdict_of(ratio: f64, ci_low: f64, ci_high: f64, floor: f64) -> Verdict {
    if ci_low > 1.0 && ratio - 1.0 > floor {
        Verdict::Regression
    } else if ci_high < 1.0 && 1.0 - ratio > floor {
        Verdict::Improvement
    } else {
        Verdict::NoChange
    }
}

/// Compares a new side against its baseline. Each side is the medians of its runs, in ns.
pub fn compare(base: &[f64], new: &[f64]) -> Result<Comparison, String> {
    if base.is_empty() || new.is_empty() {
        return Err("each side needs at least one run".to_string());
    }
    let ratio = median(new) / median(base);
    let (ci_low, ci_high) = bootstrap_ci(base, new);
    let base_spread = spread(base);
    let floor = FLOOR_MIN.max(SPREAD_FACTOR * base_spread);
    Ok(Comparison {
        ratio,
        ci_low,
        ci_high,
        base_spread,
        floor,
        base_runs: base.len(),
        new_runs: new.len(),
        more_runs_needed: needs_ten_runs(base) || needs_ten_runs(new),
        verdict: verdict_of(ratio, ci_low, ci_high, floor),
    })
}

pub fn verdict_label(v: Verdict) -> &'static str {
    match v {
        Verdict::Regression => "regression",
        Verdict::Improvement => "improvement",
        Verdict::NoChange => "no-change",
    }
}

/// One line for humans and for the release notes.
pub fn format_line(c: &Comparison) -> String {
    format!(
        "verdict={} ratio={:.3} ci95=[{:.3},{:.3}] effect={:+.1}% floor={:.1}% base_spread={:.1}% runs={}/{} more_runs_needed={}",
        verdict_label(c.verdict),
        c.ratio,
        c.ci_low,
        c.ci_high,
        (c.ratio - 1.0) * 100.0,
        c.floor * 100.0,
        c.base_spread * 100.0,
        c.base_runs,
        c.new_runs,
        if c.more_runs_needed { "yes" } else { "no" },
    )
}

/// Reads two run files and compares them.
pub fn run(base: &Path, new: &Path) -> Result<Comparison, String> {
    let read = |p: &Path| -> Result<Vec<f64>, String> {
        let text = std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))?;
        parse_samples(&text).map_err(|e| format!("{}: {e}", p.display()))
    };
    compare(&read(base)?, &read(new)?)
}

#[cfg(test)]
mod tests;
