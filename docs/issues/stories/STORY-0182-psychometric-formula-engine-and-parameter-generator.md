---
id: STORY-0182
type: story
title: "Psychometric Formula Engine and Dynamic Parameter Generator"
status: done
epic: EPIC-062
parents: ['EPIC-062']
estimate: 3
labels: [m14, rehearse]
acceptance_criteria:
  - 'Pure Rust implementations of core psychometric formulas (se, confidence intervals, standard score conversions)'
  - 'Synthetic parameter generator producing statistically plausible problem instances with deterministic seeds'
  - 'Round-trip precision tests verifying zero drift and accurate floating-point tolerance bounds'
---

## Context
Psychometric drills need an exact, pure mathematical engine that can evaluate textbook equations and generate randomized problem sets on demand.

## Description
Develop the formula engine in `studyfrog-rehearsal`:
- Implement core psychometric functions:
  - `standard_error_of_measurement(s_x: f64, r_xx: f64) -> f64`
  - `confidence_interval(mean: f64, z: f64, se: f64) -> (f64, f64)`
  - `standard_score_transform(raw: f64, mean: f64, sd: f64, target_mean: f64, target_sd: f64) -> f64`
  - `spearman_brown_prophecy(r_old: f64, k: f64) -> f64`
- Implement a seeded parameter generator `generate_formula_exercise` creating random variations bounded by plausible ranges (e.g. $r_{xx} \in [0.65, 0.98]$, $s_x \in [10, 15]$).

## Acceptance Criteria
- [x] Pure Rust implementations of core psychometric formulas (se, confidence intervals, standard score conversions)
- [x] Synthetic parameter generator producing statistically plausible problem instances with deterministic seeds
- [x] Round-trip precision tests verifying zero drift and accurate floating-point tolerance bounds

## Design Notes
Keep all types pure with zero inherent `impl` blocks (Invariant P1). Validate inputs against edge conditions ($r_{xx} > 1.0$, negative standard deviations).

## Dependencies
EPIC-062

## Implementation (2026-11-21)
`studyfrog-rehearsal::psychometric` (SE, reliability round trip, confidence intervals, standard scores, Spearman-Brown, z-values) and `studyfrog-rehearsal::formula_gen` (seeded exercises via `rng`). Tests: `tests/psychometric.rs`, `tests/formula.rs`. The exercise screen is STORY-0184.

## Open Questions
None.
