---
id: TASK-0032
type: task
title: Injected monotonic clock
status: done
epic: EPIC-007
parents: [STORY-0014]
estimate: 3
labels: [m3, rehearsal]
acceptance_criteria:
  - "`Clock` trait (the one permitted seam) with a system implementation and a fake"
  - "Latency = event time - ItemShown - paused time within the item"
  - "Pause/resume property tests"
---

## Context
Spec: latency capture.

## Description
Injected monotonic clock.

## Acceptance Criteria
- [x] `Clock` trait (the one permitted seam) with a system implementation and a fake
- [x] Latency = event time - ItemShown - paused time within the item
- [x] Pause/resume property tests

## Design Notes
`Clock` trait (system and manual implementations; `input_now` stamps an action). Latency = t - shown_at - paused time within the item; tested with several pauses, a pause before rating, and no leakage into the next item. Time going backwards is refused.

## Dependencies
TASK-0031.

## Open Questions
None.
