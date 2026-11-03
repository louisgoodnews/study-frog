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
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports.

## Description
The application does not say what it expects, so users guess.

## Acceptance Criteria
- [ ] The import dialog lists every format it reads with the layout it expects and a short example, and a Save example button writes a sample file
- [ ] The Anki export settings and the Obsidian vault rules are told in words, with a link to a help page
- [ ] Error messages name the expected layout
- [ ] The help has an Import formats page and the list in the dialog is generated from the same table as the importers

## Design Notes
Add a `layout` text and `example` to each `Adapter` record (`xfer_adapters.rs`) so the dialog and the help page read one source; the prompts of TASK-0205 use the same layouts.

## Dependencies
None.

## Open Questions
None.
