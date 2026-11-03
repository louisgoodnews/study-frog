---
id: TASK-0198
type: task
title: Text macros that expand into Markdown
status: backlog
epic: EPIC-026
parents: [STORY-0074]
estimate: 5
labels: [m11, ui, md]
acceptance_criteria:
  - "Typing {table} inserts a default table and {table:2x2} a table of 2 columns and 2 rows, with the cursor in the first cell"
  - "Macros are a table of plain data (name, argument pattern, expansion function) so that others can be added"
  - "The user can see which macros exist (a help page and a hint in the editor)"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports.

## Description
Let the user type a short macro in the text and have it replaced by the Markdown it stands for.

## Acceptance Criteria
- [ ] Typing {table} inserts a default table and {table:2x2} a table of 2 columns and 2 rows, with the cursor in the first cell
- [ ] Macros are a table of plain data (name, argument pattern, expansion function) so that others can be added
- [ ] The user can see which macros exist (a help page and a hint in the editor)

## Design Notes
Macros live in `md_edit.rs` or a new `md_macros.rs` as a list of records with a free function each. Expansion is checked when the closing brace is typed. Candidates beyond tables, to be chosen: {date}, {today}, {time}, {toc}, {cloze}, {link}, {img}.

## Dependencies
TASK-0197 for the table syntax.

## Open Questions
When does a macro expand: at the closing brace, on Enter or Tab, or from a small menu? Which other macros?
