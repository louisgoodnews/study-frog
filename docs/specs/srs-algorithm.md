# SRS algorithm

## Choice: FSRS-6
Free Spaced Repetition Scheduler models memory with Difficulty, Stability and Retrievability
and schedules the next review for a target retention (default 0.90). Compared to SM-2 it
fits review logs better and lets the user trade workload against retention directly.

**Pinned revision: FSRS-6, 21 parameters (w0..w20).** Source: open-spaced-repetition
"The Algorithm" wiki, <https://github.com/open-spaced-repetition/awesome-fsrs/wiki/The-Algorithm>,
read 2026-10-30. Re-check against that page when implementing TASK-0016.

Default weights:
```
[0.212, 1.2931, 2.3065, 8.2956, 6.4133, 0.8334, 3.0194, 0.001, 1.8722, 0.1666, 0.796,
 1.4835, 0.0614, 0.2629, 1.6483, 0.6014, 1.8729, 0.5425, 0.0912, 0.0658, 0.1542]
```
Forgetting curve (trainable decay w20):
`R(t,S) = (1 + factor * t/S)^(-w20)`, `factor = 0.9^(-1/w20) - 1`, so `R(S,S) = 0.9`.
Interval for target retention r (derived by inverting R, not quoted from the wiki; verify
against the reference implementation): `I(r,S) = (S/factor) * (r^(-1/w20) - 1)`.
The remaining update equations (difficulty, stability after success/failure) are taken from the
wiki page during TASK-0016 and covered by golden vectors from the reference implementation.

## Parameters
| Name | Default | Notes |
|---|---|---|
| `request_retention` | 0.90 | per-stack override |
| `maximum_interval_days` | 36500 | |
| `weights` | FSRS-6 defaults above | later: optimiser fitting on user history (post-M7) |
| `enable_fuzz` | true | deterministic given a seed argument |

## Interface (free functions in `studyfrog-srs`; `MemState` lives in `studyfrog-core`)
```
initial_state(rating, &Params) -> MemState
next_state(&MemState, rating, elapsed_days: f32, &Params) -> MemState   // elapsed < 1.0: same-day rules
retrievability(&MemState, elapsed_days, &Params) -> f32
interval_exact(&MemState, &Params) -> f32          // days to reach request_retention
interval_days(&MemState, &Params) -> u32           // rounded, >= 1, <= maximum_interval_days
fuzz_interval(interval, unit /* [0,1) */, maximum) -> u32
```
No clock or RNG inside: elapsed time and the fuzz `unit` (derived from a seed by the caller) are
arguments. Learning steps (minutes-scale re-presentation inside a session) belong to the
rehearsal engine, not to these functions.

Equations implemented (FSRS-6): D0, linear damping with mean reversion, S0, same-day stability,
recall stability (hard penalty w15, easy bonus w16), forget stability (min of long-term and
`S / e^(w17 w18)`). Implemented in `difficulty.rs` and `stability.rs`.

## Baseline
SM-2 is implemented alongside as a baseline for tests and as a user-selectable simple mode.

## Verification
- proptest: stability >= minimum and difficulty in [1, 10] over any sequence; stability monotonic in
  rating; retrievability in [0, 1], decreasing, 0.9 at t = S; interval monotonic in stability; fuzz
  stays in its window.
- Golden test: py-fsrs `test_review_card` (13 ratings, default parameters, no fuzz) reproduced exactly.
- criterion bench: scheduling 100k cards.
