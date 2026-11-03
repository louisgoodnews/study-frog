---
id: STORY-0074
type: story
title: Markdown editor, formatting, macros and lists
status: backlog
epic: EPIC-026
parents: [EPIC-026]
estimate: 8
labels: [m11, ui, md]
acceptance_criteria:
  - "Tables can be written, previewed and inserted from the toolbar"
  - "Typing a macro such as {table} or {table:2x2} inserts its Markdown"
  - "Code blocks are highlighted by language, text can be underlined and coloured, and a note can show an automatic table of contents"
  - "Enter continues a list and the markers sit level with their text"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports.

## Description
Make the Markdown editor and its preview complete enough that the user does not leave the app to format a note.

## Acceptance Criteria
- [ ] Tables can be written, previewed and inserted from the toolbar
- [ ] Typing a macro such as {table} or {table:2x2} inserts its Markdown
- [ ] Code blocks are highlighted by language, text can be underlined and coloured, and a note can show an automatic table of contents
- [ ] Enter continues a list and the markers sit level with their text

## Design Notes
Tasks TASK-0197, TASK-0198, TASK-0214, TASK-0215 and TASK-0216, bugs BUG-0007 and BUG-0008.

## Dependencies
None.

## Open Questions
None.
