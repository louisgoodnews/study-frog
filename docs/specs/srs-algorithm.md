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
