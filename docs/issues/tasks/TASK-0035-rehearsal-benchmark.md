---
id: TASK-0035
type: task
title: Rehearsal benchmark
status: backlog
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
- [ ] criterion: step transition < 10 us (budget), queue build for 10k due items
- [ ] Baseline recorded in BENCHMARKING.md; zero allocation per step where practical

## Design Notes
Counting allocator in the bench build.

## Dependencies
TASK-0031, TASK-0034.

## Open Questions
None.
