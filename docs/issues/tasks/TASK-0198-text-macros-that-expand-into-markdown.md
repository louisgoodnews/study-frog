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
  - "The macros for the owner's list exist: {table}, {table:CxR}, {toc}, {code}, {code:LANG}, {quote}, {image}, {color:NAME}"
  - "Macros are a table of plain data (name, argument pattern, expansion function) so that others can be added"
  - "The user can see which macros exist (a hint in the editor and a help page)"
  - "A macro expands when its closing brace is typed, never while it is still being typed"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports. The owner answered the open questions the same day (recorded below).

## Description
Let the user type a short macro in the text and have it replaced by the Markdown it stands for.

## Acceptance Criteria
- [ ] Typing {table} inserts a default table and {table:2x2} a table of 2 columns and 2 rows, with the cursor in the first cell
- [ ] The macros for the owner's list exist: {table}, {table:CxR}, {toc}, {code}, {code:LANG}, {quote}, {image}, {color:NAME}
- [ ] Macros are a table of plain data (name, argument pattern, expansion function) so that others can be added
- [ ] The user can see which macros exist (a hint in the editor and a help page)
- [ ] A macro expands when its closing brace is typed, never while it is still being typed

## Design Notes
Owner's answer (2026-11-03): the macros are those for the formatting features of TASK-0197, TASK-0214, TASK-0215 and TASK-0216. Following the owner's token-wise idea for saving (see TASK-0200) a macro is a token: it expands as soon as its closing brace is typed. Macros live in `md_macros.rs` as a list of records with one free function each; unknown names are left as typed. {color:NAME} wraps the selection or places an empty span; {image} opens the picture chooser.

## Dependencies
TASK-0197, TASK-0214, TASK-0215 and TASK-0216 for the syntax each macro writes.

## Open Questions
Is expanding at the closing brace right, or would the owner rather press Tab? (the answer 'see above' did not say)
