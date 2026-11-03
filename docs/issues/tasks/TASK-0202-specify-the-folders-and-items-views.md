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
  - "docs/specs/vault-views.md describes the Folders view and the Items view, what each shows and which actions each has, with the owner's wishes as the reference"
  - "It shows how stack folders differ from normal folders in the tree and says what moving a file or a folder does to the primary stack"
  - "The owner has accepted it"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0. On the same day the owner decided about stacks: an item can be referenced by several stacks, held in a `referenced_by` field, with the original parent as the primary reference; stacks are not folders per se, but a folder can be marked as a 'stack' folder and is then also treated as a stack, while a 'non-stack' folder is a normal folder.

## Description
Write down the two views before building them.

## Acceptance Criteria
- [ ] docs/specs/vault-views.md describes the Folders view and the Items view, what each shows and which actions each has, with the owner's wishes as the reference
- [ ] It shows how stack folders differ from normal folders in the tree and says what moving a file or a folder does to the primary stack
- [ ] The owner has accepted it

## Design Notes
Owner's decisions (2026-11-03): folders are not stacks per se; a folder can be marked as a stack folder, otherwise it is a normal folder; an item can be referenced by several stacks. Still to settle: which files the tree shows (notes, stack files, PDFs, pictures, the wiki folder), how links are kept when files move (wikilinks by title stay, PDF and picture paths are rewritten as in the PDF rename), drag and drop and the keyboard, undo, files changed outside the app, and the names of the two views in the sidebar.

## Dependencies
TASK-0210, because what a folder and a stack are depends on it.

## Open Questions
None.
