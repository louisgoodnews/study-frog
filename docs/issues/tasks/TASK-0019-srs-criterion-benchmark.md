---
id: TASK-0019
type: task
title: "SRS criterion benchmark"
status: done
epic: EPIC-004
parents: [STORY-0008]
estimate: 2
labels: [m1, srs, perf]
acceptance_criteria:
  - "Bench: schedule 100k cards for FSRS and SM-2"
  - "Baseline saved; budget in BENCHMARKING.md updated"
---

## Context
Replaces the provisional 50 ms budget with a measurement.

## Description
SRS criterion benchmark.

## Acceptance Criteria
- [x] Bench: schedule 100k cards for FSRS and SM-2
- [x] Baseline saved; budget in BENCHMARKING.md updated

## Design Notes
criterion 0.8 in `crates/studyfrog-srs/benches/scheduling.rs`, deterministic LCG data, no rand dependency. Baseline `m1` saved locally (`--save-baseline m1`, lives in target/, not committed); numbers recorded in docs/contributing/BENCHMARKING.md.

## Dependencies
TASK-0016, TASK-0018.

## Open Questions
None.
