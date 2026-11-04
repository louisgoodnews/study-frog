---
id: TASK-0220
type: task
title: Export formats TOML and RON next to JSON
status: done
epic: EPIC-026
parents: [STORY-0079]
estimate: 3
labels: [m11, store, ui]
acceptance_criteria:
  - "The export dialog offers JSON, TOML and RON for the library or a stack, with the same data: items, references and, in the advanced mode, the per-stack values (`source`, `difficulty`, `familiarity`, due date) with the average and the review history"
  - "The three formats read back to the same bundle where the importer reads them, and the dialog says which formats can be imported again"
  - "Files of the three formats are valid for other tools, checked by parsing them with the standard crates in the tests"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0, and decided about stacks the same day: an item can be referenced by several stacks, held in a `referenced_by` field, with the original parent as the primary reference; a folder is a stack only when marked; a normal folder inside a stack folder belongs to the nearest stack above it; the vault is viewed like Obsidian's file explorer; each stack that references an item has its own independent memory of it (schedule, due date, familiarity, difficulty); the item's overall value is the average over its references; in a run over several stacks a card is due if it is due in any of them and a rating updates every selected stack that holds the card, also where it was not yet due (an early review is allowed); the per-stack values stay in the database and are never written into the card's file; the file holds only `referenced_by`; exports may be JSON, TOML or RON.

## Description
The owner: exports could be JSON, TOML or RON, since the per-stack values stay in the database until exported.

## Acceptance Criteria
- [x] The export dialog offers JSON, TOML and RON for the library or a stack, with the same data: items, references and, in the advanced mode, the per-stack values (`source`, `difficulty`, `familiarity`, due date) with the average and the review history
- [x] The three formats read back to the same bundle where the importer reads them, and the dialog says which formats can be imported again
- [x] Files of the three formats are valid for other tools, checked by parsing them with the standard crates in the tests

## Design Notes
Build or buy (P10): the `toml` crate is already in the workspace; for RON compare the `ron` crate with a small writer (export only is a plain data dump); record the choice in the task. Import: JSON today; TOML and RON need a reader, which the crates give; an in-house RON reader is not worth it. The table of adapters of TASK-0209 lists the three formats with their layouts.

## Dependencies
TASK-0212 and TASK-0219.

## Open Questions
None.
