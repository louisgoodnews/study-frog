---
id: TASK-0282
type: task
title: "Release check: compare the reference run with the saved baseline before tagging"
status: review
epic: EPIC-067
parents: ['STORY-0202']
estimate: 1
labels: [m14, perf, docs]
acceptance_criteria:
  - 'The release section of BENCHMARKING.md lists the steps: reference run on the owner''s machine, bench-compare against the saved baseline for each case, and the verdict lines in the release notes'
  - 'A regression verdict blocks the release until it is either fixed or accepted in writing by the owner'
---

## Context
Part of STORY-0202: "a change that slows a hot path past its budget or its baseline is caught before release."

## Description
Document the step and where its output goes. The step is run by the owner, on the owner's machine, before tagging.

## Acceptance Criteria
- [x] The release steps are in BENCHMARKING.md (section "Release 1.0.0 check")
- [ ] A regression verdict blocks the release unless the owner accepts it in writing

## Run (2026-11-24, reference machine)
- `cargo bench --workspace` and a second reference run of the same code (commit 76461b6) were compared per case with
  `scripts/bench-compare.sh` against `docs/benchmarks/reference-1.0.0/`. Verdict: 16 of 28 cases `regression`, 12 `no-change`.
  The code did not change between runs, and the machine state did (one busy process against fourteen). The criterion
  run of `cargo bench --workspace` agrees with the first run, so the second script run is the outlier.
- Per the criterion above the regression verdict blocks the tag until the owner accepts it in writing or a repeat run
  clears it. The task stays in `review`; the owner's decision is pending.

## Design Notes
- BENCHMARKING.md's budget numbers are not changed here (STORY-0203).

## Dependencies
TASK-0278 and TASK-0279 (runs on the owner's machine), STORY-0203 (baselines).

## Open Questions
- Whether the release checklist lives in BENCHMARKING.md or in the release process document.
