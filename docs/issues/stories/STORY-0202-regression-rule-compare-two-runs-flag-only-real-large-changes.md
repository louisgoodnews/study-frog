---
id: STORY-0202
type: story
title: "Regression rule: compare two runs and flag only real, large changes"
status: in_progress
epic: EPIC-067
parents: ['EPIC-067']
estimate: 5
labels: [m14, perf, tooling]
acceptance_criteria:
  - 'The comparison implements the EPIC-067 rule: the bootstrap 95% interval of the new/baseline ratio of medians excludes 1.0, and the effect exceeds max(5%, 3 times the baseline''s spread)'
  - 'Unit tests on synthetic runs with a known outcome pass (regression, no change under the floor, noisy baseline, improvement, a single outlier)'
  - 'The verdict and its numbers print on one line; the exit code is 1 only for a regression'
  - 'Before a release, the owner''s reference run is compared with the saved baseline, and the lines go in the release notes'
---

## Context
EPIC-067 asks that a regression shows as a number and is caught before release. Without a rule, a noisy
run either hides a real slowdown or raises a false alarm on every run.

## Description
- `repo-lint bench-compare` (`tools/repo-lint/src/bench_compare.rs`, std only, no new crate) implements the rule:
  a seeded bootstrap of 2000 resamples per side, the 2.5 and 97.5 percentiles of the ratio of medians, and the
  floor from the baseline's spread.
- `scripts/bench-compare.sh <baseline.ns> <new.ns>` is the command for the owner.
- The release check that uses the comparison is a separate task, because it needs the owner's runs.

## Acceptance Criteria
- [x] The comparison implements the rule in EPIC-067 Decisions (TASK-0280)
- [x] Unit tests on synthetic runs with a known outcome pass (TASK-0280)
- [x] The verdict prints on one line; exit code 1 only for a regression (TASK-0281)
- [ ] The release check compares the reference run with the saved baseline (TASK-0282)

## Design Notes
- The floor uses the baseline's spread, so a noisy baseline needs a larger change to be flagged. A test
  shows that a 20% slowdown is not flagged against a baseline that spreads 48%.
- With 5 runs the interval is coarse. The line reports `more_runs_needed=yes` when a side should be repeated to 10.
- Improvements are reported, never failed.

## Dependencies
TASK-0280, TASK-0281, TASK-0282. TASK-0282 needs the owner's reference runs (STORY-0201).

## Open Questions
- The 5% floor is provisional (EPIC-067 Decisions). It is changed only with evidence from the owner's spread measurement.
