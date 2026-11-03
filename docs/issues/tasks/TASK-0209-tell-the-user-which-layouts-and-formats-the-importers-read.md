---
id: TASK-0209
type: task
title: Tell the user which layouts and formats the importers read
status: backlog
epic: EPIC-026
parents: [STORY-0078]
estimate: 3
labels: [m11, ui, docs]
acceptance_criteria:
  - "The import dialog lists every format it reads with the layout it expects and a short example, and a Save example button writes a sample file"
  - "The Anki export settings and the Obsidian vault rules are told in words, with a link to a help page"
  - "Error messages name the expected layout"
  - "The help has an Import formats page and the list in the dialog is generated from the same table as the importers"
  - "The prompts of TASK-0205 take their layouts and examples from the same table"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports. The owner answered the open questions the same day (recorded below).

## Description
The application does not say what it expects, so users and their AIs guess. The owner did not know which CSV format the application expects.

## Acceptance Criteria
- [ ] The import dialog lists every format it reads with the layout it expects and a short example, and a Save example button writes a sample file
- [ ] The Anki export settings and the Obsidian vault rules are told in words, with a link to a help page
- [ ] Error messages name the expected layout
- [ ] The help has an Import formats page and the list in the dialog is generated from the same table as the importers
- [ ] The prompts of TASK-0205 take their layouts and examples from the same table

## Design Notes
Add a `layout` text and `example` to each `Adapter` record (`xfer_adapters.rs`) so the dialog, the help page and the prompts read one source.

## Dependencies
None.

## Open Questions
None.
