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
| `request_retention` | 0.90 | vault default in `studyfrog.toml`, per-stack override in `_stack.toml` |
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

## Choosing the algorithm
FSRS-6 is the default; SM-2 is selectable per stack (`scheduler = "sm2"` in `_stack.toml`) or for the whole
vault (`[srs] scheduler` in `studyfrog.toml`). Both write to the same `review` history; their state lives in
`mem_state` (FSRS) and `sm2_state` (SM-2). When a stack is switched, items keep their old due date until the
next review, which starts fresh state in the new algorithm. Custom FSRS weights, the retention default and the
maximum interval come from `studyfrog.toml` (see [config-files](config-files.md)).

## Familiarity
A familiarity value from 0.0 to 1.0 per item may scale the interval FSRS computes, as a factor
`exp(beta * (F - F_expected(S)))` that is 1 when familiarity is what the stability leads one to expect. The stored `MemState`
is never changed, `interval_days` stays as it is, and a strength of 0 gives the plain interval. It is on from the start (owner's decision, 2026-11-02, beta 2); a replay of the
user's own history reports whether it predicts better than plain FSRS and Settings can switch it off. Model, simulation and open decisions:
[familiarity-and-fsrs](../research/familiarity-and-fsrs.md) (TASK-0183, TASK-0141, TASK-0184).

Implemented (TASK-0141) in `studyfrog-srs::familiarity` (`familiarity_factor`, `interval_days_with`, `scaled_interval_days`, `before_deadline`) and called by
`studyfrog-service::familiarity` after each rating: the factor uses the item's familiarity ([familiarity](familiarity.md)) after the rating and the
stability FSRS just computed, the due date is the interval times the factor (never below one day, never above the maximum), and a stretched
interval is cut at the next exam or deadline of the item's stack or one above it, but never below the plain interval. SM-2 uses its interval as the
stability and keeps its stored interval, so its progression is unchanged. Switches: `familiarity_strength` in `studyfrog.toml` (default 2, 0 is off)
and `familiarity = false` in a `_stack.toml`. Not yet used: the time the answer was on screen and the expected reading time (the factor sees
ratings, views and the difficulty mark), the exam date for the fall of familiarity, and the re-rating replay, which recomputes plain FSRS dates.
