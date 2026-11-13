---
id: STORY-0033
type: story
title: "Run plans, orders and filters"
status: done
epic: EPIC-007
parents: [EPIC-007]
estimate: 5
labels: [m3, core, rehearsal]
acceptance_criteria:
  - "A run is built from a plan: order, filter, limit, requeue, schedule effect"
  - "Orders: due-first, sequential, shuffled (seeded), hardest/easiest first, stalest first, lowest predicted recall"
  - "Filters: kind, stack, tags, state (new/learned/due/not due), difficulty range"
  - "The plan is persisted with the run"
---

## Context
Rehearsal is not only spaced repetition: users also run through every item once, shuffle, or drill the hard ones (owner requirement, 2026-10-30).

## Description
Run plan types in core, queue builders in rehearsal, persistence in store.

## Acceptance Criteria
- [x] A run is built from a plan: order, filter, limit, requeue, schedule effect
- [x] Orders: due-first, sequential, shuffled (seeded), hardest/easiest first, stalest first, lowest predicted recall
- [x] Filters: kind, stack, tags, state (new/learned/due/not due), difficulty range
- [x] The plan is persisted with the run

## Design Notes
Children: TASK-0068, TASK-0069, TASK-0070, TASK-0072. Parent epic EPIC-007.

## Dependencies
Needs TASK-0015 (run types) and the SRS functions.

## Open Questions
None.
