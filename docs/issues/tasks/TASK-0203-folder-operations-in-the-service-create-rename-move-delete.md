---
id: TASK-0203
type: task
title: Folder operations in the service (create, rename, move, delete)
status: backlog
epic: EPIC-026
parents: [STORY-0076]
estimate: 5
labels: [m11, store]
acceptance_criteria:
  - "A folder can be created, renamed, moved and deleted, and a file can be moved or renamed, inside the vault only"
  - "Links and picture and PDF references that name a moved file are rewritten; wikilinks by title keep working"
  - "A move that crosses a stack boundary says how many items change their primary stack, asks before doing it, and fixes `referenced_by`"
  - "The index is updated and a failure half-way leaves the vault as it was"
  - "Hostile names and paths are refused"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0. On the same day the owner decided about stacks: an item can be referenced by several stacks, held in a `referenced_by` field, with the original parent as the primary reference; stacks are not folders per se, but a folder can be marked as a 'stack' folder and is then also treated as a stack, while a 'non-stack' folder is a normal folder.

## Description
The use cases behind the Folders view, as pure plans with an executor, tested against a real vault.

## Acceptance Criteria
- [ ] A folder can be created, renamed, moved and deleted, and a file can be moved or renamed, inside the vault only
- [ ] Links and picture and PDF references that name a moved file are rewritten; wikilinks by title keep working
- [ ] A move that crosses a stack boundary says how many items change their primary stack, asks before doing it, and fixes `referenced_by`
- [ ] The index is updated and a failure half-way leaves the vault as it was
- [ ] Hostile names and paths are refused

## Design Notes
Build on the PDF rename (`pdf_ops`) and the reindex; a plan lists every file change, is applied as a unit with rollback, and returns an undo snapshot like the bulk edits.

## Dependencies
TASK-0202 and TASK-0211.

## Open Questions
None.
