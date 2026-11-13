---
id: TASK-0219
type: task
title: "Familiarity and difficulty per stack, averaged on the item"
status: done
epic: EPIC-026
parents: [STORY-0079]
estimate: 5
labels: [m11, srs, report, ui]
acceptance_criteria:
  - "Familiarity is computed for each (item, stack) from the reviews made in that stack, with that stack's exam date and settings, and the reports and the Needs attention card of a stack use the stack's value"
  - "The item's overall familiarity and difficulty are the average over its references; the item page and the library show the average and, per reference, `source`, `difficulty`, `familiarity` and the due date"
  - "The values are read from the database and never written into the card's file"
  - "The replay that judges familiarity (TASK-0184) still works and says which values it used"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0, and decided about stacks the same day: an item can be referenced by several stacks, held in a `referenced_by` field, with the original parent as the primary reference; a folder is a stack only when marked; a normal folder inside a stack folder belongs to the nearest stack above it; the vault is viewed like Obsidian's file explorer; each stack that references an item has its own independent memory of it (schedule, due date, familiarity, difficulty); the item's overall value is the average over its references; in a run over several stacks a card is due if it is due in any of them and a rating updates every selected stack that holds the card, also where it was not yet due (an early review is allowed); the per-stack values stay in the database and are never written into the card's file; the file holds only `referenced_by`; exports may be JSON, TOML or RON.

## Description
The owner: familiarity and the other values count in reference to the stack the item is viewed by, and the overall value of a flashcard is the average of all its references.

## Acceptance Criteria
- [x] Familiarity is computed for each (item, stack) from the reviews made in that stack, with that stack's exam date and settings, and the reports and the Needs attention card of a stack use the stack's value
- [x] The item's overall familiarity and difficulty are the average over its references; the item page and the library show the average and, per reference, `source`, `difficulty`, `familiarity` and the due date
- [x] The values are read from the database and never written into the card's file
- [x] The replay that judges familiarity (TASK-0184) still works and says which values it used

## Design Notes
`studyfrog_srs::familiarity` is pure and takes the item's history and the exam distance; call it per stack with that stack's reviews and inputs and keep a small `average_over` function. The library column 'Familiarity' shows the average with the per-stack values in the tooltip. The exports of TASK-0220 carry the same list.

## Dependencies
TASK-0218.

## Result (2026-11-04)
`fam_refs::reference_scores` computes familiarity for each (item, stack) from the reviews that stack made (`reviews_by_stack`), the stack's own exam and the item's views; `average_over` gives the item's value. Used by: the queue (the stack that makes the card due), Needs attention (the lowest reference, with the stack's name when the item has several), the familiarity report and stack percentages (a stack counts its own references), the library column (average, per-stack values in the tooltip), the item page (average familiarity and difficulty and one line per reference with source, familiarity, difficulty and due date) and the replay (one history per item and stack; the Settings text says how many). Values are read from the database; nothing is written to the file (tested). Views (reading time) are per item and count in every stack.

## Open Questions
None.
