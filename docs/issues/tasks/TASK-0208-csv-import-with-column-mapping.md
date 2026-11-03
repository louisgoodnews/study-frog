---
id: TASK-0208
type: task
title: CSV import with column mapping
status: backlog
epic: EPIC-026
parents: [STORY-0078]
estimate: 5
labels: [m11, ui, store]
acceptance_criteria:
  - "A CSV or TSV file with other column names, other orders or no header can be imported by choosing which column is the front, the back, the title, the tags and the stack"
  - "The delimiter and the text encoding are detected and can be changed, and the first rows are shown"
  - "Files with only two columns import as front and back; a column of kinds can make notes and questions"
  - "A file that cannot be read says what was expected and what it found"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports.

## Description
CSV import exists, but only for files whose header has `front` and `back` (and `stack`, `kind`, `title`, `tags`); anything else fails with one line.

## Acceptance Criteria
- [ ] A CSV or TSV file with other column names, other orders or no header can be imported by choosing which column is the front, the back, the title, the tags and the stack
- [ ] The delimiter and the text encoding are detected and can be changed, and the first rows are shown
- [ ] Files with only two columns import as front and back; a column of kinds can make notes and questions
- [ ] A file that cannot be read says what was expected and what it found

## Design Notes
Extend `xfer_csv.rs` with a read plan that takes a mapping; the dialog shows a mapping step between choosing the file and the preview; remember the mapping per file name pattern. The semicolon and tab delimiters, quotes and line breaks in cells are kept as now.

## Dependencies
TASK-0209 for how the expected layout is told.

## Open Questions
Which CSV files did the owner try, and what happened? (to reproduce)
