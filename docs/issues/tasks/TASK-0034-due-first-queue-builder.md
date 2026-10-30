---
id: TASK-0034
type: task
title: Due-first queue builder
status: backlog
epic: EPIC-007
parents: [STORY-0015]
estimate: 3
labels: [m3, rehearsal]
acceptance_criteria:
  - "Pure builder: due items first (most overdue first), then new items up to the stack's daily limits"
  - "Deterministic given a seed for any shuffling"
  - "proptest: respects limits, no duplicates"
---

## Context
Inputs: due list and new list as plain data.

## Description
Due-first queue builder.

## Acceptance Criteria
- [ ] Pure builder: due items first (most overdue first), then new items up to the stack's daily limits
- [ ] Deterministic given a seed for any shuffling
- [ ] proptest: respects limits, no duplicates

## Design Notes
SoA vs AoS decision for the event log is taken here with data (EPIC-003 open question).

## Dependencies
TASK-0033.

## Open Questions
None.
