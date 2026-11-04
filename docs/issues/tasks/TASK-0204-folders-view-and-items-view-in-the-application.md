---
id: TASK-0204
type: task
title: Folders view and Items view in the application
status: done
epic: EPIC-026
parents: [STORY-0076]
estimate: 8
labels: [m11, ui]
acceptance_criteria:
  - "The sidebar offers Folders and Items; Items is the library as it is now"
  - "Folders is a file explorer like Obsidian's: toolbar, tree with chevrons and indent guides, stack folders told apart from normal folders; it creates, renames (in place), moves (drag and drop and a menu) and deletes folders and files, marks a folder as a stack or as normal, and opens a file"
  - "Every action is available from the keyboard and has an accessible name"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0, and decided about stacks the same day: an item can be referenced by several stacks, held in a `referenced_by` field, with the original parent as the primary reference; a folder can be marked as a stack folder, otherwise it is a normal folder; a normal folder inside a stack folder belongs to the nearest stack above it; the vault should be viewed and organized as in Obsidian's file explorer; and the schedule, the due date, the familiarity and other values of an item belong to each stack that references it, the item's overall value being the average over its references (Flashcard 100 in stack A with 5 weeks and in stack B with 3 weeks).

## Description
The Folders view for organizing and the Items view for working with the items, as two screens.

## Acceptance Criteria
- [x] The sidebar offers Folders and Items; Items is the library as it is now
- [x] Folders is a file explorer like Obsidian's: toolbar, tree with chevrons and indent guides, stack folders told apart from normal folders; it creates, renames (in place), moves (drag and drop and a menu) and deletes folders and files, marks a folder as a stack or as normal, and opens a file
- [x] Every action is available from the keyboard and has an accessible name

## Design Notes
A tree widget that draws only the rows in view (the vault can hold thousands of files); look at the owner's reference in TASK-0202.

## Dependencies
TASK-0202, TASK-0203 and TASK-0217.

## Open Questions
None.
