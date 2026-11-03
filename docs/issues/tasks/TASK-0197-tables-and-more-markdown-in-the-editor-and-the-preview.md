---
id: TASK-0197
type: task
title: Tables and more Markdown in the editor and the preview
status: backlog
epic: EPIC-026
parents: [STORY-0074]
estimate: 5
labels: [m11, ui, md]
acceptance_criteria:
  - "The parser reads pipe tables (with alignment) and the preview draws them with a header row"
  - "The toolbar has a Table button that inserts a table"
  - "The list of other features chosen with the owner (see the open question) works in both the editor and the preview, and round-trips through save"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports.

## Description
Add tables and further CommonMark and GitHub-flavoured features to `studyfrog-md` blocks and to the preview, so a card or a note can hold them.

## Acceptance Criteria
- [ ] The parser reads pipe tables (with alignment) and the preview draws them with a header row
- [ ] The toolbar has a Table button that inserts a table
- [ ] The list of other features chosen with the owner (see the open question) works in both the editor and the preview, and round-trips through save

## Design Notes
A `Block::Table` with a header, alignments and rows in `studyfrog-md`; the renderer uses a grid with the theme's spacing; unknown Markdown is kept verbatim as now. Cards in a rehearsal show tables too.

## Dependencies
None.

## Open Questions
Which "etc." does the owner want besides tables: nested lists, footnotes, definition lists, task lists with nesting, highlighted code blocks, math, callouts, a table of contents?
