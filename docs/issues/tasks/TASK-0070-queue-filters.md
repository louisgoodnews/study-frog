---
id: TASK-0070
type: task
title: Queue filters
status: done
epic: EPIC-007
parents: [STORY-0033]
estimate: 3
labels: [m3, rehearsal]
acceptance_criteria:
  - "Filter by kind, stack, tags (any), state (new, learned, due, not due) and difficulty range"
  - "Empty list means no restriction; difficulty filter excludes items without memory state"
  - "proptest: filtered result is a subset and respects every predicate"
---

## Context
Filters apply before ordering and the length cap.

## Description
Queue filters.

## Acceptance Criteria
- [x] Filter by kind, stack, tags (any), state (new, learned, due, not due) and difficulty range
- [x] Empty list means no restriction; difficulty filter excludes items without memory state
- [x] proptest: filtered result is a subset and respects every predicate

## Design Notes
Difficulty needs an FSRS memory state; SM-2 items have none and are excluded by a difficulty filter (documented).

## Dependencies
TASK-0068.

## Open Questions
None.
