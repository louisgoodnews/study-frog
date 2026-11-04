---
id: TASK-0218
type: task
title: Schedules, due dates and reviews per stack
status: done
epic: EPIC-026
parents: [STORY-0079]
estimate: 8
labels: [m11, store, srs]
acceptance_criteria:
  - "The memory state and the due date of an item are kept independently for each stack that holds or references it, using that stack's own scheduler, retention, limits and weights"
  - "A review is recorded for the stack it was made in; one rating in a run over several stacks writes one review for every selected stack that holds the card, also where it was not yet due, and none for other stacks"
  - "Existing schedules and reviews become the primary stack's in a schema migration that can be checked on a copy"
  - "A card in a run over several stacks counts as due when it is due in any of them and is queued once"
  - "Queues, the calendar planner and the daily limits of a stack look only at that stack's schedule for its items"
  - "A test: a card in stack A (5 weeks) and stack B (3 weeks) is due on two different days, shows in each stack's queue and planner on its own day, and rating it in a run of A alone leaves B's schedule unchanged"
  - "A run of both stacks updates both and a run of everything updates every stack that holds the card; an early review in a stack is scheduled like any early review"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0, and decided about stacks the same day: an item can be referenced by several stacks, held in a `referenced_by` field, with the original parent as the primary reference; a folder is a stack only when marked; a normal folder inside a stack folder belongs to the nearest stack above it; the vault is viewed like Obsidian's file explorer; each stack that references an item has its own independent memory of it (schedule, due date, familiarity, difficulty); the item's overall value is the average over its references; in a run over several stacks a card is due if it is due in any of them and a rating updates every selected stack that holds the card, also where it was not yet due (an early review is allowed); the per-stack values stay in the database and are never written into the card's file; the file holds only `referenced_by`; exports may be JSON, TOML or RON.

## Description
The owner's example: Flashcard 100 is in stack A with a scheduling of 5 weeks and in stack B with one of 3 weeks, and each stack remembers it on its own.

## Acceptance Criteria
- [x] The memory state and the due date of an item are kept independently for each stack that holds or references it, using that stack's own scheduler, retention, limits and weights
- [x] A review is recorded for the stack it was made in; one rating in a run over several stacks writes one review for every selected stack that holds the card, also where it was not yet due, and none for other stacks
- [x] Existing schedules and reviews become the primary stack's in a schema migration that can be checked on a copy
- [x] A card in a run over several stacks counts as due when it is due in any of them and is queued once
- [x] Queues, the calendar planner and the daily limits of a stack look only at that stack's schedule for its items
- [x] A test: a card in stack A (5 weeks) and stack B (3 weeks) is due on two different days, shows in each stack's queue and planner on its own day, and rating it in a run of A alone leaves B's schedule unchanged
- [x] A run of both stacks updates both and a run of everything updates every stack that holds the card; an early review in a stack is scheduled like any early review

## Design Notes
Today `schedule` and `reviews` are keyed by item. Key them by (item, stack): add a `stack` column, fill it with the primary stack (the Inbox, uid 0, for loose items), change `insert_review_timed`, the due-date reads, the queue builders and the planner. The run knows its selection (`RunPlan` has the stacks); the step recorder writes a review per selected stack that holds the card. The pure scheduler functions in `studyfrog-srs` do not change (FSRS and SM-2 already take the time since the last review); the settings come from the stack the state belongs to. Queue building for several stacks merges the candidates by item and takes the earliest due date among the selected stacks. Difficulty marks given in a run are attributed to the same stacks as the rating.

## Dependencies
TASK-0210, TASK-0211 and TASK-0212.

## Result (2026-11-04)
Schema v21 keys `review`, `mem_state` and `sm2_state` by (item, stack) and v22 adds `rating_id`, so a rating written for several stacks is one row each and `review_once` shows it once. The migration is tested on a database at v20 (`tests/memory_migration.rs`). Decisions the criteria left open: (1) an item that one selected stack has never scheduled counts as new in a run over several stacks; (2) a card is counted against the daily budget of the selected stack that makes it due; (3) screens with one value per item (library, item page, reports' item lists) still read the primary stack through the views `mem_primary`, `sm2_primary` and `review_once` until TASK-0219; (4) moving a planned review moves one stack's due date and the calendar entry names its stack; (5) rerating rebuilds every stack's state from that stack's own reviews. Queue merge cost: 3.8 ms to merge 10k items held by two stacks each (release build, `tests/queue_merge_timing.rs`, run by hand); the queue builder itself is unchanged (82 us to 3.9 ms).

## Open Questions
None.
