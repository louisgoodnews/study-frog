---
id: TASK-0032
type: task
title: Injected monotonic clock
status: backlog
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
- [ ] `Clock` trait (the one permitted seam) with a system implementation and a fake
- [ ] Latency = event time - ItemShown - paused time within the item
- [ ] Pause/resume property tests

## Design Notes
Monotonic ms since run start as u32; use std::time::Instant at the edge.

## Dependencies
TASK-0031.

## Open Questions
None.
