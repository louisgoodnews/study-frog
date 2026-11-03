---
id: TASK-0217
type: task
title: Mark a folder as a stack or as a normal folder
status: backlog
epic: EPIC-026
parents: [STORY-0079]
estimate: 3
labels: [m11, store, ui]
acceptance_criteria:
  - "A folder can be made a stack (a stack file with a new id and the folder's name as title is written) and a stack can be made a normal folder"
  - "Making a normal folder takes a confirmation that names what changes: the stack's settings, status, goals and subjects that go with the file, sub-stacks, items that reference it, and where its items go (the stack above or the Inbox)"
  - "The change can be undone, and a failure half-way leaves the vault as it was"
  - "The actions are in the Folders view, in the stack page's More actions, and in the New folder dialog as a choice"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0. On the same day the owner decided about stacks: an item can be referenced by several stacks, held in a `referenced_by` field, with the original parent as the primary reference; stacks are not folders per se, but a folder can be marked as a 'stack' folder and is then also treated as a stack, while a 'non-stack' folder is a normal folder.

## Description
The owner's switch between a stack folder and a normal folder.

## Acceptance Criteria
- [ ] A folder can be made a stack (a stack file with a new id and the folder's name as title is written) and a stack can be made a normal folder
- [ ] Making a normal folder takes a confirmation that names what changes: the stack's settings, status, goals and subjects that go with the file, sub-stacks, items that reference it, and where its items go (the stack above or the Inbox)
- [ ] The change can be undone, and a failure half-way leaves the vault as it was
- [ ] The actions are in the Folders view, in the stack page's More actions, and in the New folder dialog as a choice

## Design Notes
A pure plan with an executor and an undo snapshot, like the bulk edits. Making a stack adopts the items in the folder that had no stack of their own. Unmarking keeps the files of the items, writes the stack file's content into the undo snapshot, drops references to the stack from items (listing how many) and leaves sub-stacks as they are.

## Dependencies
TASK-0210 and TASK-0211.

## Open Questions
None.
