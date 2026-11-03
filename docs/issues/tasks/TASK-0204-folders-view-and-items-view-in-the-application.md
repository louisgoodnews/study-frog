---
id: TASK-0204
type: task
title: Folders view and Items view in the application
status: backlog
epic: EPIC-026
parents: [STORY-0076]
estimate: 8
labels: [m11, ui]
acceptance_criteria:
  - "The sidebar offers Folders and Items; Items is the library as it is now"
  - "Folders shows the tree of the vault, with stack folders told apart from normal folders; creates, renames, moves (drag and drop and a menu) and deletes folders and files, marks a folder as a stack or as normal, and opens a file"
  - "Every action is available from the keyboard and has an accessible name"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0. On the same day the owner decided about stacks: an item can be referenced by several stacks, held in a `referenced_by` field, with the original parent as the primary reference; stacks are not folders per se, but a folder can be marked as a 'stack' folder and is then also treated as a stack, while a 'non-stack' folder is a normal folder.

## Description
The Folders view for organizing and the Items view for working with the items, as two screens.

## Acceptance Criteria
- [ ] The sidebar offers Folders and Items; Items is the library as it is now
- [ ] Folders shows the tree of the vault, with stack folders told apart from normal folders; creates, renames, moves (drag and drop and a menu) and deletes folders and files, marks a folder as a stack or as normal, and opens a file
- [ ] Every action is available from the keyboard and has an accessible name

## Design Notes
A tree widget that draws only the rows in view (the vault can hold thousands of files); reuse the conventions of the stack page.

## Dependencies
TASK-0202, TASK-0203 and TASK-0217.

## Open Questions
None.
