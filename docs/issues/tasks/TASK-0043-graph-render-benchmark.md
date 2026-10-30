---
id: TASK-0043
type: task
title: Graph render benchmark
status: backlog
epic: EPIC-009
parents: [STORY-0019]
estimate: 3
labels: [m4, ui, perf]
acceptance_criteria:
  - "5k nodes at 60 fps measured"
  - "Zero allocation per frame, verified with a heap profiler (dhat)"
---

## Context
Replaces the provisional graph budget.

## Description
Graph render benchmark.

## Acceptance Criteria
- [ ] 5k nodes at 60 fps measured
- [ ] Zero allocation per frame, verified with a heap profiler (dhat)

## Design Notes
Record baselines in BENCHMARKING.md.

## Dependencies
TASK-0042.

## Open Questions
None.
