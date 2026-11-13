---
id: TASK-0097
type: task
title: "Create dialog"
status: done
epic: EPIC-020
parents: [STORY-0037]
estimate: 5
labels: [m8, ui]
acceptance_criteria:
  - "The top bar Create button opens a modal with kind tabs (Stack, Card, Note, Question), title, text fields per kind, parent stack picker, tags, Create another and Create"
  - "Missing input is explained in the dialog; a refusal from the vault keeps what was typed; Escape, the cross, Cancel and a click outside close it"
  - "A new stack is opened after it is made; Create another keeps kind, stack and tags"
---

## Context


## Description
Create dialog.

## Acceptance Criteria
- [x] The top bar Create button opens a modal with kind tabs (Stack, Card, Note, Question), title, text fields per kind, parent stack picker, tags, Create another and Create
- [x] Missing input is explained in the dialog; a refusal from the vault keeps what was typed; Escape, the cross, Cancel and a click outside close it
- [x] A new stack is opened after it is made; Create another keeps kind, stack and tags

## Design Notes
`ui::modal` (update), `modal_types`, `view::modal` (egui Modal). The old Create dropdown and the inline new-stack form are gone; study sessions and board cards are made on their own screens.

## Dependencies
TASK-0096.

## Open Questions
None.
