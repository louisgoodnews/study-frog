---
id: TASK-0280
type: task
title: "Pure regression rule with a bootstrap interval, and its unit tests"
status: done
epic: EPIC-067
parents: ['STORY-0202']
estimate: 3
labels: [m14, perf, tooling]
acceptance_criteria:
  - 'bench_compare implements the rule in EPIC-067 Decisions: a seeded bootstrap 95% interval of median(new)/median(base), and a floor of max(5%, 3 x the baseline spread)'
  - 'Unit tests on synthetic runs with known outcomes pass: identical runs, 20% slower, 3% slower (below floor), noisy baseline, 20% faster, a single outlier, determinism, spread and repeat rule, parsing'
---

## Context
Part of STORY-0202. Standard library only: a bootstrap of 2000 resamples is about 40 lines, which fits better
than a statistics crate for this one comparison (P10).

## Description
`tools/repo-lint/src/bench_compare.rs` (rule, 195 lines) and `bench_compare/tests.rs` (tests, the file kept under 250 lines).

## Acceptance Criteria
- [x] The rule of EPIC-067 Decisions is implemented as pure functions
- [x] Unit tests with known outcomes pass (12 tests in repo-lint)

## Design Notes
- Regression: the interval's lower end is above 1.0 and the effect is above the floor. Improvement is the mirror.
- The floor for a noisy baseline is large: a test with a baseline spreading 48% requires about 146% to flag anything.
- The interval is resampled over the runs, so with 5 runs it is coarse. `more_runs_needed` reports that.

## Dependencies
STORY-0202.

## Open Questions
- The 5% floor is provisional (EPIC-067).
