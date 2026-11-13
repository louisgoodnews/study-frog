---
id: STORY-0034
type: story
title: "Stack settings, SM-2, budgets and filter persistence"
status: done
epic: EPIC-018
parents: [EPIC-018]
estimate: 8
labels: [m3, service, store, md, core]
acceptance_criteria:
  - "`scheduler` (fsrs|sm2), `request_retention` and daily limits of a stack are parsed, indexed and used"
  - "Ratings go through the stack's scheduler; reviews are shared history"
  - "Spaced runs honour per-stack daily limits using today's review counts"
  - "The run filter round-trips through the database"
---

## Context
Owner request: take on the service gaps before M4.

## Description
Four tasks, ordered by dependency.

## Acceptance Criteria
- [x] `scheduler` (fsrs|sm2), `request_retention` and daily limits of a stack are parsed, indexed and used
- [x] Ratings go through the stack's scheduler; reviews are shared history
- [x] Spaced runs honour per-stack daily limits using today's review counts
- [x] The run filter round-trips through the database

## Design Notes
Children: TASK-0074, TASK-0075, TASK-0076, TASK-0077. Parent epic EPIC-018.

## Dependencies
M3 exit.

## Open Questions
None.
