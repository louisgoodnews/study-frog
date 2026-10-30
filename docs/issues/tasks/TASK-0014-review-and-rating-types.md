---
id: TASK-0014
type: task
title: Review and Rating types
status: done
epic: EPIC-003
parents: [STORY-0006]
estimate: 2
labels: [m1, core, srs]
acceptance_criteria:
  - "Rating enum Again/Hard/Good/Easy"
  - "Review record: item uid, rating, timestamp, elapsed days"
---

## Context
Input for srs.

## Description
Review and Rating types.

## Acceptance Criteria
- [x] Rating enum Again/Hard/Good/Easy
- [x] Review record: item uid, rating, timestamp, elapsed days

## Design Notes
Data only. `Rating` has explicit discriminants 1..4 (the persisted values) with free functions `rating_value` / `rating_from_value`. `Review.elapsed_days` is `f32` days since the previous review (0 for the first). FSRS memory state (`MemState`) is not here; it arrives with TASK-0016.

## Dependencies
TASK-0012.

## Open Questions
Resolved in TASK-0013: use `UnixMs(i64)` from `studyfrog-core::time`; no chrono/time in core.
