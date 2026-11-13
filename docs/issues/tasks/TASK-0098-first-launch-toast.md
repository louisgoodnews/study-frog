---
id: TASK-0098
type: task
title: "A clickable notification on first launch"
status: done
epic: EPIC-020
parents: [STORY-0037]
estimate: 3
labels: [m8, ui]
acceptance_criteria:
  - "With no vault to reopen the app shows a welcome page and a toast; clicking the toast opens the vault dialog"
  - "The welcome page has no path field; the path field, Choose folder and recent vaults are in the vault dialog"
  - "A vault that opens closes the dialog and clears the toasts"
---

## Context


## Description
First launch: a clickable notification.

## Acceptance Criteria
- [x] With no vault to reopen the app shows a welcome page and a toast; clicking the toast opens the vault dialog
- [x] The welcome page has no path field; the path field, Choose folder and recent vaults are in the vault dialog
- [x] A vault that opens closes the dialog and clears the toasts

## Design Notes
`modal::Toast`, `view::toast`, `view::welcome`.

## Dependencies
TASK-0097.

## Open Questions
None.
