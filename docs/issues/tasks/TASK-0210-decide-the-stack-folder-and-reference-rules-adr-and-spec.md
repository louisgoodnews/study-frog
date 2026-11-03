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
  - "They state that memory, schedule, due date and familiarity are independent per (item, stack), what the item's overall value is (the average over its references), and the run rules: a card is due if due in any of its stacks, and a rating updates every selected stack that holds it"
  - "They state where the per-stack values live (database, file or both) and the shape in which the item page and the exports show them"
  - "They state what moving files, deleting a stack and removing the stack mark do to references and to the per-stack data, and what import and export (JSON, CSV, Obsidian, Anki) do"
  - "The owner has accepted it"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0, and decided about stacks the same day: an item can be referenced by several stacks, held in a `referenced_by` field, with the original parent as the primary reference; a folder can be marked as a stack folder, otherwise it is a normal folder; a normal folder inside a stack folder belongs to the nearest stack above it; the vault should be viewed and organized like Obsidian's file explorer; each stack that references an item has its own memory of it (schedule, due date, familiarity), independent of the others, so rating a card in stack A does not touch stack B; the item's overall value is the average over its references; in a run over several stacks a card is due if it is due in any of them and a rating updates every selected stack that holds the card, never a stack outside the selection.

## Description
Write the rules down before changing the model.

## Acceptance Criteria
- [ ] An ADR and an update of docs/specs/markdown-schema.md and docs/specs/srs-algorithm.md state: a folder is a stack when it holds a stack file (`_stack.md`, as today) and a normal folder otherwise; an item's primary stack is the nearest stack folder above it, none meaning the Inbox; `referenced_by` lists the stacks (by id) that also reference the item
- [ ] They state that memory, schedule, due date and familiarity are independent per (item, stack), what the item's overall value is (the average over its references), and the run rules: a card is due if due in any of its stacks, and a rating updates every selected stack that holds it
- [ ] They state where the per-stack values live (database, file or both) and the shape in which the item page and the exports show them
- [ ] They state what moving files, deleting a stack and removing the stack mark do to references and to the per-stack data, and what import and export (JSON, CSV, Obsidian, Anki) do
- [ ] The owner has accepted it

## Design Notes
Owner's decisions (2026-11-03): an item can be referenced by several stacks, the original parent is the primary reference; a folder is a stack only when marked; a normal folder inside a stack folder belongs to the nearest stack above it; memory is per stack and independent (rating a card in stack A does not change stack B); the item's overall value is the average over its references; a run over several stacks treats a card as due if it is due in any of its stacks, and a rating updates every selected stack that holds the card (so one stack when one stack was chosen, all of them when the card is in all selected stacks), never a stack outside the selection; familiarity per stack comes from the reviews made in that stack. The owner's picture of the stored values: the card's file holds a list like `[{"source": "stack_a", "rating": "easy", "familiarity": 0.755}]`, one entry per reference, which can be averaged, also kept for the card alone. Concern for the record: the vault's Markdown is what the user writes and the databases are derived history (docs/VISION.md, principle 1); a familiarity value changes with time and with every review, so a copy in the file would be stale at once, would rewrite the file after every run (touching its modified time, the index and the auto-save of TASK-0200) and could collide with the user's own edits. Recommendation: the per-stack values stay in the database as today; the item page, the library and the JSON export show them in exactly the owner's shape (`source`, `rating`, `familiarity`, plus due date and stability) and the average; the file keeps only `referenced_by`. Other proposals: (1) the stack file is the mark, so old vaults are unchanged; (2) `referenced_by` is a list of stack ids (stable under renames and moves); (3) moving a file across a stack boundary changes its primary stack (the plan says how many items change) and the per-stack data stays with the stack; (4) a reference to a stack that is gone is a vault problem with one action to clean it; (5) CSV gets an optional `also_in` column, Obsidian folders map to normal folders unless marked, Anki decks map to stacks as today.

## Dependencies
None.

## Open Questions
(1) Should the per-stack values live in the database only, shown in the owner's shape and exported, or also be written into the card's file despite the concerns above? (2) What does the owner's 'rating' mean: the last rating given (Again, Hard, Good, Easy) or the difficulty mark (easy, medium, difficult)? (3) A rating in a run over everything (for example Today's rehearsal) updates every stack that holds the card, also those where it was not yet due: an early review there. Is that what the owner wants?
