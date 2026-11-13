---
id: TASK-0212
type: task
title: "Services follow the membership"
status: done
epic: EPIC-026
parents: [STORY-0079]
estimate: 8
labels: [m11, store]
acceptance_criteria:
  - "Queues, the stack summary, the roadmap, reports, the calendar planner, bulk edits and import and export use the membership: a referenced item belongs to every stack that references it and is queued once in a run"
  - "For vaults without references every result is the same as before, checked by a test on the same vault"
  - "The library's stack column shows the primary stack and an 'Also in' column lists the others"
  - "JSON export and import carry the references and CSV has the optional `also_in` column"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0, and decided about stacks the same day: an item can be referenced by several stacks, held in a `referenced_by` field, with the original parent as the primary reference; a folder can be marked as a stack folder, otherwise it is a normal folder; a normal folder inside a stack folder belongs to the nearest stack above it; the vault should be viewed and organized as in Obsidian's file explorer; and the schedule, the due date, the familiarity and other values of an item belong to each stack that references it, the item's overall value being the average over its references (Flashcard 100 in stack A with 5 weeks and in stack B with 3 weeks).

## Description
Everything that asks which items are in a stack asks the folder today; it must ask the membership.

## Acceptance Criteria
- [x] Queues, the stack summary, the roadmap, reports, the calendar planner, bulk edits and import and export use the membership: a referenced item belongs to every stack that references it and is queued once in a run
- [x] For vaults without references every result is the same as before, checked by a test on the same vault
- [x] The library's stack column shows the primary stack and an 'Also in' column lists the others
- [x] JSON export and import carry the references and CSV has the optional `also_in` column

## Design Notes
Done: `ItemMeta.also_in`, `IndexRow.also_in`, `IndexedItem.also_in` and `Candidate.also_in` (from the view `membership`); `in_stack` / `row_in_stack` / `in_scored` replace the folder-prefix checks in the stack summary, the sub-stack tree, the reports explorer, the familiarity report, the assistant's context and exam; the run filter admits an item through any of its stacks and a queue holds it once; the library has an Also in column; JSON, CSV and import carry `also_in` (`xfer_refs.rs`). Left to TASK-0218: what a stack's queue and the planner do with a referenced item's own schedule (until then they use the item's one schedule, attributed to its primary stack). The query language has no `also_in` field yet.

Replace the folder-prefix checks (`stack_of`, `inside`) in the queue builders, the summary, the roadmap, the reports and the planner by the membership query of TASK-0211; the folder stays only the item's place on disk. What a stack's queue does with the stack's own schedule comes with TASK-0218.

## Dependencies
TASK-0211.

## Open Questions
None.
