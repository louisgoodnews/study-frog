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
  - "The memory state and the due date of an item are kept independently for each stack that holds or references it, using that stack's own scheduler, retention, limits and weights"
  - "A review is recorded for the stack it was made in; one rating in a run over several stacks writes one review for every selected stack that holds the card and none for other stacks"
  - "Existing schedules and reviews become the primary stack's in a schema migration that can be checked on a copy"
  - "A card in a run over several stacks counts as due when it is due in any of them and is queued once"
  - "Queues, the calendar planner and the daily limits of a stack look only at that stack's schedule for its items"
  - "A test: a card in stack A (5 weeks) and stack B (3 weeks) is due on two different days, shows in each stack's queue and planner on its own day, and rating it in a run of A alone leaves B's schedule unchanged"
  - "A run of both stacks updates both and a run of everything updates every stack that holds the card"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0, and decided about stacks the same day: an item can be referenced by several stacks, held in a `referenced_by` field, with the original parent as the primary reference; a folder can be marked as a stack folder, otherwise it is a normal folder; a normal folder inside a stack folder belongs to the nearest stack above it; the vault should be viewed and organized like Obsidian's file explorer; each stack that references an item has its own memory of it (schedule, due date, familiarity), independent of the others, so rating a card in stack A does not touch stack B; the item's overall value is the average over its references; in a run over several stacks a card is due if it is due in any of them and a rating updates every selected stack that holds the card, never a stack outside the selection.

## Description
The owner's example: Flashcard 100 is in stack A with a scheduling of 5 weeks and in stack B with one of 3 weeks, and each stack remembers it on its own.

## Acceptance Criteria
- [ ] The memory state and the due date of an item are kept independently for each stack that holds or references it, using that stack's own scheduler, retention, limits and weights
- [ ] A review is recorded for the stack it was made in; one rating in a run over several stacks writes one review for every selected stack that holds the card and none for other stacks
- [ ] Existing schedules and reviews become the primary stack's in a schema migration that can be checked on a copy
- [ ] A card in a run over several stacks counts as due when it is due in any of them and is queued once
- [ ] Queues, the calendar planner and the daily limits of a stack look only at that stack's schedule for its items
- [ ] A test: a card in stack A (5 weeks) and stack B (3 weeks) is due on two different days, shows in each stack's queue and planner on its own day, and rating it in a run of A alone leaves B's schedule unchanged
- [ ] A run of both stacks updates both and a run of everything updates every stack that holds the card

## Design Notes
Today `schedule` and `reviews` are keyed by item. Key them by (item, stack): add a `stack` column, fill it with the primary stack (the Inbox, uid 0, for loose items), change `insert_review_timed`, the due-date reads, the queue builders and the planner. The run knows its selection (`RunPlan` has the stacks); the step recorder writes a review per selected stack that holds the card. The pure scheduler functions in `studyfrog-srs` do not change; the settings come from the stack the state belongs to. Queue building for several stacks merges the candidates by item and takes the earliest due date among the selected stacks.

## Dependencies
TASK-0210, TASK-0211 and TASK-0212.

## Open Questions
None.
