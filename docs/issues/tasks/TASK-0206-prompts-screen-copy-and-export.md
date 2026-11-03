---
id: TASK-0206
type: task
title: Prompts screen with copy and export
status: backlog
epic: EPIC-026
parents: [STORY-0077]
estimate: 3
labels: [m11, ui]
acceptance_criteria:
  - "A Prompts section (in the Assistant screen and linked from Help and from the import dialog) lists the prompts and each has Copy to clipboard and Save as .md"
  - "All prompts can be exported into a folder in one step"
  - "Copy works without a connection to any assistant"
  - "The import dialog offers 'Copy a prompt that makes this file' next to each format"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports. The owner answered the open questions the same day (recorded below).

## Description
Make the prompts easy to reach for use outside the application, right where the user is about to import.

## Acceptance Criteria
- [ ] A Prompts section (in the Assistant screen and linked from Help and from the import dialog) lists the prompts and each has Copy to clipboard and Save as .md
- [ ] All prompts can be exported into a folder in one step
- [ ] Copy works without a connection to any assistant
- [ ] The import dialog offers 'Copy a prompt that makes this file' next to each format

## Design Notes
A pure registry of prompts as plain data; the clipboard through egui; the file dialog through the host as for backups.

## Dependencies
TASK-0205.

## Open Questions
None.
