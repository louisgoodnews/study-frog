---
id: STORY-0014
type: story
title: "Latency capture"
status: done
epic: EPIC-007
parents: [EPIC-007]
estimate: 5
labels: [m3, rehearsal, store]
acceptance_criteria:
  - "Injected monotonic clock; paused time excluded"
  - "Run events persisted per item in one transaction; crash recovery"
---

## Context
Latency is a core data source for reports (M7).

## Description
Latency capture.

## Acceptance Criteria
- [x] Injected monotonic clock; paused time excluded
- [x] Run events persisted per item in one transaction; crash recovery

## Design Notes
Children: TASK-0032, TASK-0033.

## Dependencies
Parent epic EPIC-007.

## Open Questions
None.
