---
id: TASK-0202
type: task
title: Specify the folders and items views
status: backlog
epic: EPIC-026
parents: [STORY-0076]
estimate: 3
labels: [m11, docs, decision]
acceptance_criteria:
  - "docs/specs/vault-views.md describes the Folders view, modelled on Obsidian's file explorer, and the Items view, what each shows and which actions each has"
  - "It shows how stack folders differ from normal folders in the tree and says what moving a file or a folder does to the primary stack"
  - "The owner has accepted it"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0, and decided about stacks the same day: an item can be referenced by several stacks, held in a `referenced_by` field, with the original parent as the primary reference; a folder can be marked as a stack folder, otherwise it is a normal folder; a normal folder inside a stack folder belongs to the nearest stack above it; the vault should be viewed and organized as in Obsidian's file explorer; and the schedule, the due date, the familiarity and other values of an item belong to each stack that references it, the item's overall value being the average over its references (Flashcard 100 in stack A with 5 weeks and in stack B with 3 weeks).

## Description
Write down the two views before building them.

## Acceptance Criteria
- [ ] docs/specs/vault-views.md describes the Folders view, modelled on Obsidian's file explorer, and the Items view, what each shows and which actions each has
- [ ] It shows how stack folders differ from normal folders in the tree and says what moving a file or a folder does to the primary stack
- [ ] The owner has accepted it

## Design Notes
Reference (the owner's screenshot of Obsidian's file explorer, 2026-11-03): a small toolbar over the tree with New note, New folder, Sort order, Expand or collapse all and Collapse all; a tree with a chevron for each folder, indent guides, the file name without its extension and a small tag for other types (PDF); the `_stack` file is listed inside its folder. The owner finds this view more helpful than StudyFrog's. Still to settle: whether the stack file is a row of its own or shown as a mark on the folder, which files the tree shows (notes, PDFs, pictures, the wiki folder), how links are kept when files move (wikilinks by title stay, PDF and picture paths are rewritten as in the PDF rename), drag and drop, inline rename and the keyboard (F2, Delete, arrows), the context menu, undo, files changed outside the app, and the names of the two views in the sidebar.

## Dependencies
TASK-0210, because what a folder and a stack are depends on it.

## Open Questions
None.
