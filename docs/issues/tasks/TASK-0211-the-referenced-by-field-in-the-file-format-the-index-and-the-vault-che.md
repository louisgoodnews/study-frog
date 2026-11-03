---
id: TASK-0211
type: task
title: The referenced_by field in the file format, the index and the vault checks
status: done
epic: EPIC-026
parents: [STORY-0079]
estimate: 5
labels: [m11, md, store]
acceptance_criteria:
  - "The item's front matter key `referenced_by` (a list of stack ids) is read and written, unknown keys are kept as now, and the round-trip property test covers it"
  - "The index has a membership table (stack, item, primary or referenced) built by the scanner and kept by the incremental reindex; a schema migration adds it to existing databases"
  - "A reference to a missing stack, to the item's own primary stack or listed twice is a vault problem with a clear message"
  - "Vaults without `referenced_by` read exactly as before"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0, and decided about stacks the same day: an item can be referenced by several stacks, held in a `referenced_by` field, with the original parent as the primary reference; a folder can be marked as a stack folder, otherwise it is a normal folder; a normal folder inside a stack folder belongs to the nearest stack above it; the vault should be viewed and organized as in Obsidian's file explorer; and the schedule, the due date, the familiarity and other values of an item belong to each stack that references it, the item's overall value being the average over its references (Flashcard 100 in stack A with 5 weeks and in stack B with 3 weeks).

## Description
The format and the index for references. No migration of vault files is needed: the stack file already marks a stack folder.

## Acceptance Criteria
- [x] The item's front matter key `referenced_by` (a list of stack ids) is read and written, unknown keys are kept as now, and the round-trip property test covers it
- [x] The index has a membership table (stack, item, primary or referenced) built by the scanner and kept by the incremental reindex; a schema migration adds it to existing databases
- [x] A reference to a missing stack, to the item's own primary stack or listed twice is a vault problem with a clear message
- [x] Vaults without `referenced_by` read exactly as before

## Design Notes
Done as `studyfrog_md::refs` (the key stays an untyped front-matter entry, so nothing else in the model changed), schema v20 (`schema_refs.rs`: `item_ref` and the view `membership`) and `studyfrog_store::refs_index` (`members_of`, `stacks_of`, `reference_problems`). A view, not a maintained table, so the membership can never disagree with the stacks and the files.

Follows the ADR of TASK-0210. `studyfrog_md` gets the key for notes, cards and questions; `studyfrog-store` a table and a membership query (`members_of(stack)`, `stacks_of(item)`).

## Dependencies
TASK-0210.

## Open Questions
None.
