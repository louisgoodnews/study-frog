---
id: TASK-0072
type: task
title: Persist run plan (schema v2)
status: done
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
- [x] Migration v2 adds order, effect, seed, limit and requeue flag to `run`
- [x] Upgrades a v1 database in place
- [x] Filter is not persisted (the queue itself is)

## Design Notes
Migration v2 adds `order_name`, `effect`, `seed`, `run_limit`, `requeue_again` (with CHECK constraints; `order` is a reserved word, hence `order_name`). Defaults equal the spaced plan. Tested: a v1 database with a run row upgrades in place. The filter is not stored; `load_run` returns an empty filter.

## Dependencies
TASK-0068.

## Open Questions
None.
