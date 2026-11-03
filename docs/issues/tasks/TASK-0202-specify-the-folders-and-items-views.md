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
  - "It says how stack files appear in the Folders view (as files with their own icon) and what moving a file or a folder does and does not change"
  - "The owner has accepted it"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports. The owner answered the open questions the same day (recorded below).

## Description
Write down the two views before building them.

## Acceptance Criteria
- [ ] docs/specs/vault-views.md describes the Folders view and the Items view, what each shows and which actions each has, with the owner's wishes as the reference
- [ ] It says how stack files appear in the Folders view (as files with their own icon) and what moving a file or a folder does and does not change
- [ ] The owner has accepted it

## Design Notes
Owner's answer (2026-11-03): folders are folders and not stacks; a stack is a file that points to its contents. So moving a file between folders changes no stack membership. Still to settle: which files the tree shows (notes, stack files, PDFs, pictures, the wiki folder), how links are kept when files move (wikilinks by title stay, PDF and picture paths are rewritten as in the PDF rename), drag and drop and the keyboard, undo, files changed outside the app, and the names of the two views in the sidebar.

## Dependencies
TASK-0210, because what a folder and a stack are depends on it.

## Open Questions
None.
