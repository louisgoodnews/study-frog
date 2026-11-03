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
  - "Links and picture and PDF references that name a moved file are rewritten; wikilinks by title keep working; no stack membership changes"
  - "The index is updated and a failure half-way leaves the vault as it was"
  - "Hostile names and paths are refused"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports. The owner answered the open questions the same day (recorded below).

## Description
The use cases behind the Folders view, as pure plans with an executor, tested against a real vault.

## Acceptance Criteria
- [ ] A folder can be created, renamed, moved and deleted, and a file can be moved or renamed, inside the vault only
- [ ] Links and picture and PDF references that name a moved file are rewritten; wikilinks by title keep working; no stack membership changes
- [ ] The index is updated and a failure half-way leaves the vault as it was
- [ ] Hostile names and paths are refused

## Design Notes
Build on the PDF rename (`pdf_ops`) and the reindex; a plan lists every file change, is applied as a unit with rollback, and returns an undo snapshot like the bulk edits.

## Dependencies
TASK-0202 and TASK-0211.

## Open Questions
None.
