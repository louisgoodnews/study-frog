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
  - "Folders shows the tree of the vault, creates, renames, moves (drag and drop and a menu) and deletes folders and files, and opens a file"
  - "Every action is available from the keyboard and has an accessible name"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports.

## Description
The Folders view for organizing and the Items view for working with the items, as two screens.

## Acceptance Criteria
- [ ] The sidebar offers Folders and Items; Items is the library as it is now
- [ ] Folders shows the tree of the vault, creates, renames, moves (drag and drop and a menu) and deletes folders and files, and opens a file
- [ ] Every action is available from the keyboard and has an accessible name

## Design Notes
A tree widget that draws only the rows in view (the vault can hold thousands of files); reuse the status and tab conventions of the stack page.

## Dependencies
TASK-0202 and TASK-0203.

## Open Questions
None.
