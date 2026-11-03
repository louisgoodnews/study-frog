---
id: TASK-0210
type: task
title: Decide the stack folder and reference rules (ADR and spec)
status: backlog
epic: EPIC-026
parents: [STORY-0079]
estimate: 3
labels: [m11, docs, decision]
acceptance_criteria:
  - "An ADR and an update of docs/specs/markdown-schema.md and docs/specs/srs-algorithm.md state: a folder is a stack when it holds a stack file (`_stack.md`, as today) and a normal folder otherwise; an item's primary stack is the nearest stack folder above it, none meaning the Inbox; `referenced_by` lists the stacks (by id) that also reference the item"
  - "They state how schedule, due date, status, familiarity and the other values work per stack, what a review in one stack does to the others, what the item's average is, and what runs over several stacks do"
  - "They state what moving files, deleting a stack and removing the stack mark do to references and to the per-stack data, and what import and export (JSON, CSV, Obsidian, Anki) do"
  - "The owner has accepted it"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0, and decided about stacks the same day: an item can be referenced by several stacks, held in a `referenced_by` field, with the original parent as the primary reference; a folder can be marked as a stack folder, otherwise it is a normal folder; a normal folder inside a stack folder belongs to the nearest stack above it; the vault should be viewed and organized as in Obsidian's file explorer; and the schedule, the due date, the familiarity and other values of an item belong to each stack that references it, the item's overall value being the average over its references (Flashcard 100 in stack A with 5 weeks and in stack B with 3 weeks).

## Description
Write the rules down before changing the model.

## Acceptance Criteria
- [ ] An ADR and an update of docs/specs/markdown-schema.md and docs/specs/srs-algorithm.md state: a folder is a stack when it holds a stack file (`_stack.md`, as today) and a normal folder otherwise; an item's primary stack is the nearest stack folder above it, none meaning the Inbox; `referenced_by` lists the stacks (by id) that also reference the item
- [ ] They state how schedule, due date, status, familiarity and the other values work per stack, what a review in one stack does to the others, what the item's average is, and what runs over several stacks do
- [ ] They state what moving files, deleting a stack and removing the stack mark do to references and to the per-stack data, and what import and export (JSON, CSV, Obsidian, Anki) do
- [ ] The owner has accepted it

## Design Notes
Owner's decisions (2026-11-03): an item can be referenced by several stacks; `referenced_by` holds the references while the original parent is the primary reference; stacks are not folders per se, a folder can be marked as a stack folder, otherwise it is normal; a normal folder inside a stack folder belongs to the nearest stack above it; the schedule and the values are per stack with the item's overall value the average over its references. Proposals for the record: (1) the stack file is the mark, so old vaults are unchanged; (2) `referenced_by` is a list of stack ids in the item's front matter (stable under renames and moves; the UI shows names); (3) the memory state, the due date and the familiarity live per (item, stack) pair; an existing item's current state becomes its primary stack's; (4) moving a file across a stack boundary changes its primary stack (the plan says how many items change) and the per-stack data stays with the stack; (5) a reference to a stack that is gone is a vault problem with one action to clean it; (6) JSON export carries the references and per-stack states, CSV gets an optional `also_in` column; Obsidian folders map to normal folders unless the user marks them, Anki decks map to stacks as today.

## Dependencies
None.

## Open Questions
See the questions in the reply to the owner of 2026-11-03, recorded here when answered: (a) does rating a card in stack A also update stack B's schedule or only A's; (b) what exactly is averaged on the item (familiarity, difficulty, retrievability, all); (c) are priority, difficulty and status per stack too; (d) in a run over several stacks or over everything, which schedule decides that an item is due and which schedules does a rating update; (e) is familiarity per stack computed from the reviews made in that stack only or from all reviews of the item with the stack's own exam dates and settings.
