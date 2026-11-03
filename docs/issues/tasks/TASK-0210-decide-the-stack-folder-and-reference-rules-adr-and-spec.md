---
id: TASK-0210
type: task
title: Decide the stack folder and reference rules (ADR and spec)
status: done
epic: EPIC-026
parents: [STORY-0079]
estimate: 3
labels: [m11, docs, decision]
acceptance_criteria:
  - "An ADR and an update of docs/specs/markdown-schema.md and docs/specs/srs-algorithm.md state: a folder is a stack when it holds a stack file (`_stack.md`, as today) and a normal folder otherwise; an item's primary stack is the nearest stack folder above it, none meaning the Inbox; `referenced_by` lists the stacks (by id) that also reference the item"
  - "They state that memory, schedule, due date, familiarity and difficulty are independent per (item, stack), that they live in the database and never in the card's file, what the item's overall value is (the average over its references), and the run rules: a card is due if due in any of its stacks, a rating updates every selected stack that holds it (early reviews included) and none outside the selection"
  - "They state the shape in which the item page, the library and the exports show the per-stack values"
  - "They state what moving files, deleting a stack and removing the stack mark do to references and to the per-stack data, and what import and export (JSON, TOML, RON, CSV, Obsidian, Anki) do"
  - "The owner has accepted it"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0, and decided about stacks the same day: an item can be referenced by several stacks, held in a `referenced_by` field, with the original parent as the primary reference; a folder is a stack only when marked; a normal folder inside a stack folder belongs to the nearest stack above it; the vault is viewed like Obsidian's file explorer; each stack that references an item has its own independent memory of it (schedule, due date, familiarity, difficulty); the item's overall value is the average over its references; in a run over several stacks a card is due if it is due in any of them and a rating updates every selected stack that holds the card, also where it was not yet due (an early review is allowed); the per-stack values stay in the database and are never written into the card's file; the file holds only `referenced_by`; exports may be JSON, TOML or RON.

## Description
Write the rules down before changing the model.

## Acceptance Criteria
- [x] An ADR and an update of docs/specs/markdown-schema.md and docs/specs/srs-algorithm.md state: a folder is a stack when it holds a stack file (`_stack.md`, as today) and a normal folder otherwise; an item's primary stack is the nearest stack folder above it, none meaning the Inbox; `referenced_by` lists the stacks (by id) that also reference the item
- [x] They state that memory, schedule, due date, familiarity and difficulty are independent per (item, stack), that they live in the database and never in the card's file, what the item's overall value is (the average over its references), and the run rules: a card is due if due in any of its stacks, a rating updates every selected stack that holds it (early reviews included) and none outside the selection
- [x] They state the shape in which the item page, the library and the exports show the per-stack values
- [x] They state what moving files, deleting a stack and removing the stack mark do to references and to the per-stack data, and what import and export (JSON, TOML, RON, CSV, Obsidian, Anki) do
- [x] The owner has accepted it

## Design Notes
Written as [ADR 0011](../../adr/0011-stack-folders-and-references.md) with the updates of `markdown-schema.md` and `srs-algorithm.md`. The owner's decisions are the content; 'accepted' is marked as the owner's decisions of 2026-11-03 and the wording may still be amended by the owner.

Owner's decisions (2026-11-03): an item can be referenced by several stacks, the original parent is the primary reference; a folder is a stack only when marked; a normal folder inside a stack folder belongs to the nearest stack above it; memory is per stack and independent; the item's overall value is the average over its references; a card in a run over several stacks is due if it is due in any of them, and a rating updates every selected stack that holds it, none outside the selection, also where it was not yet due (the owner: reviewing a card scheduled for tomorrow today leaves it in mind today and tomorrow, so the effect is the same and an early review is fine); familiarity per stack comes from the reviews made in that stack; the per-stack values stay in the database and are not written into the card's file, which holds only `referenced_by`; exports can be JSON, TOML or RON. Shape shown by the item page, the library and the exports (the owner's, with `difficulty` instead of the first draft's `rating`): `{ source, difficulty, familiarity }` per reference, plus the due date and the stability, and the average. `difficulty` is the difficulty mark (easy, medium, difficult): the last mark given in that stack, else the item's own value; the average maps the marks to 1, 2 and 3 and shows the nearest word and the number. Other proposals: (1) the stack file is the mark, so old vaults are unchanged; (2) `referenced_by` is a list of stack ids (stable under renames and moves); (3) moving a file across a stack boundary changes its primary stack (the plan says how many items change) and the per-stack data stays with the stack; (4) a reference to a stack that is gone is a vault problem with one action to clean it; (5) CSV gets an optional `also_in` column, Obsidian folders map to normal folders unless marked, Anki decks map to stacks as today.

## Dependencies
None.

## Open Questions
None.
