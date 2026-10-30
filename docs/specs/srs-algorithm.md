# SRS algorithm

## Choice: FSRS
Free Spaced Repetition Scheduler models memory with Difficulty, Stability and Retrievability
and schedules the next review for a target retention (default 0.90). Compared to SM-2 it
fits review logs better and lets the user trade workload against retention directly.
Use the current published FSRS revision (weights vector of 19+ parameters, pinned in code with
the version recorded in this file at M1).

## Parameters
| Name | Default | Notes |
|---|---|---|
| `request_retention` | 0.90 | per-stack override |
| `maximum_interval_days` | 36500 | |
| `weights` | published defaults | later: optimiser fitting on user history (post-M7) |
| `enable_fuzz` | true | deterministic given a seed argument |

## Interface (free functions in `studyfrog-srs`)
```
next_state(card: &MemState, rating: Rating, elapsed_days: f32, p: &Params) -> MemState
interval_days(state: &MemState, p: &Params) -> f32
retrievability(state: &MemState, elapsed_days: f32) -> f32
```
No clock or RNG inside: elapsed time and fuzz seed are arguments.

## Baseline
SM-2 is implemented alongside as a baseline for tests and as a user-selectable simple mode.

## Verification
- proptest: intervals are monotonic in rating; stability never negative; retrievability in
  [0, 1] and decreasing in elapsed time.
- Reference vectors from the published implementation as golden tests.
- criterion bench: scheduling 100k cards.
