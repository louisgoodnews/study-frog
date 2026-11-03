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
  - "An ADR and an update of docs/specs/markdown-schema.md state: a folder is a stack when it holds a stack file (`_stack.md`, as today) and a normal folder otherwise; an item's primary stack is the nearest stack folder above it, none meaning the Inbox; `referenced_by` lists the stacks (by id) that also reference the item"
  - "They state what scheduling, status, limits and roll-ups do for an item in several stacks, what moving files, deleting a stack and removing the stack mark do to references, and what import and export (JSON, CSV, Obsidian, Anki) do"
  - "The owner has accepted it"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0. On the same day the owner decided about stacks: an item can be referenced by several stacks, held in a `referenced_by` field, with the original parent as the primary reference; stacks are not folders per se, but a folder can be marked as a 'stack' folder and is then also treated as a stack, while a 'non-stack' folder is a normal folder.

## Description
Write the rules down before changing the model.

## Acceptance Criteria
- [ ] An ADR and an update of docs/specs/markdown-schema.md state: a folder is a stack when it holds a stack file (`_stack.md`, as today) and a normal folder otherwise; an item's primary stack is the nearest stack folder above it, none meaning the Inbox; `referenced_by` lists the stacks (by id) that also reference the item
- [ ] They state what scheduling, status, limits and roll-ups do for an item in several stacks, what moving files, deleting a stack and removing the stack mark do to references, and what import and export (JSON, CSV, Obsidian, Anki) do
- [ ] The owner has accepted it

## Design Notes
Owner's decisions (2026-11-03): an item can be referenced by several stacks; `referenced_by` holds the references while the original parent is the primary reference; stacks are not folders per se; a folder can be marked as a stack folder and then is also a stack, a non-stack folder is a normal folder. Proposals for the record: (1) the stack file is the mark, so nothing changes for old vaults; (2) `referenced_by` is a list of stack ids in the item's front matter (stable under renames and moves; the UI shows names); (3) a normal folder inside a stack folder only organizes: its items belong to the nearest stack above; a stack may sit inside a normal folder; (4) one schedule per item: the primary stack's scheduler and limits decide, the other stacks list the item and count it in their summaries; an item is never queued twice in one run; global counts count it once; (5) moving a file across a stack boundary changes its primary stack (the plan says how many items change) and removes the new primary from `referenced_by` if listed; (6) a reference to a stack that is gone is shown as a vault problem and can be cleaned with one action; (7) JSON export carries the references, CSV gets an optional `also_in` column (names separated by semicolons), Obsidian folders map to normal folders unless the user marks them, Anki decks map to stacks as today.

## Dependencies
None.

## Open Questions
Is it right that a normal folder inside a stack folder belongs to the nearest stack above? And should an item's schedule follow the primary stack only, as proposed?
