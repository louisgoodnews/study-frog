---
id: STORY-0079
type: story
title: Stack folders and items referenced by several stacks
status: backlog
epic: EPIC-026
parents: [EPIC-026]
estimate: 8
labels: [m11, store, md, ui, decision]
acceptance_criteria:
  - "A folder is a stack when it is marked as one and is a normal folder otherwise, and the user can change that"
  - "An item has one primary stack (where it lives) and can be referenced by other stacks through its `referenced_by` field"
  - "Queues, reports, the calendar and import and export treat a referenced item as part of every stack that references it"
  - "Existing vaults keep working without a migration"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0. On the same day the owner decided about stacks: an item can be referenced by several stacks, held in a `referenced_by` field, with the original parent as the primary reference; stacks are not folders per se, but a folder can be marked as a 'stack' folder and is then also treated as a stack, while a 'non-stack' folder is a normal folder.

## Description
Today a folder with a `_stack.md` is a stack and its items are the files inside it; items outside any stack go to an implicit Inbox. The owner wants an item to be usable in several stacks and wants folders that are only folders. Most of the model is already there: the existing stack file is the marker of a stack folder, so old vaults stay valid.

## Acceptance Criteria
- [ ] A folder is a stack when it is marked as one and is a normal folder otherwise, and the user can change that
- [ ] An item has one primary stack (where it lives) and can be referenced by other stacks through its `referenced_by` field
- [ ] Queues, reports, the calendar and import and export treat a referenced item as part of every stack that references it
- [ ] Existing vaults keep working without a migration

## Design Notes
Tasks TASK-0210 to TASK-0213 and TASK-0217. Smaller than first feared (no migration), but it still touches queues, summaries, reports and the importers, so the decision record comes first.

## Dependencies
None.

## Open Questions
None.
