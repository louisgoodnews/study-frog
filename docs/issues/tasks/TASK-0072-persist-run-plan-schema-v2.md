---
id: TASK-0072
type: task
title: Persist run plan (schema v2)
status: backlog
epic: EPIC-007
parents: [STORY-0033]
estimate: 2
labels: [m3, store]
acceptance_criteria:
  - "Migration v2 adds order, effect, seed, limit and requeue flag to `run`"
  - "Upgrades a v1 database in place"
  - "Filter is not persisted (the queue itself is)"
---

## Context
Demonstrates the first real schema upgrade.

## Description
Persist run plan (schema v2).

## Acceptance Criteria
- [ ] Migration v2 adds order, effect, seed, limit and requeue flag to `run`
- [ ] Upgrades a v1 database in place
- [ ] Filter is not persisted (the queue itself is)

## Design Notes
Columns on `run`, defaults match the spaced plan.

## Dependencies
TASK-0068.

## Open Questions
None.
