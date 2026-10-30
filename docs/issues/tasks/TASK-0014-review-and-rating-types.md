---
id: TASK-0014
type: task
title: Review and Rating types
status: ready
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
- [ ] Rating enum Again/Hard/Good/Easy
- [ ] Review record: item uid, rating, timestamp, elapsed days

## Design Notes
Data only.

## Dependencies
TASK-0012.

## Open Questions
Resolved in TASK-0013: use `UnixMs(i64)` from `studyfrog-core::time`; no chrono/time in core.
