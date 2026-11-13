---
id: TASK-0035
type: task
title: "Rehearsal benchmark"
status: done
epic: EPIC-007
parents: [STORY-0015]
estimate: 2
labels: [m3, rehearsal, perf]
acceptance_criteria:
  - "criterion: step transition < 10 us (budget), queue build for 10k due items"
  - "Baseline recorded in BENCHMARKING.md; zero allocation per step where practical"
---

## Context
Replaces the provisional budget with a measurement.

## Description
Rehearsal benchmark.

## Acceptance Criteria
- [x] criterion: step transition < 10 us (budget), queue build for 10k due items
- [x] Baseline recorded in BENCHMARKING.md; zero allocation per step where practical

## Design Notes
criterion bench in `crates/studyfrog-rehearsal/benches/rehearsal.rs`. Allocation counting is not done: a counting global allocator needs `unsafe` (forbidden workspace-wide); `dhat` would be the route if a profile ever shows allocator pressure. Measured: 23 ns per step; queue build for 10k candidates: spaced 337 us, shuffled 82 us, hardest-first 1.08 ms, lowest-retrievability 3.9 ms (a `powf` per comparison; cache keys if it ever matters).

## Dependencies
TASK-0031, TASK-0034.

## Open Questions
None.
