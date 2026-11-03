---
id: TASK-0218
type: task
title: Schedules, due dates and reviews per stack
status: backlog
epic: EPIC-026
parents: [STORY-0079]
estimate: 8
labels: [m11, store, srs]
acceptance_criteria:
  - "The memory state and the due date of an item are kept for each stack that holds or references it, using that stack's own scheduler, retention, limits and weights"
  - "A review is recorded with the stack it was made in; existing schedules and reviews become the primary stack's in a schema migration that can be checked on a copy"
  - "Queues, the calendar planner and the daily limits of a stack look only at that stack's schedule for its items"
  - "A test: a card in stack A (5 weeks) and stack B (3 weeks) is due on two different days and shows in each stack's queue and planner on its own day"
  - "What a rating does to the other stacks' schedules follows the answer to question (a) of TASK-0210"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0, and decided about stacks the same day: an item can be referenced by several stacks, held in a `referenced_by` field, with the original parent as the primary reference; a folder can be marked as a stack folder, otherwise it is a normal folder; a normal folder inside a stack folder belongs to the nearest stack above it; the vault should be viewed and organized as in Obsidian's file explorer; and the schedule, the due date, the familiarity and other values of an item belong to each stack that references it, the item's overall value being the average over its references (Flashcard 100 in stack A with 5 weeks and in stack B with 3 weeks).

## Description
The owner's example: Flashcard 100 is in stack A with a scheduling of 5 weeks and in stack B with one of 3 weeks.

## Acceptance Criteria
- [ ] The memory state and the due date of an item are kept for each stack that holds or references it, using that stack's own scheduler, retention, limits and weights
- [ ] A review is recorded with the stack it was made in; existing schedules and reviews become the primary stack's in a schema migration that can be checked on a copy
- [ ] Queues, the calendar planner and the daily limits of a stack look only at that stack's schedule for its items
- [ ] A test: a card in stack A (5 weeks) and stack B (3 weeks) is due on two different days and shows in each stack's queue and planner on its own day
- [ ] What a rating does to the other stacks' schedules follows the answer to question (a) of TASK-0210

## Design Notes
Today `schedule` and `reviews` are keyed by item. Key them by (item, stack): add a `stack` column, fill it with the primary stack (the Inbox, uid 0, for loose items), change `insert_review_timed`, the due-date reads, the queue builders and the planner. The pure scheduler functions in `studyfrog-srs` do not change; the settings come from the stack the state belongs to.

## Dependencies
TASK-0210, TASK-0211 and TASK-0212.

## Open Questions
None.
