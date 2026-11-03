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
  - "The owner has accepted it"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports.

## Description
Write down the two views before building them.

## Acceptance Criteria
- [ ] docs/specs/vault-views.md describes the Folders view and the Items view, what each shows and which actions each has, with the owner's wishes as the reference
- [ ] The owner has accepted it

## Design Notes
Questions to settle: does Folders show every file (notes, stack files, PDFs, pictures, the wiki folder) or only Markdown; are stacks the folders; does moving a file change its stack; what happens to links (wikilinks by title stay, PDF and picture paths are rewritten as in the PDF rename); drag and drop and the keyboard; undo; conflict with files changed outside the app; naming of the views in the sidebar.

## Dependencies
None.

## Open Questions
Is a folder always a stack, or can a folder be only a folder?
