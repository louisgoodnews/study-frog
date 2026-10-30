---
id: TASK-0074
type: task
title: Persist the run filter
status: done
epic: EPIC-018
parents: [STORY-0034]
estimate: 2
labels: [m3, store]
acceptance_criteria:
  - "Migration v3 stores the filter (kinds, stacks, tags, state, difficulty range)"
  - "`load_run` returns the exact plan that was stored"
  - "Persisted names for kind and state filters live in core"
---

## Context
The plan must be reproducible from the database.

## Description
Persist the run filter.

## Acceptance Criteria
- [x] Migration v3 stores the filter (kinds, stacks, tags, state, difficulty range)
- [x] `load_run` returns the exact plan that was stored
- [x] Persisted names for kind and state filters live in core

## Design Notes
Migration v3: `filter_state`, `filter_diff_min/max` on `run`, ordered list tables for kinds, stacks and tags (a position column keeps order; tags may contain any character). Tested with a full filter (order, non-ASCII, commas, tabs).

## Dependencies
TASK-0072.

## Open Questions
None.
